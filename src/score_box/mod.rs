pub mod rules;

use crate::dices::Throw;
use crate::dices::Throw::{First, Second};
use crate::utils::base10_to_base7;
use std::collections::HashMap;
use std::format;
use std::fs::File;
use std::io::{BufWriter, Write};
use crate::DEBUG_DIR;

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

        let path_name = &format!("{}/{}/{}.{}.txt", path, DEBUG_DIR, self.name, name);
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
