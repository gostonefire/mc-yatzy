use std::collections::HashMap;
use std::io::stdin;
use std::str::FromStr;
use crate::dices::{Dices, Throw};
use crate::game_box::rules::GameRules;
use crate::game_worker::load_game;
use crate::hand_worker::load_hands;
use crate::score_box::rules::{best_available_hand, Hand};
use crate::utils::{base10_to_base2, base10_to_base7, base7_to_base10, initcap};

pub fn play_with_own_dices(path: &str) -> Result<(), String> {
    let mut human_available_hands: u16 = 32767;
    let mut mc_available_hands: u16 = 32767;
    let mut human_scores: HashMap<u8, u16> = HashMap::new();
    let mut mc_scores: HashMap<u8, u16> = HashMap::new();

    let hands = load_hands(path)?;
    let game = load_game(path)?;

    let hand_names = hands
        .iter()
        .map(|h| {
            let mut name = h.name();
            name.push(':');
            initcap(name)
        })
        .collect::<Vec<String>>();

    println!("Input dices without separators, e.g. 13234");
    while human_available_hands > 0 {
        let (dices, hand) = query_human_input(human_available_hands);
        let score = hands[hand as usize].score(dices);
        human_scores.insert(hand, score as u16);
        human_available_hands -= 2u16.pow(hand as u32);

        let (dices, hand) = query_mc_input(mc_available_hands, &game, &hands)?;
        let score = hands[hand as usize].score(dices);
        mc_scores.insert(hand, score as u16);
        mc_available_hands -= 2u16.pow(hand as u32);

        print_score_card(&human_scores, &mc_scores, &hand_names);
    }

    Ok(())
}

fn query_mc_input(available_hands: u16, game: &GameRules, hands: &Vec<Box<Hand>>) -> Result<(Vec<u8>, u8), String> {
    let mut dices = Dices::new();

    let t1_code = base7_to_base10(&dices.throw_and_hold(None));
    let h1 = best_available_hand(Throw::First, t1_code, available_hands, &hands)?;
    let (_, s1_code, _) = hands[h1].optimal_holds(Throw::First)?.get(&t1_code).unwrap();

    let t2_code = base7_to_base10(&dices.throw_and_hold(Some(base10_to_base7(*s1_code))));
    let h2 = best_available_hand(Throw::Second, t2_code, available_hands, &hands)?;
    let (_, s2_code, _) = hands[h2].optimal_holds(Throw::Second)?.get(&t2_code).unwrap();

    let t3_code = base7_to_base10(&dices.throw_and_hold(Some(base10_to_base7(*s2_code))));
    let h3 = best_available_game_hand(t3_code, available_hands, &hands, &game)?;

    Ok((base10_to_base7(t3_code), h3 as u8))
}

fn query_human_input(available_hands: u16) -> (Vec<u8>, u8) {
    let throw = [
        ("First throw:", "First hold:"),
        ("Second throw:", "Second hold:"),
        ("Third throw:", "")
    ];

    let mut t_vec: Vec<u8> = Vec::new();
    let mut h_vec: Vec<u8> = Vec::new();
    for (i, t) in throw.iter().enumerate() {
        t_vec = get_dices_input(t.0, None, Some(&h_vec));
        println!("Your dices: {:?}", t_vec);

        if i < 2 {
            h_vec = get_dices_input(t.1, Some(&t_vec), None);
            if h_vec.len() == 5 {
                println!("\nYou stayed with dices: {:?}", h_vec);
                break;
            } else {
                println!("\nYou are holding: {:?}", h_vec);
            }
        }
    }
    println!();

    let hand = get_hand_choice(base10_to_base2(available_hands, true));
    (t_vec, hand)
}

fn check_hold(dices: &Vec<u8>, hold: &Vec<u8>) -> bool {
    let mut hold_iter = hold.iter();
    let mut h = if let Some(h) = hold_iter.next() {
        *h
    } else {
        return true;
    };

    for d in dices {
        if *d == h {
            h = if let Some(h) = hold_iter.next() {
                *h
            } else {
                return true;
            };
        }
    }

    false
}

fn get_hand_choice(available_hands: Vec<u8>) -> u8 {
    let stdin = stdin();
    let mut input = String::new();

    println!("Chose hand to score from {:?}:", available_hands);
    loop {
        input.clear();
        stdin.read_line(&mut input).unwrap();
        input = input.trim().to_string();

        match u8::from_str(&input) {
            Ok(d) if available_hands.contains(&d) => {
                return d-1;
            }
            _ => {
                println!("...choice of [{}] not an available hand, try again!", input);
                continue;
            }
        }
    }
}

pub fn best_available_game_hand(thrown: u16, available_hands: u16, hands: &Vec<Box<Hand>>, game: &GameRules) -> Result<usize, String> {
    let mut best_hand: Option<usize> = None;
    let mut expected_score: f64;
    let mut max_score: f64 = -1.0;
    let mut score: u8;

    for hand in base10_to_base2(available_hands, false) {
        score = hands[hand as usize].score(base10_to_base7(thrown)) as u8;
        expected_score = game.expected_total_score(score, hand, available_hands)?;

        if expected_score > max_score {
            max_score = expected_score;
            best_hand = Some(hand as usize);
        }
    }

    if let Some(hand) = best_hand {Ok(hand)} else {Err("No best hand found".to_string())}
}

fn get_dices_input(caption: &str, dices: Option<&Vec<u8>>, hold: Option<&Vec<u8>>) -> Vec<u8> {
    let stdin = stdin();
    let mut input = String::new();
    let mut res: Vec<u8> = Vec::with_capacity(5);

    let (min_dices, max_dices): (usize, usize) = if let Some(_) = dices {
        (0, 5)
    } else if let Some(h) = hold {
        (5 - h.len(), 5 - h.len())
    } else {
        (5, 5)
    };

    println!("{}", caption);
    'outer: loop {
        input.clear();
        stdin.read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if input.len() < min_dices || input.len() > max_dices {
            println!(
                "...wrong number of dices, must be {}, try again!",
                if max_dices != min_dices {
                    format!("between {} and {}", min_dices, max_dices)
                } else {
                    min_dices.to_string()
                }
            );
            continue 'outer;
        }

        res.clear();
        let chars = input.chars();
        for c in chars {
            match u8::from_str(&c.to_string()) {
                Ok(d) if d > 0 && d < 7 => res.push(d),
                _ => {
                    println!("...input contained illegal character/number: [{}], try again!", c);
                    continue 'outer;
                }
            }
        }

        if let Some(d) = dices {
            if !check_hold(d, &res) {
                println!("...not fully part of available dices: {:?} - {:?}, try again!", d, res);
                continue 'outer;
            }
        }

        if let Some(h) = hold {
            res.extend(h);
        }

        res.sort();
        break 'outer;
    }

    res
}

fn print_score_card(human_scores: &HashMap<u8, u16>, mc_scores: &HashMap<u8, u16>, names: &Vec<String>) {
    let mut human_total: u16 = 0;
    let mut mc_total: u16 = 0;

    println!("____________________________________");
    println!("| Player:              |Human|  MC |");
    println!("|==================================|");
    for i in 0..15u8 {
        let human_score = match human_scores.get(&i) {
            Some(score) => {
                human_total += *score;
                score.to_string()
            },
            None => String::new()
        };
        let mc_score = match mc_scores.get(&i) {
            Some(score) => {
                mc_total += *score;
                score.to_string()
            },
            None => String::new()
        };
        println!("| {:2}. {:16} | {:3} | {:3} |", i+1, names[i as usize], human_score, mc_score);

        if i == 5 {
            println!("|==================================|");
            println!("|     {:16} | {:3} | {:3} |", "Sum:", human_total, mc_total);
            let human_bonus: u16 = if human_total >= 63 {50} else {0};
            let mc_bonus: u16 = if mc_total >= 63 {50} else {0};
            println!("|     {:16} | {:3} | {:3} |", "Bonus:", human_bonus, mc_bonus);
            human_total += human_bonus;
            mc_total += mc_bonus
        }
    }
    println!("|==================================|");
    println!("     {:16}   {:3}   {:3}\n", "Total:", human_total, mc_total);
}