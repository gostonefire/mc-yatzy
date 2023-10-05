use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};
use chrono::Local;
use num_format::{Locale, ToFormattedString};
use rand::seq::SliceRandom;
use crate::distr_worker::load_hand_distributions;
use crate::score_box::rules::{Hand, HandDistribution};
use crate::utils::{base10_to_base2, base10_to_base7};
use rust_tree_map::{NodeData, NodeId};
use rust_tree_map::OpenMode::OpenCreate;
use rust_tree_map::multi_file_tree_map::MultiFileTreeMap;

type SplitterType = fn(u16) -> u8;

const LAP_BATCH: i64 = 100_000_000;

pub fn mc_play(path: &str, laps: i64) -> Result<(), String> {
    let mut remaining_laps = laps;
    let mut lap_offset = 0i64;

    loop {
        if remaining_laps > LAP_BATCH {
            run(path, LAP_BATCH, lap_offset)?;
        } else {
            run(path, remaining_laps, lap_offset)?;
        }
        remaining_laps -= LAP_BATCH;
        lap_offset += LAP_BATCH;

        if remaining_laps > 0 {
            let now = Local::now();
            println!("SSD 30 min cool of period started at {}", now.format("%T"));
            thread::sleep(Duration::from_secs(1800));
        } else {
            break;
        }
    }

    Ok(())
}

fn run(path: &str, laps: i64, lap_offset: i64) -> Result<(), String> {
    let mut rng = rand::thread_rng();
    let mut available_hands: u16;
    let mut mc_scores: HashMap<u8, u16> = HashMap::new();

    let mut hand_distr = load_hand_distributions(path, true).unwrap();
    let mut visited: Vec<NodeId> = Vec::new();

    println!("Opening game tree");
    let splitter: SplitterType = |k| (k >> 9) as u8;
    let mut tree = MultiFileTreeMap::new(path, 15, OpenCreate, splitter)
        .map_err(|e| e.to_string())?;
    println!("... {} nodes available", tree.len());

    let top = tree.get_top();
    let mut start = Instant::now();
    for lap in lap_offset..(lap_offset + laps) {
        // For each lap, start with a blank score card and on the top of the hash tree
        available_hands = 32767;
        let mut hand_vec = base10_to_base2(available_hands, false);

        let mut node = top;
        mc_scores.clear();
        visited.clear();
        visited.push(node);

        // Run through all 15 available hands in random order
        while available_hands > 0 {
            let hand = *hand_vec.choose(&mut rng).unwrap();
            let score = hand_distr[hand as usize].sample_from_distribution();

            mc_scores.insert(hand, score as u16);
            available_hands -= 2u16.pow(hand as u32);
            hand_vec = base10_to_base2(available_hands, false);

            let key = hand_score_to_key(hand as u16, score as u16);
            let child = tree.get_child(node, key).map_err(|e| e.to_string())?;
            match child {
                Some(c) => {
                    node = c.node_id;
                },
                None => {
                    let n_scores = hand_distr.iter()
                        .enumerate()
                        .filter(|(h, _)| hand_vec.contains(&(*h as u8)))
                        .map(|(_, h)| h.len())
                        .sum::<u32>();
                    let c = tree.add_child(node, key, 0, 0, n_scores)
                        .map_err(|e| {println!("{} {} {}", node, key, n_scores); e.to_string()})?;
                    node = c;
                }
            }
            visited.push(node);
        }

        // Calculate if we are eligible for the bonus of 50 points
        let bonus = if mc_scores
            .iter()
            .filter(|&(&h, _)| h < 6)
            .map(|(_, &s)| s)
            .sum::<u16>() >= 63 {50} else {0};
        let score = mc_scores.iter().map(|(_, &s)| s).sum::<u16>();

        // Back propagate the total score in the hash tree
        for node in &visited {
            tree.update_node_add(*node, 1, (bonus + score) as i64).map_err(|e| e.to_string())?;
        }

        if (lap + 1) % 1000000 == 0 {
            let elapsed = start.elapsed();
            println!("...processed {} laps in {} seconds", (lap + 1).to_formatted_string(&Locale::en), elapsed.as_secs());
            start = Instant::now();
        }
    }

    println!("Game strategy tree size: {} nodes", tree.len());
    Ok(())
}

pub fn best_available_game_hand_from_tree(thrown: u16, available_hands: u16, used_hands: &Vec<u16>, tree: &mut MultiFileTreeMap<SplitterType>, hands: &Vec<Box<Hand>>, distr: &Vec<Box<HandDistribution>>, bonus_extra: f64) -> Result<usize, String> {
    let mut best_hand: Option<usize> = None;
    let mut child: Option<NodeData>;
    let mut max_score: f64 = f64::MIN;
    let mut score: f64;

    // First try using strategy tree, which may not have every possible combination
    let mut node = Some(tree.get_top());
    for used_hand in used_hands {
        match tree.get_child(node.unwrap(), *used_hand).map_err(|e| e.to_string())? {
            Some(n) => { node = Some(n.node_id); },
            None => { node = None; break; },
        }
    }

    if let Some(parent) = node {
        for hand in base10_to_base2(available_hands, false) {
            score = hands[hand as usize].score(base10_to_base7(thrown));
            child = tree.get_child(parent, hand_score_to_key(hand as u16, score as u16))
                .map_err(|e| e.to_string())?;

            if let Some(c) = child {
                //let elem = tree.get_elem(c)?;
                score = if c.hits > 0 {c.score as f64 / c.hits as f64 } else {0.0};
                if score > max_score {
                    max_score = score;
                    best_hand = Some(hand as usize);
                }
            } else {
                println!("...reverting to distribution statistics");
                return best_available_game_hand(thrown, available_hands, hands, distr, bonus_extra);
            }
        }
    } else {
        println!("...reverting to distribution statistics");
        return best_available_game_hand(thrown, available_hands, hands, distr, bonus_extra)
    }

    if let Some(hand) = best_hand {Ok(hand)} else {Err("No best hand found".to_string())}
}

pub fn best_available_game_hand(thrown: u16, available_hands: u16, hands: &Vec<Box<Hand>>, distr: &Vec<Box<HandDistribution>>, bonus_extra: f64) -> Result<usize, String> {
    let mut best_hand: Option<usize> = None;
    let mut expected_score: f64;
    let mut max_gain: f64 = f64::MIN;
    let mut score: f64;
    let mut gain: f64;
    let mut extras: Option<f64>;

    let bonus_addition = bonus_extra / 21.0;

    // Try to find hand which gives the best score compared to average score
    for hand in base10_to_base2(available_hands, false) {
        score = hands[hand as usize].score(base10_to_base7(thrown));
        extras = if hand <= 5 {Some(bonus_addition * (hand + 1) as f64)} else {None};
        expected_score = distr[hand as usize].mean_score(extras)?;
        gain = score - expected_score;

        if gain > max_gain {
            max_gain = gain;
            best_hand = Some(hand as usize);
        }
    }

    if let Some(hand) = best_hand {Ok(hand)} else {Err("No best hand found".to_string())}
}

pub fn hand_score_to_key(hand: u16, score: u16) -> u16 {
    (hand << 9) + score
}

#[allow(dead_code)]
pub fn key_to_hand_score(key: u16) -> (u16, u16) {
    let score = key & 0b111111111;
    let hand = key >> 9;
    (hand, score)
}
