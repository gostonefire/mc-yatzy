mod utils;
mod dices;

use std::collections::HashMap;
use utils::{base7_to_base10, print_result};
use dices::Dices;


fn main() {
    let mut dice = Dices::new();
    let mut mc1: HashMap<(u16, u16), (f64, f64)> = HashMap::new();
    let mut mc2: HashMap<(u16, u16), (f64, f64)> = HashMap::new();

    for _ in 0..1000000 {
        let (t1_code, s1_code, t2_code, s2_code, score) = play_round(&mut dice);

        update_scores(&mut mc1, t1_code, s1_code, score);
        update_scores(&mut mc2, t2_code, s2_code, score);
    }

    let res1 = optimal_holds(&mc1);
    print_result(&res1);

    let res2 = optimal_holds(&mc2);
    print_result(&res2);

}

fn optimal_holds(mc: &HashMap<(u16,u16),(f64,f64)>) -> HashMap<u16, (u16, f64)> {
    let mut res: HashMap<u16, (u16, f64)> = HashMap::new();

    for ((throw, hold), (hits, value)) in mc.iter() {
        match res.get(throw) {
            Some((_, score)) => {
                if *value / *hits > *score {
                    res.insert(*throw, (*hold, *value / *hits));
                }
            }
            None => {
                res.insert(*throw, (*hold, *value / *hits));
            }
        }
    }
    res
}

fn update_scores(mc: &mut HashMap<(u16, u16), (f64, f64)>, t_code: u16, s_code: u16, score: f64) {
    match mc.get(&(t_code, s_code)) {
        Some((hits, value)) => {
            mc.insert((t_code, s_code), (*hits + 1.0, *value + score));
        }
        None => {
            mc.insert((t_code, s_code), (1.0, score));
        }
    }
}

fn play_round(dice: &mut Dices) -> (u16, u16, u16, u16, f64) {
    let throw1 = dice.throw_and_hold(None);
    let t1_code = base7_to_base10(&throw1);

    let selected1 = dice.select(throw1);
    let s1_code = base7_to_base10(&selected1);

    let throw2 = dice.throw_and_hold(Some(selected1));
    let t2_code = base7_to_base10(&throw2);

    let selected2 = dice.select(throw2);
    let s2_code = base7_to_base10(&selected2);

    let throw3 = dice.throw_and_hold(Some(selected2));

    let score = throw3
        .into_iter()
        .filter(|x| *x == 6)
        .collect::<Vec<u8>>()
        .len()
        * 6;

    (t1_code, s1_code, t2_code, s2_code, score as f64)
}
