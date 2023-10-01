use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use crate::utils::{base10_to_base2, records_in_file, sort_key, write_records_header};

pub struct GameRules {
    // (score, hand, available hands) (expected score)
    rules: HashMap<(u8, u8, u16), f64>,
    name: String,
}

impl GameRules {
    pub fn new() -> GameRules {
        GameRules {
            rules: HashMap::new(),
            name: "game".to_string(),
        }
    }

    pub fn expected_total_score(&self, score: u8, hand: u8, available_hands: u16) -> Result<f64, String> {
        let expected_score = self.rules.get(&(score, hand, available_hands));

        match expected_score {
            Some(h) => Ok(*h),
            None => Err("Error, combination of score, hand and available hands not in table".to_string()),
        }
    }

    pub fn optimal_games_mut(&mut self) -> &mut HashMap<(u8, u8, u16), f64> {
        &mut self.rules
    }

    pub fn load_optimal_games(&mut self, path: &str) -> Result<(), String> {
        let path_name = &format!("{}/{}.bin", path, self.name);
        let mut buf_reader = match File::open(path_name) {
            Ok(f) => BufReader::new(f),
            Err(e) => return Err(format!("Error while open file {}: {}", path_name, e)),
        };

        let mut buf = [0u8;12];
        let mut n_records = records_in_file(&mut buf_reader, path_name)?;

        while n_records > 0 {
            match buf_reader.read_exact(&mut buf) {
                Ok(()) => {
                    let score = buf[0];
                    let hand = buf[1];
                    let available_hands = u16::from_le_bytes(buf[2..4].try_into().unwrap());
                    let expected_score = f64::from_le_bytes(buf[4..12].try_into().unwrap());

                    self.rules.insert((score, hand, available_hands), expected_score);
                }
                Err(e) => {
                    return Err(format!("Error while reading from file {}: {}", path_name, e));
                }
            }
            n_records -= 1;
        }
        Ok(())
    }

    pub fn save_optimal_games(&self, path: &str) -> Result<(), String> {
        let path_name = &format!("{}/{}.bin", path, self.name);
        let mut buf_writer = match File::create(path_name) {
            Ok(f) => BufWriter::new(f),
            Err(e) => return Err(format!("Error while open/create file {}: {}", path_name, e)),
        };

        let _ = write_records_header(&mut buf_writer, &vec![&self.rules], path_name)?;

        let mut buf = [0u8;12];
        let mut offset: usize;

        for ((score, hand, available_hands), expected_score) in &self.rules {
            offset = 2;
            buf[0] = *score;
            buf[1] = *hand;
            available_hands.to_le_bytes().iter().for_each(|v| {buf[offset] = *v; offset += 1;});
            expected_score.to_le_bytes().iter().for_each(|v| {buf[offset] = *v; offset += 1;});

            if let Err(e) = buf_writer.write_all(&buf) {
                return Err(format!("Error while writing to file {}: {}", path_name, e));
            }
        }

        if let Err(e) = buf_writer.flush() {
            return Err(format!("Error while writing to file {}: {}", path_name, e));
        }
        Ok(())
    }

    pub fn export_optimal_games(&self, path: &str) -> Result<(), String> {
        let path_name = &format!("{}/export.{}.txt", path, self.name);
        let mut buf_writer = match File::create(path_name) {
            Ok(f) => BufWriter::new(f),
            Err(e) => return Err(format!("Error while open/create file {}: {}", path_name, e)),
        };

        let mut keys: Vec<(u8, u8, u16)> = self.rules.keys().map(|k| *k).collect::<Vec<(u8, u8, u16)>>();
        keys.sort_by_key(|k| (k.0 as u64 * 1000 + k.1 as u64) * 10000000 + sort_key(k.2));


        for (score, hand, available_hands) in keys {
            let expected_score = self.rules.get(&(score, hand, available_hands)).unwrap();
            let a_vec = base10_to_base2(available_hands, true);

            let row = format!("{:2} [{:2}] {:51} -> {}\n", score, hand + 1, format!("{:?}", a_vec), *expected_score);
            if let Err(e) = buf_writer.write_all(row.as_bytes()) {
                return Err(format!("Error while writing to file {}: {}", path_name, e));
            }
        }

        if let Err(e) = buf_writer.flush() {
            return Err(format!("Error while writing to file {}: {}", path_name, e));
        }
        Ok(())
    }
}
