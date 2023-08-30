use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

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

    pub fn select(&mut self, dices: Vec<u8>) -> Vec<u8> {
        let n_holds = self.n_holds.sample(&mut self.rng);

        let mut selected: Vec<u8> = dices
            .choose_multiple(&mut self.rng, n_holds)
            .cloned()
            .collect();
        selected.sort();

        selected
    }
}
