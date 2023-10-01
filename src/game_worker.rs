use crate::dices::{Dices, Throw};
use crate::game_box::rules::GameRules;
use crate::game_box::{IntermediateResult, MCGame, MCScore};
use crate::hand_worker::load_hands;
use crate::utils::{available_threads, base10_to_base7, base7_to_base10};
use num_format::{Locale, ToFormattedString};
use std::sync::{Arc, Mutex};
use rayon::prelude::IntoParallelIterator;
use crate::score_box::rules::best_available_hand;
use rayon::iter::ParallelIterator;


pub fn learn_game(mut laps: i64, path: &str) -> Result<GameRules, String> {
    let sub_laps: i64 = 2_000_000_000;
    let mut result: GameRules = GameRules::new();
    if laps > sub_laps {
        while laps > 0 {
            println!(
                "{} laps left to go in sub laps of max {}",
                laps.to_formatted_string(&Locale::en),
                sub_laps.to_formatted_string(&Locale::en)
            );
            result = learn_game_sub_laps(sub_laps, path)?;
            laps -= sub_laps;
        }
    } else {
        result = learn_game_sub_laps(laps, path)?;
    }
    return Ok(result);
}

pub fn learn_game_sub_laps(laps: i64, path: &str) -> Result<GameRules, String> {
    // Use available threads, but save some for master thread
    let n_threads = available_threads() - 2;
    let sub_laps = laps / n_threads as i64;

    let mc_score = prepare_mc_score(path);
    let mc = Arc::new(Mutex::new(mc_score));

    (0..n_threads as u8).into_par_iter().for_each(|i| run_wrap(i, sub_laps, path.to_string(), mc.clone()));

    let mc_score = mc.lock().unwrap();
    post_process_mc_score(path, &mc_score)
}

fn run_wrap(
    part_no: u8,
    laps: i64,
    path: String,
    mc_score: Arc<Mutex<MCScore>>,
) {
    if let Err(e) = run(part_no, laps, path, mc_score) {
        println!("Error while learning game: {}", e);
    }
}

fn run(
    part_no: u8,
    laps: i64,
    path: String,
    mc_score: Arc<Mutex<MCScore>>,
) -> Result<(), String> {
    let mut dices = Dices::new();
    let mut mc = MCGame::new();

    let hands = load_hands(&path)?;
    let mut n_hands: usize;

    let mut total_laps: u64;

    println!("Thread {} starting {} laps", part_no, laps);
    for _ in 0..laps {
        n_hands = mc.reset();
        let mut results: Vec<IntermediateResult> = vec![IntermediateResult::new();n_hands];
        loop {
            let available_hands = if let Some(ah) = mc.available_hands() {
                ah
            } else {
                break;
            };

            let t1_code = base7_to_base10(&dices.throw_and_hold(None));
            let h1 = best_available_hand(Throw::First, t1_code, available_hands, &hands)?;
            let (_, s1_code, _) = hands[h1].optimal_holds(Throw::First)?.get(&t1_code).unwrap();

            let t2_code = base7_to_base10(&dices.throw_and_hold(Some(base10_to_base7(*s1_code))));
            let h2 = best_available_hand(Throw::Second, t2_code, available_hands, &hands)?;
            let (_, s2_code, _) = hands[h2].optimal_holds(Throw::Second)?.get(&t2_code).unwrap();

            let t3_code = base7_to_base10(&dices.throw_and_hold(Some(base10_to_base7(*s2_code))));

            let hand = mc.pick_hand().expect("should always return Some in this context");
            let score = hands[hand].score(base10_to_base7(t3_code));

            results[hand].available_hands = available_hands;
            results[hand].score = score as u8;
        }

        {
            let mut m = mc_score.lock().unwrap();
            total_laps = (*m).update_scores(&results);
        }

        if total_laps % 10000000 == 0 {
            println!("...processed {} laps", total_laps.to_formatted_string(&Locale::en));
        }
    }

    println!("Thread {} done", part_no);
    Ok(())
}

fn prepare_mc_score(path: &str) -> MCScore {
    println!("MCScore collector loading existing master score file");
    let mut mc_score = MCScore::new();
    if mc_score.load_scores(path).unwrap() == None {
        println!("MCScore collector found no existing master score file, starting a new one");
    }

    mc_score
}

fn post_process_mc_score(path: &str, mc_score: &MCScore) -> Result<GameRules, String> {
    println!("...saving master score file");
    mc_score.save_scores(&path)?;

    println!("...analysing optimal games");
    let mut gr = GameRules::new();
    mc_score.update_optimal_game(gr.optimal_games_mut());

    println!("...saving optimal games");
    gr.save_optimal_games(&path)?;

    Ok(gr)
}

pub fn load_game(path: &str) -> Result<GameRules, String> {
    let mut games_rules = GameRules::new();
    games_rules.load_optimal_games(path)?;

    Ok(games_rules)
}

pub fn print_statistics(path: &str) -> Result<(), String> {
    let mut mc_score = MCScore::new();

    println!("Start loading mc score file for statistics");
    if let None = mc_score.load_scores(path)? {
        return Err("Error, mc score file not found".to_string());
    }

    println!("...calculating statistics");
    let result = mc_score.statistics()?;

    println!("{}\n", result);

    Ok(())
}
