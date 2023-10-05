use crate::dices::{Dices, Throw};
use crate::hand_worker::LearnMode;
use crate::score_box::rules::*;
use crate::score_box::rules::HandType::*;
use crate::utils::{base10_to_base7, base7_to_base10, thread_pool};

pub fn learn_hand_distributions(laps: i64, path: &str, rule: Option<usize>) -> Result<(), String> {
    let pool = thread_pool()?;
    let mut learn: [&LearnMode;15] = [&LearnMode::Skip;15];
    if let Some(r) = rule {
        learn[r] = &LearnMode::Learn;
    } else {
        learn = [&LearnMode::Learn;15];
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

    let mut hand = Hand::new(hand_type.clone());
    if let Err(e) = hand.load_optimal_holds(path) {
        println!("{}", e);
        return;
    }

    let mut dices = Dices::new();
    let mut hd = HandDistribution::new(hand_type);
    println!("Distribution learning {}", hd.name());

    for _ in 0..laps {
        hd.update_scores(play_hand(&mut dices, &hand));
    }

    if let Err(e) = hd.save_distribution(path) {
        println!("Could not save \"{}\n distribution, error: {}", hd.name(), e);
    }
}

fn play_hand(dices: &mut Dices, hand: &Hand) -> u8 {
    let t1_code = base7_to_base10(&dices.throw_and_hold(None));
    let (_, s1_code, _) = hand.optimal_holds(Throw::First).unwrap().get(&t1_code).unwrap();

    let t2_code = base7_to_base10(&dices.throw_and_hold(Some(base10_to_base7(*s1_code))));
    let (_, s2_code, _) = hand.optimal_holds(Throw::Second).unwrap().get(&t2_code).unwrap();

    let t3_code = base7_to_base10(&dices.throw_and_hold(Some(base10_to_base7(*s2_code))));

    hand.score(base10_to_base7(t3_code)) as u8
}

pub fn load_hand_distributions(path: &str, fail: bool) -> Result<Vec<Box<HandDistribution>>, String> {
    let mut res: Vec<Box<HandDistribution>> = Vec::with_capacity(15);

    res.push(Box::new(HandDistribution::new(Ones)));
    res.push(Box::new(HandDistribution::new(Twos)));
    res.push(Box::new(HandDistribution::new(Threes)));
    res.push(Box::new(HandDistribution::new(Fours)));
    res.push(Box::new(HandDistribution::new(Fives)));
    res.push(Box::new(HandDistribution::new(Sixes)));
    res.push(Box::new(HandDistribution::new(OnePair)));
    res.push(Box::new(HandDistribution::new(TwoPairs)));
    res.push(Box::new(HandDistribution::new(ThreeOfAKind)));
    res.push(Box::new(HandDistribution::new(FourOfAKind)));
    res.push(Box::new(HandDistribution::new(SmallStraight)));
    res.push(Box::new(HandDistribution::new(LargeStraight)));
    res.push(Box::new(HandDistribution::new(FullHouse)));
    res.push(Box::new(HandDistribution::new(Chance)));
    res.push(Box::new(HandDistribution::new(Yatzy)));

    let mut index = res.len();
    while index > 0 {
        if let Err(e) = res[index-1].load_distribution(path) {
            if fail {
                return Err(e);
            }
            res.remove(index-1);
        }
        index -= 1;
    }

    Ok(res)
}
