use crate::score_box::{Hands, OptimalHolds, Throw};
use std::collections::HashMap;

pub struct Ones {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl Ones {
    pub fn new() -> Ones {
        Ones {
            optimal_holds: OptimalHolds::new(),
            min_holds: 5,
            name: "ones".to_string(),
            id: 0,
        }
    }
}

impl Hands for Ones {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let score = dices
            .into_iter()
            .filter(|x| *x == 1)
            .collect::<Vec<u8>>()
            .len()
            * 1;

        score as f64
    }
}

pub struct Twos {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl Twos {
    pub fn new() -> Twos {
        Twos {
            optimal_holds: OptimalHolds::new(),
            min_holds: 5,
            name: "twos".to_string(),
            id: 1,
        }
    }
}

impl Hands for Twos {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let score = dices
            .into_iter()
            .filter(|x| *x == 2)
            .collect::<Vec<u8>>()
            .len()
            * 2;

        score as f64
    }
}

pub struct Threes {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl Threes {
    pub fn new() -> Threes {
        Threes {
            optimal_holds: OptimalHolds::new(),
            min_holds: 5,
            name: "threes".to_string(),
            id: 2,
        }
    }
}

impl Hands for Threes {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let score = dices
            .into_iter()
            .filter(|x| *x == 3)
            .collect::<Vec<u8>>()
            .len()
            * 3;

        score as f64
    }
}

pub struct Fours {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl Fours {
    pub fn new() -> Fours {
        Fours {
            optimal_holds: OptimalHolds::new(),
            min_holds: 5,
            name: "fours".to_string(),
            id: 3,
        }
    }
}

impl Hands for Fours {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let score = dices
            .into_iter()
            .filter(|x| *x == 4)
            .collect::<Vec<u8>>()
            .len()
            * 4;

        score as f64
    }
}

pub struct Fives {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl Fives {
    pub fn new() -> Fives {
        Fives {
            optimal_holds: OptimalHolds::new(),
            min_holds: 5,
            name: "fives".to_string(),
            id: 4,
        }
    }
}

impl Hands for Fives {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let score = dices
            .into_iter()
            .filter(|x| *x == 5)
            .collect::<Vec<u8>>()
            .len()
            * 5;

        score as f64
    }
}

pub struct Sixes {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl Sixes {
    pub fn new() -> Sixes {
        Sixes {
            optimal_holds: OptimalHolds::new(),
            min_holds: 5,
            name: "sixes".to_string(),
            id: 5,
        }
    }
}

impl Hands for Sixes {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let score = dices
            .into_iter()
            .filter(|x| *x == 6)
            .collect::<Vec<u8>>()
            .len()
            * 6;

        score as f64
    }
}

pub struct OnePair {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl OnePair {
    pub fn new() -> OnePair {
        OnePair {
            optimal_holds: OptimalHolds::new(),
            min_holds: 2,
            name: "one_pair".to_string(),
            id: 6,
        }
    }
}

impl Hands for OnePair {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let mut groups: [u8; 6] = [0; 6];
        for dice in dices {
            groups[dice as usize - 1] += 1;
        }

        let mut pair: u8 = 0;
        for dice in (1..7).rev() {
            if groups[dice - 1] > 1 {
                pair = dice as u8;
                break;
            }
        }

        (pair * 2) as f64
    }
}

pub struct TwoPairs {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl TwoPairs {
    pub fn new() -> TwoPairs {
        TwoPairs {
            optimal_holds: OptimalHolds::new(),
            min_holds: 4,
            name: "two_pairs".to_string(),
            id: 7,
        }
    }
}

impl Hands for TwoPairs {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let mut groups: [u8; 6] = [0; 6];
        for dice in dices {
            groups[dice as usize - 1] += 1;
        }

        let mut pairs: [u8; 2] = [0; 2];
        let mut pair: usize = 0;
        for dice in (1..7).rev() {
            if groups[dice - 1] > 1 {
                pairs[pair] = dice as u8;
                pair += 1;
            }
            if pair > 1 {
                break;
            }
        }

        let res = if pair < 2 {
            0
        } else {
            pairs[0] * 2 + pairs[1] * 2
        };

        res as f64
    }
}

pub struct ThreeOfAKind {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl ThreeOfAKind {
    pub fn new() -> ThreeOfAKind {
        ThreeOfAKind {
            optimal_holds: OptimalHolds::new(),
            min_holds: 3,
            name: "three_of_a_kind".to_string(),
            id: 8,
        }
    }
}

impl Hands for ThreeOfAKind {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let mut groups: [u8; 6] = [0; 6];
        for dice in dices {
            groups[dice as usize - 1] += 1;
        }

        let mut triple: u8 = 0;
        for dice in (1..7).rev() {
            if groups[dice - 1] > 2 {
                triple = dice as u8;
                break;
            }
        }

        (triple * 3) as f64
    }
}

pub struct FourOfAKind {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl FourOfAKind {
    pub fn new() -> FourOfAKind {
        FourOfAKind {
            optimal_holds: OptimalHolds::new(),
            min_holds: 4,
            name: "four_of_a_kind".to_string(),
            id: 9,
        }
    }
}

impl Hands for FourOfAKind {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let mut groups: [u8; 6] = [0; 6];
        for dice in dices {
            groups[dice as usize - 1] += 1;
        }

        let mut quad: u8 = 0;
        for dice in (1..7).rev() {
            if groups[dice - 1] > 3 {
                quad = dice as u8;
                break;
            }
        }

        (quad * 4) as f64
    }
}

pub struct SmallStraight {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl SmallStraight {
    pub fn new() -> SmallStraight {
        SmallStraight {
            optimal_holds: OptimalHolds::new(),
            min_holds: 5,
            name: "small_straight".to_string(),
            id: 10,
        }
    }
}

impl Hands for SmallStraight {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let score: f64 = match dices.as_slice() {
            [1, 2, 3, 4, 5] => 15.0,
            _ => 0.0,
        };

        score
    }
}

pub struct LargeStraight {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl LargeStraight {
    pub fn new() -> LargeStraight {
        LargeStraight {
            optimal_holds: OptimalHolds::new(),
            min_holds: 5,
            name: "large_straight".to_string(),
            id: 11,
        }
    }
}

impl Hands for LargeStraight {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let score: f64 = match dices.as_slice() {
            [2, 3, 4, 5, 6] => 20.0,
            _ => 0.0,
        };

        score
    }
}

pub struct FullHouse {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl FullHouse {
    pub fn new() -> FullHouse {
        FullHouse {
            optimal_holds: OptimalHolds::new(),
            min_holds: 5,
            name: "full_house".to_string(),
            id: 12,
        }
    }
}

impl Hands for FullHouse {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let mut groups: [u8; 6] = [0; 6];
        for dice in dices {
            groups[dice as usize - 1] += 1;
        }

        let mut triple: u8 = 0;
        let mut pair: u8 = 0;
        for dice in (1..7).rev() {
            if groups[dice - 1] > 2 {
                triple = dice as u8;
            } else if groups[dice - 1] > 1 {
                pair = dice as u8;
            }
            if triple > 0 && pair > 0 {
                break;
            }
        }

        let res = if triple > 0 && pair > 0 {
            triple * 3 + pair * 2
        } else {
            0
        };

        res as f64
    }
}

pub struct Chance {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl Chance {
    pub fn new() -> Chance {
        Chance {
            optimal_holds: OptimalHolds::new(),
            min_holds: 5,
            name: "chance".to_string(),
            id: 13,
        }
    }
}

impl Hands for Chance {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let score: u8 = dices.iter().sum();

        score as f64
    }
}

pub struct Yatzy {
    optimal_holds: OptimalHolds,
    min_holds: u8,
    name: String,
    id: usize,
}

impl Yatzy {
    pub fn new() -> Yatzy {
        Yatzy {
            optimal_holds: OptimalHolds::new(),
            min_holds: 5,
            name: "yatzy".to_string(),
            id: 14,
        }
    }
}

impl Hands for Yatzy {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn min_holds(&self) -> u8 {
        self.min_holds
    }

    fn optimal_holds_mut(&mut self) -> [&mut HashMap<u16, (u8, u16, f64)>;2] {
        [&mut self.optimal_holds.first, &mut self.optimal_holds.second]
    }

    fn optimal_holds(&self, throw: Throw) -> Result<&HashMap<u16, (u8, u16, f64)>, String> {
        match throw {
            Throw::First => Ok(&self.optimal_holds.first),
            Throw::Second => Ok(&self.optimal_holds.second),
            _ => Err("Illegal throw".to_string()),
        }
    }

    fn score(&self, dices: Vec<u8>) -> f64 {
        let mut groups: [u8; 6] = [0; 6];
        for dice in dices {
            groups[dice as usize - 1] += 1;
        }

        for dice in (1..7).rev() {
            if groups[dice - 1] > 4 {
                return 50.0;
            }
        }

        0.0
    }
}
