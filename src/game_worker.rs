
use crate::dices::{Dices, Throw};
use crate::game_box::rules::GameRules;
use crate::game_box::{IntermediateResult, MCGame, MCScore};
use crate::hand_worker::load_hands;
use crate::utils::{base10_to_base2, base10_to_base7, base7_to_base10};
use num_format::{Locale, ToFormattedString};
use rayon::spawn;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use crate::score_box::rules::Hand;

enum CollectorCmd {
    InitDone,
    Result(GameRules),
    Error(String),
}

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
    // We will be using two threads as players
    let sub_laps = laps / 2;

    // Set-up locks, path clones and channels for the threads to be spawned
    let lock = Arc::new(Mutex::new(0u64));
    let lock0 = lock.clone();
    let lock1 = lock.clone();
    let lock2 = lock.clone();
    let path_box: Arc<String> = Arc::new(path.to_string());
    let path_box0 = path_box.clone();
    let path_box1 = path_box.clone();
    let path_box2 = path_box.clone();
    let (control_tx, control_rx): (Sender<CollectorCmd>, Receiver<CollectorCmd>) = mpsc::channel();
    let (data_tx0, data_rx): (
        Sender<Vec<IntermediateResult>>,
        Receiver<Vec<IntermediateResult>>,
    ) = mpsc::channel();
    let data_tx1 = data_tx0.clone();
    let data_tx2 = data_tx0.clone();

    // Fire away the collector thread and wait for its initialization to be done
    spawn(move || mcscore_collector(path_box, data_rx, control_tx, lock));
    match control_rx.recv().unwrap() {
        CollectorCmd::InitDone => (),
        _ => return Err("Bug, collector did not send InitDone when expected to".to_string()),
    }

    // Start worker threads, two should be just about right
    spawn(move || run_wrap(0, sub_laps, path_box0, data_tx0, lock0));
    spawn(move || run_wrap(1, sub_laps, path_box1, data_tx1, lock1));
    spawn(move || run_wrap(2, sub_laps, path_box2, data_tx2, lock2));

    // Wait for collector thread to finish up its work then return game rules result
    return match control_rx.recv().unwrap() {
        CollectorCmd::Result(gr) => Ok(gr),
        CollectorCmd::Error(e) => Err(e),
        CollectorCmd::InitDone => {
            Err("Bug, got InitDone MCScore collector, expected Result or Error".to_string())
        }
    };
}

fn run_wrap(
    part_no: u8,
    laps: i64,
    path: Arc<String>,
    ch_tx: Sender<Vec<IntermediateResult>>,
    lock: Arc<Mutex<u64>>,
) {
    if let Err(e) = run(part_no, laps, path, ch_tx, lock) {
        println!("Error while learning game: {}", e);
    }
}

fn run(
    part_no: u8,
    laps: i64,
    path: Arc<String>,
    ch_tx: Sender<Vec<IntermediateResult>>,
    lock: Arc<Mutex<u64>>,
) -> Result<(), String> {
    let mut dices = Dices::new();
    let mut mc = MCGame::new();

    let hands = load_hands(&path)?;
    let mut n_hands: usize;

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
            results[hand].thrown = t3_code;
            results[hand].score = score;
        }

        // Check lock so collector can pause this thread if we are to far ahead
        {
            let mut a = lock.lock().unwrap();
            *a += 1;
        }

        if let Err(e) = ch_tx.send(results) {
            return Err(format!(
                "Error while sending result to MCScore master thread: {}",
                e
            ));
        }
    }

    println!("Thread {} done", part_no);
    Ok(())
}

fn mcscore_collector(
    path: Arc<String>,
    ch_rx: Receiver<Vec<IntermediateResult>>,
    ch_tx: Sender<CollectorCmd>,
    lock: Arc<Mutex<u64>>,
) {
    println!("MCScore collector loading existing master score file");
    let mut mc_score = MCScore::new();
    if mc_score.load_scores(&path).unwrap() == None {
        println!("MCScore collector found no existing master score file, starting a new one");
    }
    // Send to inform the master flow that the collector is ready to take results
    ch_tx.send(CollectorCmd::InitDone).unwrap();

    // Loop until all worker threads has been terminated, thus being released by expected recv error
    let mut laps: u64 = 0;
    let mut guard: Option<MutexGuard<u64>> = None;
    let mut release_target: u64 = 0;

    loop {
        match ch_rx.recv() {
            Ok(ir) => {
                mc_score.update_scores(&ir);
                laps += 1;
                if laps % 10000000 == 0 {
                    println!("...processed {} laps", laps.to_formatted_string(&Locale::en));
                }

                // Control senders to wait if they are too far ahead
                match guard {
                    None => {
                        let g = lock.lock().unwrap();
                        if *g > laps + 1_000_000 {
                            println!("...collector lifting guard");
                            release_target = *g - 1000;
                            guard = Some(g);
                        }
                    }
                    Some(d) => {
                        if laps >= release_target {
                            println!("...collector dropping guard");
                            drop(d);
                            guard = None;
                        } else {
                            guard = Some(d);
                        }
                    }
                }
            }
            Err(_) => {
                println!("...saving master score file");
                if let Err(e) = mc_score.save_scores(&path) {
                    ch_tx
                        .send(CollectorCmd::Error(format!(
                            "Error while saving master file: {}",
                            e
                        )))
                        .unwrap();
                    return;
                }
                println!("...analysing optimal games");
                let mut gr = GameRules::new();
                mc_score.update_optimal_game(gr.optimal_games_mut());

                println!("...saving optimal games");
                gr.save_optimal_games(&path).unwrap();

                // Send to inform the sequential flow that the collector is completely done
                ch_tx.send(CollectorCmd::Result(gr)).unwrap();
                return;
            }
        }
    }
}

fn best_available_hand(throw: Throw, thrown: u16, available_hands: u16, hands: &Vec<Box<Hand>>) -> Result<usize, String> {
    let mut best_hand: Option<usize> = None;
    let mut max_prob: f64 = 0.0;
    let mut prob: f64;

    match throw {
        Throw::First => {
            for hand in base10_to_base2(available_hands, false) {
                prob = hands[hand as usize].max_score_probability(Throw::First, thrown)?;
                if prob > max_prob {
                    max_prob = prob;
                    best_hand = Some(hand as usize);
                }
            }
        },
        Throw::Second => {
            for hand in base10_to_base2(available_hands, false) {
                prob = hands[hand as usize].max_score_probability(Throw::Second, thrown)?;
                if prob > max_prob {
                    max_prob = prob;
                    best_hand = Some(hand as usize);
                }
            }
        }
        _ => return Err("Illegal throw at this point".to_string()),
    }

    if let Some(hand) = best_hand {Ok(hand)} else {Err("No best hand found".to_string())}
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
