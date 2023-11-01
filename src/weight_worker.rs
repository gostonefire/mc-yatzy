use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Write};
use std::sync::mpsc::{channel, Sender};
use std::time::{Duration, Instant};
use chrono::Local;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::ThreadPoolBuilder;
use crate::score_box::rules::{Hand};
use crate::utils::{available_threads, base10_to_base2, base3_to_base10, factor, records_in_file};
use crate::dices::Dices;
use crate::EXPORT_DIR;
use crate::hand_worker::load_hands;
use crate::play_worker::throw_hand;

pub struct RunResult {
    total_score: u32,
    total_bonus: u32,
    avg_score: f32,
    pub weights: [f32;15],
    laps: u32,
    used_bonus: u32,
    generation: u32,
}

impl RunResult {
    fn new() -> RunResult {
        RunResult {
            total_score: 0,
            total_bonus: 0,
            avg_score: 0.0,
            weights: [0.5f32;15],
            laps: 0,
            used_bonus: 0,
            generation: 0,
        }
    }
    fn from(total_score: u32, total_bonus: u32, laps: u32, used_bonus: u32, weights: [f32;15]) -> RunResult {
        RunResult {
            total_score,
            total_bonus,
            avg_score: (total_score + total_bonus) as f32 / laps as f32,
            weights,
            laps,
            used_bonus,
            generation: 0,
        }
    }
    fn true_avg_score(&self) -> f32 {
        if self.laps > 0 {
            (self.total_score + self.total_bonus / self.used_bonus * ACTUAL_BONUS) as f32 / self.laps as f32
        } else {
            0.0
        }
    }
}

const ACTUAL_BONUS: u32 = 50u32;
const TUNING_LAPS: u32 = 1000000;

pub fn strategy_learn(path: &str, laps: Vec<i64>, use_bonus: Option<u32>) -> Result<(), String> {
    ThreadPoolBuilder::new().num_threads(available_threads() - 1).build_global().unwrap();
    let bonus = use_bonus.map_or(ACTUAL_BONUS, |b| b);

    let (mut generation, mut res_vec) = load_weights(path, Some(bonus))?
        .map_or((0u32, Vec::from([RunResult::new()])),|r| r);
    println!("Loaded {} rows of weights, best average score: {:5.2}",
             res_vec.len(), res_vec[0].true_avg_score());

    for lap in 0..laps[0] {
        println!("Starting lap {} at {}", lap + 1, Local::now().format("%T"));
        let weights = res_vec[0].weights;
        generation += 1;

        let base = base3_to_base10(&Vec::from([2u8;15])) + 1;
        let factor = factor(base)
            .into_iter().filter(|&v| v >= 15)
            .next()
            .map_or(Err(String::from("no factors")), |f| Ok(f))?;
        let batch = base / factor;

        println!("Running {} batches of size {}", factor, batch);
        let (sender, receiver) = channel::<RunResult>();
        (0..factor).into_par_iter().for_each_with(sender, |s, f| {
            super_run(path, laps[1] as u32, f, batch, weights, bonus, s);
        });

        for mut res in receiver {
            res.generation = generation;
            res_vec.push(res);
        }

        res_vec.retain(|r| r.laps > 0);
        res_vec.sort_by(|a, b| a.avg_score.total_cmp(&b.avg_score));
        res_vec.reverse();
        res_vec.truncate(1000);
        res_vec.iter_mut().for_each(|r| r.weights = trim_weights(r.weights));

        println!("Best average score: {:5.2}", res_vec[0].true_avg_score());
        println!("Worst average score: {:5.2}", res_vec[res_vec.len() - 1].true_avg_score());
        println!("lap {} ended at {}", lap + 1, Local::now().format("%T"));

        save_weights(path, Some(bonus), generation, &res_vec)?;
    }

    Ok(())
}

fn super_run(path: &str, sub_laps: u32, factor: u32, batch: u32, weights: [f32;15], bonus: u32, sender: &mut Sender<RunResult>) {

    let mut dices = Dices::new();
    let mut best_results = RunResult::new();

    let res = load_hands(path, true);

    match res {
        Ok(hands) => {
            let mut tuned_weights = [0f32;15];
            let start = factor * batch;
            let end = start + batch;
            println!("Starting factor batch {:8} to {:8}", start, end);
            let begin = Instant::now();
            for f in start..end {
                let mut tuning = [0f32;15];
                base10_to_tuning(f, &mut tuning);

                weights.iter().enumerate().for_each(|(i, &w)| tuned_weights[i] = tuning[i] + w);

                match run(sub_laps, &mut dices, &hands, tuned_weights, bonus) {
                    Ok(rr) => {
                        if rr.avg_score > best_results.avg_score {
                            best_results =rr;
                        }
                    },
                    Err(e) => {
                        println!("...error in super batch {:05}: {}", batch, e);
                        return;
                    }
                }
            }

            // Tune result to ensure the super run result isn't an outlier
            match run(TUNING_LAPS, &mut dices, &hands, best_results.weights, bonus) {
                Ok(rr) => {
                    sender.send(rr).unwrap();
                },
                Err(e) => {
                    println!("...error in super batch {:05}: {}", batch, e);
                    return;
                }
            }

            let done = begin.elapsed();
            println!("...super batch {:05} done in {}", batch, done.format());
        },
        Err(e) => {
            println!("...error in super batch {:05}: {}", batch, e);
        }
    }


}

fn run(laps: u32, dices: &mut Dices, hands: &Vec<Box<Hand>>, weights: [f32;15], bonus: u32) -> Result<RunResult, String> {
    let mut available_hands: u16;
    let mut mc_scores: HashMap<u8, u16> = HashMap::new();
    let mut total_score = 0u32;
    let mut total_bonus = 0u32;

    for _ in 0..laps {
        // For each lap, start with a blank score card and on the top of the hash tree
        available_hands = 32767;
        mc_scores.clear();

        // Run through all 15 available hands in random order
        while available_hands > 0 {
            let thrown = throw_hand(dices, available_hands, &hands)?;
            let hand_score = best_available_game_hand(&thrown, available_hands, &hands, weights)?;

            mc_scores.insert(hand_score.0, hand_score.1);
            available_hands -= 2u16.pow(hand_score.0 as u32);
        }

        // Calculate if we are eligible for the bonus of 50 points
        let bonus = if mc_scores
            .iter()
            .filter(|&(&h, _)| h < 6)
            .map(|(_, &s)| s)
            .sum::<u16>() >= 63 { bonus } else {0u32};
        let score = mc_scores.iter().map(|(_, &s)| s).sum::<u16>() as u32;
        total_score += score;
        total_bonus += bonus;
    }

    Ok(RunResult::from(total_score, total_bonus, laps, bonus, weights))
}

pub fn best_available_game_hand(thrown: &Vec<u8>, available_hands: u16, hands: &Vec<Box<Hand>>, weights: [f32;15]) -> Result<(u8, u16), String> {
    let mut best_hand_score: Option<(u8, u16)> = None;
    let mut weighted_score: f32;
    let mut max_weighted_score: f32 = f32::MIN;
    let mut score: f32;

    // Try to find hand which gives the best weighted score
    for hand in base10_to_base2(available_hands, false) {
        score = hands[hand as usize].score(thrown);
        weighted_score = score * weights[hand as usize];

        if weighted_score > max_weighted_score {
            max_weighted_score = weighted_score;
            best_hand_score = Some((hand, score as u16));
        }
    }

    if let Some(hand_score) = best_hand_score {
        Ok(hand_score)
    } else {
        Err("No best hand found".to_string())
    }
}

#[allow(dead_code)]
pub fn key_to_hand_score(key: u16) -> (u16, u16) {
    let score = key & 0b111111111;
    let hand = key >> 9;
    (hand, score)
}

fn save_weights<A>(path: &str, suffix: Option<A>, generation: u32, weights_score: &Vec<RunResult>) -> Result<(), String>
where A: Display
{
    let sfx = suffix.map_or(String::new(), |s| format!(".{}", s));
    let path_name = &format!("{}/weights{}.bin", path, sfx);
    let mut buf_writer = match File::create(path_name) {
        Ok(f) => BufWriter::new(f),
        Err(e) => return Err(format!("Error while open/create file {}: {}", path_name, e)),
    };

    let len = weights_score.len() as u64;
    let buf: [u8;8] = len.to_le_bytes();
    buf_writer.write_all(&buf).map_err(|e| {
        format!("Error while writing to weights file: {}", e)
    })?;

    let buf: [u8;4] = generation.to_le_bytes();
    buf_writer.write_all(&buf).map_err(|e| {
        format!("Error while writing to weights file: {}", e)
    })?;

    let mut buf = [0u8;80];
    for i in 0..len as usize {
        let mut offset = 0;
        weights_score[i].total_score.to_le_bytes().iter().for_each(|v| {
            buf[offset] = *v;
            offset += 1;
        });
        weights_score[i].total_bonus.to_le_bytes().iter().for_each(|v| {
            buf[offset] = *v;
            offset += 1;
        });
        weights_score[i].laps.to_le_bytes().iter().for_each(|v| {
            buf[offset] = *v;
            offset += 1;
        });
        weights_score[i].used_bonus.to_le_bytes().iter().for_each(|v| {
            buf[offset] = *v;
            offset += 1;
        });
        weights_score[i].generation.to_le_bytes().iter().for_each(|v| {
            buf[offset] = *v;
            offset += 1;
        });

        for weight in weights_score[i].weights {
            weight.to_le_bytes().iter().for_each(|v| {
                buf[offset] = *v;
                offset += 1;
            });
        };

        buf_writer.write_all(&buf).map_err(|e| {
            format!("Error while writing to weights file: {}", e)
        })?;
    }

    buf_writer.flush().unwrap();

    Ok(())
}

pub fn load_weights<A>(path: &str, suffix: Option<A>) -> Result<Option<(u32, Vec<RunResult>)>, String>
where A: Display
{
    let sfx = suffix.map_or(String::new(), |s| format!(".{}", s));
    let path_name = &format!("{}/weights{}.bin", path, sfx);
    let mut buf_reader = match File::open(path_name) {
        Ok(f) => BufReader::new(f),
        Err(e) => {
            return if e.kind().eq(&ErrorKind::NotFound) {
                Ok(None)
            } else {
                Err(format!("Error while open file {}: {}", path_name, e))
            }
        },
    };


    let mut n_records = records_in_file(&mut buf_reader, path_name)?;

    let mut buf = [0u8;4];
    buf_reader.read_exact(&mut buf).map_err(|e| {
        format!("Error while reading from weights file: {}", e)
    })?;
    let generation = u32::from_le_bytes(buf[0..4].try_into().unwrap());

    let mut buf = [0u8; 80];
    let mut res_vec: Vec<RunResult> = Vec::new();

    while n_records > 0 {
        buf_reader.read_exact(&mut buf).map_err(|e| {
            format!("Error while reading from weights file: {}", e)
        })?;

        let total_score = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let total_bonus = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        let laps = u32::from_le_bytes(buf[8..12].try_into().unwrap());
        let used_bonus = u32::from_le_bytes(buf[12..16].try_into().unwrap());
        let generation = u32::from_le_bytes(buf[16..20].try_into().unwrap());

        let mut offset: usize = 20;
        let mut weights = [0f32;15];
        for o in 0..15usize {
            weights[o] = f32::from_le_bytes(buf[offset..offset+4].try_into().unwrap());
            offset += 4;
        }
        let mut rr = RunResult::from(total_score, total_bonus, laps, used_bonus, weights);
        rr.generation = generation;
        res_vec.push(rr);

        n_records -= 1;
    }

    Ok(Some((generation, res_vec)))
}

pub fn export_weights<A>(path: &str, suffix: Option<A>, generation: u32, weights: &Vec<RunResult>) -> Result<(), String>
where A: Display
{
    let sfx = suffix.map_or(String::new(), |s| format!(".{}", s));
    let path_name = &format!("{}/{}/weights{}.txt", path, EXPORT_DIR, sfx);
    let mut buf_writer = match File::create(path_name) {
        Ok(f) => BufWriter::new(f),
        Err(e) => return Err(format!("Error while open/create file {}: {}", path_name, e)),
    };

    write!(buf_writer, "Generation: {}\n", generation).map_err(|e| e.to_string())?;
    write!(buf_writer, "avg_score  true_avg  tot_score  tot_bonus  used_bonus  laps     gen  weights\n")
        .map_err(|e| e.to_string())?;

    for weight in weights {
        write!(buf_writer, "{:0<6.3}    {:0<6.3}  {:7}   {:7}     {:3}         {:5} {:5}    [",
               weight.avg_score,
               weight.true_avg_score(), weight.total_score, weight.total_bonus,
               weight.used_bonus, weight.laps, weight.generation)
            .map_err(|e| e.to_string())?;
        let mut buf: Vec<String> = Vec::new();
        for w in weight.weights {
            buf.push(format!("{:0<2.1}", w));
        }
        write!(buf_writer, "{}]\n", buf.join(", ").to_string()).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn trim_weights(weights: [f32;15]) -> [f32;15] {
    let mut res = [0f32;15];
    weights.iter().enumerate().for_each(|(i, &w)| res[i] = (w * 10.0).round() / 10.0);

    res
}

fn base10_to_tuning(b10: u32, into: &mut [f32;15]) {
    let mut d = b10 / 3;
    let mut r = b10 % 3;

    let mut i = 0usize;
    while d > 0 || r > 0 {
        into[i] = if r == 0 {0.0} else if r == 1 {0.1} else {-0.1};
        r = d % 3;
        d /= 3;
        i += 1;
    }

    into.reverse();
}

trait DurationDisplay {
    fn format(&self) -> String;
}

impl DurationDisplay for Duration {
    fn format(&self) -> String {
        let a = self.as_secs();

        let d = a / 86400;
        let dr = a % 86400;
        let h = dr / 3600;
        let hr = dr % 3600;
        let m = hr / 60;
        let s = hr % 60;

        format!("D:{:02} H:{:02} M:{:02} S:{:02}", d, h, m, s)
    }
}