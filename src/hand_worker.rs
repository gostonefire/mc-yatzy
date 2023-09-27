use crate::dices::Dices;
use crate::dices::Throw::{First, Second};
use crate::score_box::rules::*;
use crate::score_box::MCHands;
use crate::utils::thread_pool;
use crate::score_box::rules::HandType::*;
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
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(Ones), laps, path, learn));
        //s.spawn(move |_| run(&mut MCHands::new(), &mut Ones::new(), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(Twos), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(Threes), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(Fours), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(Fives), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(Sixes), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(OnePair), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(TwoPairs), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(ThreeOfAKind), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(FourOfAKind), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(SmallStraight), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(LargeStraight), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(FullHouse), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(Chance), laps, path, learn));
        s.spawn(move |_| run(&mut MCHands::new(), &mut Hand::new(Yatzy), laps, path, learn));
    });

    Ok(())
}

fn run(mc: &mut MCHands, hand: &mut Hand, laps: i64, path: &str, learn: [&LearnMode;15]) {
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

pub fn load_hands(path: &str) -> Result<Vec<Box<Hand>>, String> {
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

    for h in 0..res.len() {
        res[h].load_optimal_holds(path)?;
    }
    Ok(res)
}
