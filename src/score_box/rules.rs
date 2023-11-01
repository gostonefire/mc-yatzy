use crate::score_box::{OptimalHolds, Throw};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use rand::distributions::WeightedIndex;
use crate::dices::Throw::{First, Second};
use crate::EXPORT_DIR;
use crate::utils::{base10_to_base2, base10_to_base7, records_in_file, write_records_header};

pub struct Hand {
    optimal_holds: OptimalHolds,
    hand: HandType,
}

impl Hand {
    pub fn new(hand: HandType) -> Hand {
        Hand {
            optimal_holds: OptimalHolds::new(),
            hand,
        }
    }

    pub fn name(&self) -> String {
        self.hand.name().clone()
    }

    pub fn id(&self) -> usize {
        self.hand.id()
    }

    pub fn min_holds(&self) -> u8 {
        self.hand.min_holds()
    }

    pub fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    pub fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            First => Ok(&self.optimal_holds.first),
            Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    pub fn max_score_probability(&self, throw: Throw, thrown: u16) -> Result<f64, String> {
        if let Some((_, _, score)) = self.optimal_holds(throw)?
            .get(&thrown) {
            Ok(*score / self.hand.max_score() as f64)
        } else {
            Err(format!("Optimal holds for hand {} empty or not complete", self.hand.name()))
        }
    }

    pub fn score(&self, dices: &Vec<u8>) -> f32 {
        self.hand.score(dices)
    }

    pub fn load_optimal_holds(&mut self, path: &str) -> Result<(), String> {
        let name = self.hand.name();
        let opt_arr = self.optimal_holds_mut();

        let path_name = &format!("{}/hand.{}.bin", path, name);
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

    pub fn save_optimal_holds(&self, path: &str) -> Result<(), String> {
        let opt_vec = vec![self.optimal_holds(First)?, self.optimal_holds(Second)?];

        let path_name = &format!("{}/hand.{}.bin", path, self.hand.name());
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

    pub fn export_optimal_holds(&self, path: &str) -> Result<(), String> {
        let opt_arr = [self.optimal_holds(First)?, self.optimal_holds(Second)?];

        let path_name = &format!("{}/{}/hand.{}.txt", path, EXPORT_DIR, self.hand.name());
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

pub struct HandDistribution {
    distr: HashMap<u8, u64>,
    n_hits: u64,
    mean: f64,
    weights: (Vec<u8>, Vec<u64>),
    weighted_index: WeightedIndex<u64>,
    hand: HandType,
}

impl HandDistribution {
    pub fn new(hand: HandType) -> HandDistribution {
        HandDistribution {
            distr: HashMap::new(),
            n_hits: 0,
            mean: 0.0,
            weights: (Vec::new(), Vec::new()),
            weighted_index: WeightedIndex::new([1]).unwrap(),
            hand,
        }
    }

    pub fn name(&self) -> String {
        self.hand.name().clone()
    }

    pub fn update_scores(&mut self, score: u8) {
        match self.distr.get(&score) {
            Some(d) => {
                self.distr.insert(score, *d + 1);
            },
            None => {
                self.distr.insert(score, 1);
            }
        }
        self.n_hits += 1;
    }

    fn update_weighted_index(&mut self) {
        if self.distr.iter().map(|(_, &h)| h).sum::<u64>() > 0 {
            self.weights= self.distr.iter().unzip();
            self.weighted_index = WeightedIndex::new(&self.weights.1).unwrap();
        }
    }

    fn update_mean_score(&mut self) {
        if self.n_hits > 0 {
            let total_score = self.distr.iter().map(|(&s, &h)| s as f64 * h as f64).sum::<f64>();
            self.mean = total_score / self.n_hits as f64;
        }
    }

    pub fn load_distribution(&mut self, path: &str) -> Result<(), String> {
        let path_name = &format!("{}/distr.{}.bin", path, self.hand.name());
        let mut buf_reader = match File::open(path_name) {
            Ok(f) => BufReader::new(f),
            Err(e) => return Err(format!("Error while open file {}: {}", path_name, e)),
        };

        self.n_hits = 0;
        let mut buf = [0u8; 9];
        let mut n_records = records_in_file(&mut buf_reader, path_name)?;

        while n_records > 0 {
            match buf_reader.read_exact(&mut buf) {
                Ok(()) => {
                    let score = buf[0];
                    let hits = u64::from_le_bytes(buf[1..9].try_into().unwrap());

                    self.distr.insert(score, hits);
                    self.n_hits += hits;
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
        self.update_mean_score();
        self.update_weighted_index();

        Ok(())
    }

    pub fn save_distribution(&mut self, path: &str) -> Result<(), String> {
        let path_name = &format!("{}/distr.{}.bin", path, self.hand.name());
        let mut buf_writer = match File::create(path_name) {
            Ok(f) => BufWriter::new(f),
            Err(e) => return Err(format!("Error while open/create file {}: {}", path_name, e)),
        };

        let _ = write_records_header(&mut buf_writer, &vec![&self.distr], path_name)?;

        let mut buf = [0u8; 9];
        let mut offset: usize;
        for (score, hits) in &self.distr {
            offset = 1;
            buf[0] = *score;
            (*hits).to_le_bytes().iter().for_each(|v| {
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
        self.update_mean_score();
        self.update_weighted_index();

        Ok(())
    }

    pub fn export_distribution(&self, path: &str) -> Result<(), String> {
        let path_name = &format!("{}/{}/distr.{}.txt", path, EXPORT_DIR, self.hand.name());
        let mut buf_writer = match File::create(path_name) {
            Ok(f) => BufWriter::new(f),
            Err(e) => return Err(format!("Error while open/create file {}: {}", path_name, e)),
        };

        let mut keys: Vec<u8> = self.distr.keys().map(|k| *k).collect();
        keys.sort();

        for score in keys {
            let hits = self.distr.get(&score).unwrap();

            let row = format!("{:2} -> {:15} {}\n", score, *hits, *hits as f64 / self.n_hits as f64);
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

#[derive(Clone)]
pub enum HandType {
    Ones,
    Twos,
    Threes,
    Fours,
    Fives,
    Sixes,
    OnePair,
    TwoPairs,
    ThreeOfAKind,
    FourOfAKind,
    SmallStraight,
    LargeStraight,
    FullHouse,
    Chance,
    Yatzy,
}

impl HandType {
    fn name(&self) -> String {
        match self {
            HandType::Ones => String::from("ones"),
            HandType::Twos => String::from("twos"),
            HandType::Threes => String::from("threes"),
            HandType::Fours => String::from("fours"),
            HandType::Fives => String::from("fives"),
            HandType::Sixes => String::from("sixes"),
            HandType::OnePair => String::from("one_pair"),
            HandType::TwoPairs => String::from("two_pairs"),
            HandType::ThreeOfAKind => String::from("three_of_a_kind"),
            HandType::FourOfAKind => String::from("four_of_a_kind"),
            HandType::SmallStraight => String::from("small_straight"),
            HandType::LargeStraight => String::from("large_straight"),
            HandType::FullHouse => String::from("full_house"),
            HandType::Chance => String::from("chance"),
            HandType::Yatzy => String::from("yatzy"),
        }
    }
    pub fn id(&self) -> usize {
        match self {
            HandType::Ones => 0,
            HandType::Twos => 1,
            HandType::Threes => 2,
            HandType::Fours => 3,
            HandType::Fives => 4,
            HandType::Sixes => 5,
            HandType::OnePair => 6,
            HandType::TwoPairs => 7,
            HandType::ThreeOfAKind => 8,
            HandType::FourOfAKind => 9,
            HandType::SmallStraight => 10,
            HandType::LargeStraight => 11,
            HandType::FullHouse => 12,
            HandType::Chance => 13,
            HandType::Yatzy => 14,
        }
    }
    fn max_score(&self) -> u8 {
        match self {
            HandType::Ones => 5,
            HandType::Twos => 10,
            HandType::Threes => 15,
            HandType::Fours => 20,
            HandType::Fives => 25,
            HandType::Sixes => 30,
            HandType::OnePair => 12,
            HandType::TwoPairs => 22,
            HandType::ThreeOfAKind => 18,
            HandType::FourOfAKind => 24,
            HandType::SmallStraight => 15,
            HandType::LargeStraight => 20,
            HandType::FullHouse => 28,
            HandType::Chance => 30,
            HandType::Yatzy => 50,
        }
    }

    fn min_holds(&self) -> u8 {
        match self {
            HandType::Ones => 5,
            HandType::Twos => 5,
            HandType::Threes => 5,
            HandType::Fours => 5,
            HandType::Fives => 5,
            HandType::Sixes => 5,
            HandType::OnePair => 2,
            HandType::TwoPairs => 4,
            HandType::ThreeOfAKind => 3,
            HandType::FourOfAKind => 4,
            HandType::SmallStraight => 5,
            HandType::LargeStraight => 5,
            HandType::FullHouse => 5,
            HandType::Chance => 5,
            HandType::Yatzy => 5,
        }
    }
    pub fn score(&self, values: &Vec<u8>) -> f32 {
        match self {
            Self::Ones=> {
                let score = values
                    .into_iter()
                    .filter(|&&x| x == 1)
                    .count()
                    * 1;

                score as f32
            },
            Self::Twos=> {
                let score = values
                    .into_iter()
                    .filter(|&&x| x == 2)
                    .count()
                    * 2;

                score as f32
            },
            Self::Threes=> {
                let score = values
                    .into_iter()
                    .filter(|&&x| x == 3)
                    .count()
                    * 3;

                score as f32
            },
            Self::Fours=> {
                let score = values
                    .into_iter()
                    .filter(|&&x| x == 4)
                    .count()
                    * 4;

                score as f32
            },
            Self::Fives=> {
                let score = values
                    .into_iter()
                    .filter(|&&x| x == 5)
                    .count()
                    * 5;

                score as f32
            },
            Self::Sixes=> {
                let score = values
                    .into_iter()
                    .filter(|&&x| x == 6)
                    .count()
                    * 6;

                score as f32
            },
            Self::OnePair=> {
                let mut groups: [u8; 6] = [0; 6];
                for dice in values {
                    groups[*dice as usize - 1] += 1;
                }

                let mut pair: u8 = 0;
                for dice in (1..7).rev() {
                    if groups[dice - 1] > 1 {
                        pair = dice as u8;
                        break;
                    }
                }

                (pair * 2) as f32
            },
            Self::TwoPairs=> {
                let mut groups: [u8; 6] = [0; 6];
                for dice in values {
                    groups[*dice as usize - 1] += 1;
                }

                let mut pairs: [u8; 2] = [0; 2];
                let mut pair: usize = 0;
                for dice in (1..7).rev() {
                    if groups[dice - 1] > 1 {
                        pairs[pair] = dice as u8;
                        pair += 1;
                    }
                    if pair > 1 {
                        break;
                    }
                }

                let res = if pair < 2 {
                    0
                } else {
                    pairs[0] * 2 + pairs[1] * 2
                };

                res as f32
            },
            Self::ThreeOfAKind=> {
                let mut groups: [u8; 6] = [0; 6];
                for dice in values {
                    groups[*dice as usize - 1] += 1;
                }

                let mut triple: u8 = 0;
                for dice in (1..7).rev() {
                    if groups[dice - 1] > 2 {
                        triple = dice as u8;
                        break;
                    }
                }

                (triple * 3) as f32
            },
            Self::FourOfAKind=> {
                let mut groups: [u8; 6] = [0; 6];
                for dice in values {
                    groups[*dice as usize - 1] += 1;
                }

                let mut quad: u8 = 0;
                for dice in (1..7).rev() {
                    if groups[dice - 1] > 3 {
                        quad = dice as u8;
                        break;
                    }
                }

                (quad * 4) as f32
            },
            Self::SmallStraight=> {
                let score: f32 = match values.as_slice() {
                    [1, 2, 3, 4, 5] => 15.0,
                    _ => 0.0,
                };

                score
            },
            Self::LargeStraight=> {
                let score: f32 = match values.as_slice() {
                    [2, 3, 4, 5, 6] => 20.0,
                    _ => 0.0,
                };

                score
            },
            Self::FullHouse=> {
                let mut groups: [u8; 6] = [0; 6];
                for dice in values {
                    groups[*dice as usize - 1] += 1;
                }

                let mut triple: u8 = 0;
                let mut pair: u8 = 0;
                for dice in (1..7).rev() {
                    if groups[dice - 1] > 2 {
                        triple = dice as u8;
                    } else if groups[dice - 1] > 1 {
                        pair = dice as u8;
                    }
                    if triple > 0 && pair > 0 {
                        break;
                    }
                }

                let res = if triple > 0 && pair > 0 {
                    triple * 3 + pair * 2
                } else {
                    0
                };

                res as f32
            },
            Self::Chance=> {
                let score: u8 = values.iter().sum();

                score as f32
            },
            Self::Yatzy=> {
                let mut groups: [u8; 6] = [0; 6];
                for dice in values {
                    groups[*dice as usize - 1] += 1;
                }

                for dice in (1..7).rev() {
                    if groups[dice - 1] > 4 {
                        return 50.0;
                    }
                }

                0.0
            },
        }
    }
}

pub fn best_available_hand(throw: Throw, thrown: u16, available_hands: u16, hands: &Vec<Box<Hand>>) -> Result<usize, String> {
    let mut best_hand: Option<usize> = None;
    let mut max_prob: f64 = 0.0;
    let mut prob: f64;

    match throw {
        First => {
            for hand in base10_to_base2(available_hands, false) {
                prob = hands[hand as usize].max_score_probability(First, thrown)?;
                if prob > max_prob {
                    max_prob = prob;
                    best_hand = Some(hand as usize);
                }
            }
        },
        Second => {
            for hand in base10_to_base2(available_hands, false) {
                prob = hands[hand as usize].max_score_probability(Second, thrown)?;
                if prob > max_prob {
                    max_prob = prob;
                    best_hand = Some(hand as usize);
                }
            }
        },
        _ => return Err("Illegal throw at this point".to_string()),
    }

    if let Some(hand) = best_hand {Ok(hand)} else {Err("No best hand found".to_string())}
}
