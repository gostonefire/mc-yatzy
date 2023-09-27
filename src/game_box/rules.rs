use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use crate::utils::{base10_to_base2, base10_to_base7, records_in_file, sort_key, write_records_header};

pub struct GameRules {
    rules: HashMap<(u16, u16), (u8, f64)>,
    name: String,
}

impl GameRules {
    pub fn new() -> GameRules {
        GameRules {
            rules: HashMap::new(),
            name: "game".to_string(),
        }
    }

    pub fn propose_hand(&self, thrown: u16, available_hands: u16) -> Result<u8, String> {
        let hand = self.rules.get(&(thrown, available_hands));

        match hand {
            Some(h) => Ok(h.0),
            None => Err("Error, combination of thrown dices and available hands not in table".to_string()),
        }
    }

    pub fn optimal_games_mut(&mut self) -> &mut HashMap<(u16, u16), (u8, f64)> {
        &mut self.rules
    }

    pub fn load_optimal_games(&mut self, path: &str) -> Result<(), String> {
        let path_name = &format!("{}/{}.bin", path, self.name);
        let mut buf_reader = match File::open(path_name) {
            Ok(f) => BufReader::new(f),
            Err(e) => return Err(format!("Error while open file {}: {}", path_name, e)),
        };

        let mut buf = [0u8;13];
        let mut n_records = records_in_file(&mut buf_reader, path_name)?;

        while n_records > 0 {
            match buf_reader.read_exact(&mut buf) {
                Ok(()) => {
                    let hand = buf[0];
                    let thrown = u16::from_le_bytes(buf[1..3].try_into().unwrap());
                    let available_hands = u16::from_le_bytes(buf[3..5].try_into().unwrap());
                    let score = f64::from_le_bytes(buf[5..13].try_into().unwrap());

                    self.rules.insert((thrown, available_hands), (hand, score));
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

        let mut buf = [0u8;13];
        let mut offset: usize;

        for ((thrown, available_hands), (hand, score)) in &self.rules {
            offset = 1;
            buf[0] = *hand;
            thrown.to_le_bytes().iter().for_each(|v| {buf[offset] = *v; offset += 1;});
            available_hands.to_le_bytes().iter().for_each(|v| {buf[offset] = *v; offset += 1;});
            score.to_le_bytes().iter().for_each(|v| {buf[offset] = *v; offset += 1;});

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

        let mut keys: Vec<(u16, u16)> = self.rules.keys().map(|k| *k).collect::<Vec<(u16, u16)>>();
        keys.sort_by_key(|k| k.0 as u64 * 10000000 + sort_key(k.1));


        for (thrown, available_hands) in keys {
            let (hold, score) = self.rules.get(&(thrown, available_hands)).unwrap();
            let t_vec = base10_to_base7(thrown);
            let a_vec = base10_to_base2(available_hands, true);

            let row = format!("{:?} {:51} -> [{:2}] -> {}\n", t_vec, format!("{:?}", a_vec), *hold + 1, *score);
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