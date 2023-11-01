use crate::dices::Dices;
use crate::dices::Throw::{First, Second};
use crate::score_box::rules::*;
use crate::score_box::MCHands;
use crate::utils::thread_pool;
use crate::score_box::rules::HandType::*;
pub enum LearnMode {
    Skip,
    Learn,
    Debug,
}

pub fn learn_hands(laps: i64, path: &str, rule: Option<usize>, full: bool) -> Result<(), String> {
    let pool = thread_pool()?;
    let mut learn: [&LearnMode;15] = [&LearnMode::Skip;15];
    if let Some(r) = rule {
        learn[r] = if full {&LearnMode::Debug} else {&LearnMode::Learn};
    } else {
        learn = if full {[&LearnMode::Debug;15]} else {[&LearnMode::Learn;15]};
    }

    pool.in_place_scope(|s| {
        s.spawn(move |_| run(Ones, laps, path, learn));
        s.spawn(move |_| run(Twos, laps, path, learn));
        s.spawn(move |_| run(Threes, laps, path, learn));
        s.spawn(move |_| run(Fours, laps, path, learn));
        s.spawn(move |_| run(Fives, laps, path, learn));
        s.spawn(move |_| run(Sixes, laps, path, learn));
        s.spawn(move |_| run(OnePair, laps, path, learn));
        s.spawn(move |_| run(TwoPairs, laps, path, learn));
        s.spawn(move |_| run(ThreeOfAKind, laps, path, learn));
        s.spawn(move |_| run(FourOfAKind, laps, path, learn));
        s.spawn(move |_| run(SmallStraight, laps, path, learn));
        s.spawn(move |_| run(LargeStraight, laps, path, learn));
        s.spawn(move |_| run(FullHouse, laps, path, learn));
        s.spawn(move |_| run(Chance, laps, path, learn));
        s.spawn(move |_| run(Yatzy, laps, path, learn));
    });

    Ok(())
}

fn run(hand_type: HandType, laps: i64, path: &str, learn: [&LearnMode;15]) {
    if let LearnMode::Skip = learn[hand_type.id()] { return; }

    let mut dices = Dices::new();
    let mut hand = Hand::new(hand_type);
    let mut mc = MCHands::new();
    println!("Learning {}", hand.name());

    for _ in 0..laps {
        let (t1_code, s1_code, s1_len, t2_code, s2_code, s2_len, throw3) = dices.play_round();
        let score = hand.score(&throw3);

        mc.update_scores(First, t1_code, s1_code, s1_len, score as f64);
        mc.update_scores(Second, t2_code, s2_code, s2_len, score as f64);
    }

    let min_holds = hand.min_holds();
    mc.update_optimal_holds(hand.optimal_holds_mut(), min_holds);

    if let Err(e) = hand.save_optimal_holds(path) {
        println!("Could not save \"{}\n optimal holds, error: {}", hand.name(), e);
    }

    if let LearnMode::Debug = learn[hand.id()] {
        if let Err(e) = mc.debug_scores(path, &hand.name()) {
            println!("Could not export \"{}\n mc hand, error: {}", hand.name(), e);
        }
    }
}

pub fn load_hands(path: &str, fail: bool) -> Result<Vec<Box<Hand>>, String> {
    let mut res: Vec<Box<Hand>> = Vec::with_capacity(15);

    res.push(Box::new(Hand::new(Ones)));
    res.push(Box::new(Hand::new(Twos)));
    res.push(Box::new(Hand::new(Threes)));
    res.push(Box::new(Hand::new(Fours)));
    res.push(Box::new(Hand::new(Fives)));
    res.push(Box::new(Hand::new(Sixes)));
    res.push(Box::new(Hand::new(OnePair)));
    res.push(Box::new(Hand::new(TwoPairs)));
    res.push(Box::new(Hand::new(ThreeOfAKind)));
    res.push(Box::new(Hand::new(FourOfAKind)));
    res.push(Box::new(Hand::new(SmallStraight)));
    res.push(Box::new(Hand::new(LargeStraight)));
    res.push(Box::new(Hand::new(FullHouse)));
    res.push(Box::new(Hand::new(Chance)));
    res.push(Box::new(Hand::new(Yatzy)));

    let mut index = res.len();
    while index > 0 {
        if let Err(e) = res[index-1].load_optimal_holds(path) {
            if fail {
                return Err(e);
            }
            res.remove(index-1);
        }
        index -= 1;
    }

    Ok(res)
}
