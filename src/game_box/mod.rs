pub mod rules;

use crate::utils::{base10_to_base2, records_in_file, write_records_header};
use rand::rngs::ThreadRng;
use rand::seq::{IteratorRandom, SliceRandom};
use std::collections::HashMap;
use std::fs::File;
use std::io::ErrorKind::NotFound;
use std::io::{BufReader, BufWriter, Read, Write};

#[derive(Clone)]
pub struct IntermediateResult {
    pub available_hands: u16,
    pub score: u64,
}

impl IntermediateResult {
    pub fn new() -> IntermediateResult {
        IntermediateResult {
            available_hands: 0,
            score: 0,
        }
    }
}

pub struct MCScore {
    // (score, hand, available hands), (hits, expected score)
    mc_scores: HashMap<(u8, u8, u16), (u64, u64)>,
    name: String,
    laps: u64,
}

impl MCScore {
    pub fn new() -> MCScore {
        MCScore {
            mc_scores: HashMap::new(),
            name: "mcscore".to_string(),
            laps: 0,
        }
    }

    pub fn update_scores(&mut self, intermediate_results: &Vec<IntermediateResult>) -> u64 {
        let bonus = if intermediate_results[0..6]
            .iter()
            .map(|i| i.score)
            .sum::<u64>()
            < 63
        {
            0u64
        } else {
            50u64
        };
        let total_score = intermediate_results.iter().map(|i| i.score).sum::<u64>() + bonus;

        let mut hand: u8;
        let mut score: u8;
        for (i, ir) in intermediate_results.iter().enumerate() {
            hand = i as u8;
            score = ir.score as u8;

            match self
                .mc_scores
                .get(&(score, hand, ir.available_hands))
            {
                Some((hits, value)) => {
                    self.mc_scores.insert(
                        (score, hand, ir.available_hands),
                        (*hits + 1, *value + total_score),
                    );
                }
                None => {
                    self.mc_scores.insert(
                        (score, hand, ir.available_hands),
                        (1, total_score),
                    );
                }
            }
        }

        self.laps += 1;
        self.laps
    }

    pub fn update_optimal_game(&self, optimal_game: &mut HashMap<(u8, u8, u16), f64>) {
        optimal_game.clear();

        let mut actual_score: f64;

        for ((score, hand, available_hands), (hits, value)) in self.mc_scores.iter() {
            actual_score = *value as f64 / *hits as f64;
            match optimal_game.get(&(*score, *hand, *available_hands)) {
                Some(expected_score) => {
                    if actual_score > *expected_score {
                        optimal_game
                            .insert((*score, *hand, *available_hands), actual_score);
                    }
                }
                None => {
                    optimal_game
                        .insert((*score, *hand, *available_hands), actual_score);
                }
            }
        }
    }

    pub fn save_scores(&self, path: &str) -> Result<(), String> {
        let path_name = &format!("{}/{}.bin", path, self.name);
        let mut buf_writer = match File::create(path_name) {
            Ok(f) => BufWriter::new(f),
            Err(e) => return Err(format!("Error while open/create file {}: {}", path_name, e)),
        };

        let _ = write_records_header(&mut buf_writer, &vec![&self.mc_scores], path_name)?;

        let mut buf = [0u8; 20];
        let mut offset: usize;

        for ((score, hand, available_hands), (hits, value)) in &self.mc_scores {
            offset = 2;
            buf[0] = *score;
            buf[1] = *hand;
            available_hands.to_le_bytes().iter().for_each(|v| {
                buf[offset] = *v;
                offset += 1;
            });
            hits.to_le_bytes().iter().for_each(|v| {
                buf[offset] = *v;
                offset += 1;
            });
            value.to_le_bytes().iter().for_each(|v| {
                buf[offset] = *v;
                offset += 1;
            });

            if let Err(e) = buf_writer.write_all(&buf) {
                return Err(format!("Error while writing to file {}: {}", path_name, e));
            }
        }

        if let Err(e) = buf_writer.flush() {
            return Err(format!("Error while writing to file {}: {}", path_name, e));
        }
        Ok(())
    }

    pub fn load_scores(&mut self, path: &str) -> Result<Option<()>, String> {
        let path_name = &format!("{}/{}.bin", path, self.name);
        let mut buf_reader = match File::open(path_name) {
            Ok(f) => BufReader::new(f),
            Err(e) => {
                return match e.kind() {
                    NotFound => Ok(None),
                    _ => Err(format!("Error while open file {}: {}", path_name, e)),
                }
            }
        };

        let mut buf = [0u8; 20];
        let mut n_records = records_in_file(&mut buf_reader, path_name)?;

        while n_records > 0 {
            match buf_reader.read_exact(&mut buf) {
                Ok(()) => {
                    let score = buf[0];
                    let hand = buf[1];
                    let available_hands = u16::from_le_bytes(buf[2..4].try_into().unwrap());
                    let hits = u64::from_le_bytes(buf[4..12].try_into().unwrap());
                    let value = u64::from_le_bytes(buf[12..20].try_into().unwrap());

                    self.mc_scores.insert((score, hand, available_hands), (hits, value));
                }
                Err(e) => {
                    return Err(format!(
                        "Error while reading from file {}: {}",
                        path_name, e
                    ));
                }
            }
            n_records -= 1;
        }
        Ok(Some(()))
    }

    pub fn statistics(&self) -> Result<String, String> {
        if self.mc_scores.len() == 0  {
            return Err("MC Score hash table is empty".to_string());
        }

        let (
            mut max_hits,
            mut min_hits,
            mut max_score_sum,
            mut min_score_sum,
            mut max_score,
            mut min_score,
        ) = (0u64, u64::MAX, 0u64, u64::MAX, 0.0f64, f64::MAX);

        let mut debug_count = 100;
        let mut count: usize = 0;
        let mut score_avg: f64;
        let mut buckets = [0u64;20];
        for ((score, hand, available_hands),(hits, value)) in &self.mc_scores {
            max_hits = max_hits.max(*hits);
            min_hits = min_hits.min(*hits);
            max_score_sum = max_score_sum.max(*value);
            min_score_sum = min_score_sum.min(*value);
            score_avg = *value as f64 / *hits as f64;
            max_score = max_score.max(score_avg);
            min_score = min_score.min(score_avg);
            if *hits < 20 {
                buckets[*hits as usize] += 1;
                if *hits == 1 && debug_count > 0 {
                    let a_vec = base10_to_base2(*available_hands, true);
                    println!("{:2} [{:2}] {:51} -> {:12} {:12}", *score, *hand + 1, format!("{:?}", a_vec), *hits, *value);
                    debug_count -= 1;
                }
            }
            count += 1;
        }

        buckets[0] = 1966080 - count as u64;
        let result = format!(
            "Records: {}\nHits: min {} max {}\nScore sum: min {} max: {}\nScore: min {:.2} max {:.2}\nHit buckets [0-19]: {:?}",
            count, min_hits, max_hits, min_score_sum, max_score_sum, min_score, max_score, buckets
        );

        Ok(result)
    }
}

pub struct MCGame {
    rng: ThreadRng,
    hand_sequence: Vec<usize>,
}

impl MCGame {
    pub fn new() -> MCGame {
        MCGame {
            rng: rand::thread_rng(),
            hand_sequence: (0..15).collect(),
        }
    }

    pub fn reset(&mut self) -> usize {
        self.hand_sequence = (0..15).collect();
        self.hand_sequence.len()
    }

    pub fn available_hands(&self) -> Option<u16> {
        if self.hand_sequence.len() != 0 {
            Some(self.available_hands_to_code())
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn propose_hand(&mut self) -> Option<usize> {
        Some(*self.hand_sequence.choose(&mut self.rng)?)
    }

    pub fn pick_hand(&mut self) -> Option<usize> {
        let i = (0..self.hand_sequence.len()).choose(&mut self.rng)?;
        Some(self.hand_sequence.swap_remove(i))
    }

    fn available_hands_to_code(&self) -> u16 {
        self.hand_sequence
            .iter()
            .map(|s| u16::pow(2, *s as u32))
            .sum()
    }
}
