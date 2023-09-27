pub mod rules;

use crate::utils::{records_in_file, write_records_header};
use rand::rngs::ThreadRng;
use rand::seq::{IteratorRandom, SliceRandom};
use std::collections::HashMap;
use std::fs::File;
use std::io::ErrorKind::NotFound;
use std::io::{BufReader, BufWriter, Read, Write};

#[derive(Clone)]
pub struct IntermediateResult {
    pub available_hands: u16,
    pub thrown: u16,
    pub score: f64,
}

impl IntermediateResult {
    pub fn new() -> IntermediateResult {
        IntermediateResult {
            available_hands: 0,
            thrown: 0,
            score: 0.0,
        }
    }
}

pub struct MCScore {
    // (thrown dices, available hands, chosen hand), (hits, score)
    mc_scores: HashMap<(u16, u16, u8), (f64, f64)>,
    name: String,
}

impl MCScore {
    pub fn new() -> MCScore {
        MCScore {
            mc_scores: HashMap::new(),
            name: "mcscore".to_string(),
        }
    }

    pub fn update_scores(&mut self, intermediate_results: &Vec<IntermediateResult>) {
        let bonus = if intermediate_results[0..6]
            .iter()
            .map(|i| i.score)
            .sum::<f64>()
            < 63.0
        {
            0.0
        } else {
            50.0
        };
        let total_score = intermediate_results.iter().map(|i| i.score).sum::<f64>() + bonus;

        for (i, hand) in intermediate_results.iter().enumerate() {
            match self
                .mc_scores
                .get(&(hand.thrown, hand.available_hands, i as u8))
            {
                Some((hits, value)) => {
                    self.mc_scores.insert(
                        (hand.thrown, hand.available_hands, i as u8),
                        (*hits + 1.0, *value + total_score),
                    );
                }
                None => {
                    self.mc_scores.insert(
                        (hand.thrown, hand.available_hands, i as u8),
                        (1.0, total_score),
                    );
                }
            }
        }
    }

    pub fn update_optimal_game(&self, optimal_game: &mut HashMap<(u16, u16), (u8, f64)>) {
        optimal_game.clear();

        for ((thrown, available_hands, hand), (hits, value)) in self.mc_scores.iter() {
            match optimal_game.get(&(*thrown, *available_hands)) {
                Some((_, score)) => {
                    if *value / *hits > *score {
                        optimal_game
                            .insert((*thrown, *available_hands), (*hand, *value / *hits));
                    }
                }
                None => {
                    optimal_game
                        .insert((*thrown, *available_hands), (*hand, *value / *hits));
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

        let mut buf = [0u8; 21];
        let mut offset: usize;

        for ((thrown, available_hands, hand), (hits, value)) in &self.mc_scores {
            offset = 1;
            buf[0] = *hand;
            thrown.to_le_bytes().iter().for_each(|v| {
                buf[offset] = *v;
                offset += 1;
            });
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

        let mut buf = [0u8; 21];
        let mut n_records = records_in_file(&mut buf_reader, path_name)?;

        while n_records > 0 {
            match buf_reader.read_exact(&mut buf) {
                Ok(()) => {
                    let hand = buf[0];
                    let thrown = u16::from_le_bytes(buf[1..3].try_into().unwrap());
                    let available_hands = u16::from_le_bytes(buf[3..5].try_into().unwrap());
                    let hits = f64::from_le_bytes(buf[5..13].try_into().unwrap());
                    let score = f64::from_le_bytes(buf[13..21].try_into().unwrap());

                    self.mc_scores.insert((thrown, available_hands, hand), (hits, score));
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
        ) = (0.0f64, f64::MAX, 0.0f64, f64::MAX, 0.0f64, f64::MAX);

        let mut count: usize = 0;
        let mut score_avg: f64;
        let mut buckets = [0u64;20];
        for (hits, score) in self.mc_scores.values() {
            max_hits = max_hits.max(*hits);
            min_hits = min_hits.min(*hits);
            max_score_sum = max_score_sum.max(*score);
            min_score_sum = min_score_sum.min(*score);
            score_avg = *score / *hits;
            max_score = max_score.max(score_avg);
            min_score = min_score.min(score_avg);
            if *hits < 20.0 {
                buckets[*hits as usize] += 1;
            }
            count += 1;
        }

        buckets[0] = 61931520 - count as u64;
        let result = format!(
            "{} records, Hits - Min: {} Max: {}, Score sum - Min {} Max: {}, Score - Min {:.2} Max: {:.2}\nHit buckets: {:?}",
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
