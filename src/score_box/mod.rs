pub mod rules;

use crate::dices::Throw;
use crate::dices::Throw::{First, Second};
use crate::utils::{base10_to_base7, records_in_file, write_records_header};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

pub struct OptimalHolds {
    first: HashMap<u16, (u8, u16, f64)>,
    second: HashMap<u16, (u8, u16, f64)>,
}

impl OptimalHolds {
    pub fn new() -> OptimalHolds {
        OptimalHolds {
            first: HashMap::new(),
            second: HashMap::new(),
        }
    }
}

pub struct MCHands {
    first: HashMap<(u16, u16), (u8, f64, f64)>,
    second: HashMap<(u16, u16), (u8, f64, f64)>,
    name: String,
}

impl MCHands {
    pub fn new() -> MCHands {
        MCHands {
            first: HashMap::new(),
            second: HashMap::new(),
            name: "mchand".to_string(),
        }
    }

    pub fn update_optimal_holds(&self, opt_holds: [&mut HashMap<u16, (u8, u16, f64)>; 2], min_holds: u8) {
        let mc = [&self.first, &self.second];

        for throw in 0..2 {
            opt_holds[throw].clear();

            for ((thrown, hold), (mc_s_len, hits, value)) in mc[throw].iter() {
                if *mc_s_len <= min_holds {
                    match opt_holds[throw].get(thrown) {
                        Some((_, _, score)) => {
                            if *value / *hits > *score
                            {
                                opt_holds[throw].insert(*thrown, (*mc_s_len, *hold, *value / *hits));
                            }
                        }
                        None => {
                            opt_holds[throw].insert(*thrown, (*mc_s_len, *hold, *value / *hits));
                        }
                    }
                }
            }
        }
    }

    pub fn update_scores(&mut self, throw: Throw, t_code: u16, s_code: u16, s_len: u8, score: f64) {
        let mc: &mut HashMap<(u16, u16), (u8, f64, f64)>;
        match throw {
            First => {
                mc = &mut self.first;
            }
            Second => {
                mc = &mut self.second;
            }
            _ => return,
        }

        match mc.get(&(t_code, s_code)) {
            Some((_, hits, value)) => {
                mc.insert((t_code, s_code), (s_len, *hits + 1.0, *value + score));
            }
            None => {
                mc.insert((t_code, s_code), (s_len, 1.0, score));
            }
        }
    }

    pub fn debug_scores(&self, path: &str, name: &str) -> Result<(), String> {
        let score_arr = [&self.first, &self.second];

        let path_name = &format!("{}/debug.{}.{}.txt", path, self.name, name);
        let mut buf_writer = match File::create(path_name) {
            Ok(f) => BufWriter::new(f),
            Err(e) => return Err(format!("Error while open/create file {}: {}", path_name, e)),
        };

        for throw in 0..score_arr.len() {
            let mut keys: Vec<(u16, u16)> = score_arr[throw]
                .keys()
                .map(|k| *k)
                .collect::<Vec<(u16, u16)>>();
            keys.sort_by_key(|k| k.0 as u64 * 100000 + k.1 as u64);

            // HashMap<(u16, u16), (u8, f64, f64)>
            for (thrown, hold) in keys {
                let (hold_len, hits, score) = score_arr[throw].get(&(thrown, hold)).unwrap();
                let t_vec = base10_to_base7(thrown);
                let s_vec = base10_to_base7(hold);

                let row = format!(
                    "{}: {:?} {:15} {} -> {:10} - {:10} - {}\n",
                    throw + 1,
                    t_vec,
                    format!("{:?}", s_vec),
                    *hold_len,
                    *hits,
                    *score,
                    *score / *hits
                );
                if let Err(e) = buf_writer.write_all(row.as_bytes()) {
                    return Err(format!("Error while writing to file {}: {}", path_name, e));
                }
            }
        }
        if let Err(e) = buf_writer.flush() {
            return Err(format!("Error while writing to file {}: {}", path_name, e));
        }
        Ok(())
    }
}

pub trait Hands {
    fn name(&self) -> String;
    fn id(&self) -> usize;
    fn min_holds(&self) -> u8;
    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>; 2];
    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String>;
    fn score(&self, dices: Vec<u8>) -> f64;

    fn load_optimal_holds(&mut self, path: &str) -> Result<(), String> {
        let name = self.name();
        let opt_arr = self.optimal_holds_mut();

        let path_name = &format!("{}/{}.bin", path, name);
        let mut buf_reader = match File::open(path_name) {
            Ok(f) => BufReader::new(f),
            Err(e) => return Err(format!("Error while open file {}: {}", path_name, e)),
        };

        let mut buf = [0u8; 14];
        let mut n_records = records_in_file(&mut buf_reader, path_name)?;

        while n_records > 0 {
            match buf_reader.read_exact(&mut buf) {
                Ok(()) => {
                    let throw = buf[0] as usize;
                    let hold_len = buf[1];
                    let thrown = u16::from_le_bytes(buf[2..4].try_into().unwrap());
                    let hold = u16::from_le_bytes(buf[4..6].try_into().unwrap());
                    let score = f64::from_le_bytes(buf[6..14].try_into().unwrap());

                    opt_arr[throw].insert(thrown, (hold_len, hold, score));
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
        Ok(())
    }

    fn save_optimal_holds(&self, path: &str) -> Result<(), String> {
        let opt_vec = vec![self.optimal_holds(First)?, self.optimal_holds(Second)?];

        let path_name = &format!("{}/{}.bin", path, self.name());
        let mut buf_writer = match File::create(path_name) {
            Ok(f) => BufWriter::new(f),
            Err(e) => return Err(format!("Error while open/create file {}: {}", path_name, e)),
        };

        let _ = write_records_header(&mut buf_writer, &opt_vec, path_name)?;

        let mut buf = [0u8; 14];
        let mut offset: usize;
        for throw in 0..opt_vec.len() {
            for (thrown, (hold_len, hold, score)) in opt_vec[throw] {
                offset = 2;
                buf[0] = throw as u8;
                buf[1] = *hold_len;
                (*thrown).to_le_bytes().iter().for_each(|v| {
                    buf[offset] = *v;
                    offset += 1;
                });
                (*hold).to_le_bytes().iter().for_each(|v| {
                    buf[offset] = *v;
                    offset += 1;
                });
                (*score).to_le_bytes().iter().for_each(|v| {
                    buf[offset] = *v;
                    offset += 1;
                });

                if let Err(e) = buf_writer.write_all(&buf) {
                    return Err(format!("Error while writing to file {}: {}", path_name, e));
                }
            }
        }
        if let Err(e) = buf_writer.flush() {
            return Err(format!("Error while writing to file {}: {}", path_name, e));
        }
        Ok(())
    }

    fn export_optimal_holds(&self, path: &str) -> Result<(), String> {
        let opt_arr = [self.optimal_holds(First)?, self.optimal_holds(Second)?];

        let path_name = &format!("{}/export.{}.txt", path, self.name());
        let mut buf_writer = match File::create(path_name) {
            Ok(f) => BufWriter::new(f),
            Err(e) => return Err(format!("Error while open/create file {}: {}", path_name, e)),
        };

        for throw in 0..opt_arr.len() {
            let mut keys: Vec<u16> = opt_arr[throw].keys().map(|k| *k).collect();
            keys.sort();

            for thrown in keys {
                let (_, hold, score) = opt_arr[throw].get(&thrown).unwrap();
                let tv = base10_to_base7(thrown);
                let hv = base10_to_base7(*hold);

                let row = format!(
                    "{}: {:?} -> {:15} -> {}\n",
                    throw + 1,
                    tv,
                    format!("{:?}", hv),
                    *score
                );
                if let Err(e) = buf_writer.write_all(row.as_bytes()) {
                    return Err(format!("Error while writing to file {}: {}", path_name, e));
                }
            }
        }
        if let Err(e) = buf_writer.flush() {
            return Err(format!("Error while writing to file {}: {}", path_name, e));
        }
        Ok(())
    }
}
