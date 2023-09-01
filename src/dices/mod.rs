use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use crate::utils::base7_to_base10;

pub enum Throw {
    First,
    Second,
    #[allow(dead_code)]
    Third,
}

pub struct Dices {
    rng: ThreadRng,
    die: Uniform<u8>,
    n_holds: Uniform<usize>,
    n_dies: usize,
}

impl Dices {
    pub fn new() -> Self {
        Dices {
            rng: rand::thread_rng(),
            die: Uniform::from(1..7),
            n_holds: Uniform::from(0..6),
            n_dies: 5,
        }
    }

    fn throw(&mut self, n_dies: usize) -> Vec<u8> {
        let res: Vec<u8> = (0..n_dies)
            .map(|_| self.die.sample(&mut self.rng))
            .collect();
        res
    }

    pub fn throw_and_hold(&mut self, hold: Option<Vec<u8>>) -> Vec<u8> {
        let mut res: Vec<u8>;

        match hold {
            Some(h) => {
                if h.len() >= self.n_dies {
                    res = h;
                } else {
                    res = self.throw(self.n_dies - h.len());
                    res.extend(h);
                }
            }
            None => {
                res = self.throw(self.n_dies);
            }
        }

        res.sort();
        res
    }

    fn select(&mut self, dices: Vec<u8>) -> Vec<u8> {
        let n_holds = self.n_holds.sample(&mut self.rng);

        let mut selected: Vec<u8> = dices
            .choose_multiple(&mut self.rng, n_holds)
            .cloned()
            .collect();
        selected.sort();

        selected
    }

    pub fn play_round(&mut self) -> (u16, u16, u8, u16, u16, u8, Vec<u8>) {
        let throw1 = self.throw_and_hold(None);
        let t1_code = base7_to_base10(&throw1);

        let selected1 = self.select(throw1);
        let s1_code = base7_to_base10(&selected1);
        let s1_len = selected1.len() as u8;

        let throw2 = self.throw_and_hold(Some(selected1));
        let t2_code = base7_to_base10(&throw2);

        let selected2 = self.select(throw2);
        let s2_code = base7_to_base10(&selected2);
        let s2_len = selected2.len() as u8;

        let throw3 = self.throw_and_hold(Some(selected2));

        (t1_code, s1_code, s1_len, t2_code, s2_code, s2_len, throw3)
    }
}
