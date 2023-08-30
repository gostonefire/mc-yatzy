use std::collections::HashMap;

pub fn base7_to_base10(b7: &Vec<u8>) -> u16 {
    let length = b7.len() as u32;
    let mut res: u16 = 0;

    if length > 0 {
        for (i, v) in b7
            .iter()
            .enumerate()
            .map(|x| (length - x.0 as u32 - 1, *x.1 as u16))
        {
            res += u16::pow(7, i) * v;
        }
    }
    res
}

pub fn base10_to_base7(b10: u16) -> Vec<u8> {
    let mut d = b10 / 7;
    let mut r = b10 % 7;
    let mut res: Vec<u8> = Vec::new();

    while d > 0 || r > 0 {
        res.push(r as u8);
        r = d % 7;
        d /= 7;
    }

    res.reverse();
    res
}

pub fn print_result(mc: &HashMap<u16, (u16, f64)>) {
    let mut throws: Vec<u16> = mc.keys().map(|k| *k).collect();
    throws.sort();

    for throw in throws {
        let (hold,score) = mc.get(&throw).unwrap();
        let tv = base10_to_base7(throw);
        let hv = base10_to_base7(*hold);

        println!("Throw: {:?} Hold: {:?} Score: {}", tv, hv, *score);
    }
}
