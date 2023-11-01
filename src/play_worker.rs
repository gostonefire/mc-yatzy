use std::collections::HashMap;
use std::io::stdin;
use std::str::FromStr;
use colored::{ColoredString, Colorize};
use crate::dices::{Dices, Throw};
use crate::weight_worker::{best_available_game_hand, load_weights};
use crate::hand_worker::load_hands;
use crate::score_box::rules::{best_available_hand, Hand};
use crate::utils::{base10_to_base2, base10_to_base7, base7_to_base10, initcap};

pub fn play_with_own_dices(path: &str, bonus: u32) -> Result<(), String> {
    let mut human_available_hands: u16 = 32767;
    let mut mc_available_hands: u16 = 32767;
    let mut human_scores: HashMap<u8, u16> = HashMap::new();
    let mut mc_scores: HashMap<u8, u16> = HashMap::new();

    let mut dices = Dices::new();
    let hands = load_hands(path, true)?;
    let weights_vec = load_weights(path, Some(bonus))?;
    let weights = if let Some((_, w)) = weights_vec {
        if w.len() > 0 {
            w[0].weights
        } else {
            return Err(String::from("Error, no weights in weights file"));
        }
    } else {
        return Err(String::from("Error, no weights file available"));
    };

    let hand_names = hands
        .iter()
        .map(|h| {
            let mut name = h.name();
            name.push(':');
            initcap(name)
        })
        .collect::<Vec<String>>();

    print_score_card(&human_scores, &mc_scores, &hand_names, (0, 0));
    println!("Input dices without separators, e.g. 13234");
    while human_available_hands > 0 {
        let (thrown, hand) = query_human_input(human_available_hands);
        let score = hands[hand as usize].score(&thrown);
        human_scores.insert(hand, score as u16);
        human_available_hands -= 2u16.pow(hand as u32);

        let (dices, mc_hand) = query_mc_input(&mut dices, mc_available_hands, &hands, weights)?;
        let score = hands[mc_hand as usize].score(&dices);
        mc_scores.insert(mc_hand, score as u16);
        mc_available_hands -= 2u16.pow(mc_hand as u32);

        print_score_card(&human_scores, &mc_scores, &hand_names, (hand, mc_hand));
    }

    Ok(())
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

fn query_mc_input(dices: &mut Dices, available_hands: u16, hands: &Vec<Box<Hand>>, weights: [f32;15]) -> Result<(Vec<u8>, u8), String> {

    let t3 = throw_hand(dices, available_hands, hands)?;
    let (h3, _) = best_available_game_hand(&t3, available_hands, hands, weights)?;

    Ok((t3, h3))
}

pub fn throw_hand(dices: &mut Dices, available_hands: u16, hands: &Vec<Box<Hand>>) -> Result<Vec<u8>, String> {
    let t1_code = base7_to_base10(&dices.throw_and_hold(None));
    let h1 = best_available_hand(Throw::First, t1_code, available_hands, &hands)?;
    let (_, s1_code, _) = hands[h1].optimal_holds(Throw::First)?.get(&t1_code).unwrap();

    let t2_code = base7_to_base10(&dices.throw_and_hold(Some(base10_to_base7(*s1_code))));
    let h2 = best_available_hand(Throw::Second, t2_code, available_hands, &hands)?;
    let (_, s2_code, _) = hands[h2].optimal_holds(Throw::Second)?.get(&t2_code).unwrap();

    let t3 = dices.throw_and_hold(Some(base10_to_base7(*s2_code)));

    Ok(t3)
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

fn print_score_card(human_scores: &HashMap<u8, u16>, mc_scores: &HashMap<u8, u16>, names: &Vec<String>, latest: (u8, u8)) {
    let mut human_total: u16 = 0;
    let mut mc_total: u16 = 0;

    println!("____________________________________");
    println!("| Player:              |Human|  MC |");
    println!("|==================================|");
    for i in 0..15u8 {
        let human_score = format_score_string(&human_scores, i, i==latest.0, &mut human_total);
        let mc_score = format_score_string(&mc_scores, i, i==latest.1, &mut mc_total);

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

fn format_score_string(scores: &HashMap<u8, u16>, pos: u8, is_latest: bool, total: &mut u16) -> ColoredString {
    let mut formatted_score = match scores.get(&pos) {
        Some(s) => {
            *total += *s;
            if pos < 6 && *s < (pos as u16 + 1) * 3 {
                s.to_string().bright_yellow()
            } else {
                s.to_string().normal()
            }
        }
        None => String::new().normal()
    };

    if is_latest {formatted_score = formatted_score.bright_green();}

    formatted_score
}