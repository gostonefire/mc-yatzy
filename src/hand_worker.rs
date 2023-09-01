use crate::dices::Dices;
use crate::dices::Throw::{First, Second};
use crate::score_box::rules::*;
use crate::score_box::{Hands, MCHands};
use crate::utils::thread_pool;

enum LearnMode {
    Skip,
    Learn,
    Debug,
}

pub fn learn_hands(laps: i64, path: &str, rule: Option<usize>, debug: bool) -> Result<(), String> {
    let pool = thread_pool()?;
    let mut learn: [&LearnMode;15] = [&LearnMode::Skip;15];
    if let Some(r) = rule {
        learn[r] = if debug {&LearnMode::Debug} else {&LearnMode::Learn};
    } else {
        learn = if debug {[&LearnMode::Debug;15]} else {[&LearnMode::Learn;15]};
    }

    pool.in_place_scope(|s| {
        s.spawn(move |_| run(&mut MCHands::new(), &mut Ones::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Twos::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Threes::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Fours::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Fives::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Sixes::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut OnePair::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut TwoPairs::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut ThreeOfAKind::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut FourOfAKind::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut SmallStraight::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut LargeStraight::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut FullHouse::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Chance::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Yatzy::new(), laps, path, learn));
    });

    Ok(())
}

fn run<A: Hands>(mc: &mut MCHands, hand: &mut A, laps: i64, path: &str, learn: [&LearnMode;15]) {
    if let LearnMode::Skip = learn[hand.id()] { return; }

    let mut dices = Dices::new();
    println!("Learning {}", hand.name());

    for _ in 0..laps {
        let (t1_code, s1_code, s1_len, t2_code, s2_code, s2_len, throw3) = dices.play_round();
        let score = hand.score(throw3);

        mc.update_scores(First, t1_code, s1_code, s1_len, score);
        mc.update_scores(Second, t2_code, s2_code, s2_len, score);
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

pub fn load_hands(path: &str) -> Result<Vec<Box<dyn Hands>>, String> {
    let mut res: Vec<Box<dyn Hands>> = Vec::with_capacity(15);

    res.push(Box::new(Ones::new()));
    res.push(Box::new(Twos::new()));
    res.push(Box::new(Threes::new()));
    res.push(Box::new(Fours::new()));
    res.push(Box::new(Fives::new()));
    res.push(Box::new(Sixes::new()));
    res.push(Box::new(OnePair::new()));
    res.push(Box::new(TwoPairs::new()));
    res.push(Box::new(ThreeOfAKind::new()));
    res.push(Box::new(FourOfAKind::new()));
    res.push(Box::new(SmallStraight::new()));
    res.push(Box::new(LargeStraight::new()));
    res.push(Box::new(FullHouse::new()));
    res.push(Box::new(Chance::new()));
    res.push(Box::new(Yatzy::new()));

    for h in 0..res.len() {
        res[h].load_optimal_holds(path)?;
    }
    Ok(res)
}
