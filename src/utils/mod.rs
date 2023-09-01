use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::thread;
use rayon::ThreadPool;

pub fn records_in_file(buf_reader: &mut BufReader<File>, context: &str) -> Result<u64, String> {
    let mut buf = [0u8;8];
    match buf_reader.read_exact(&mut buf) {
        Ok(()) => Ok(u64::from_le_bytes(buf)),
        Err(e) => return Err(format!("Error while reading from file {}: {}", context, e)),
    }
}

pub fn write_records_header<A, B>(file: &mut BufWriter<File>, arr: &Vec<&HashMap<A, B>>, context: &str) -> Result<(), String> {
    let n_records: u64 = arr.iter().map(|o| o.len() as u64).collect::<Vec<u64>>().iter().sum();
    if let Err(e) = file.write_all(&n_records.to_le_bytes()) {
        return Err(format!("Error while writing to file {}: {}", context, e));
    }

    Ok(())
}

pub fn thread_pool() -> Result<ThreadPool, String> {
    let n_threads = thread::available_parallelism()
        .expect("should get parallelism on this platform")
        .get();

    let pool = match rayon::ThreadPoolBuilder::new()
        .num_threads(n_threads)
        .build()
    {
        Ok(p) => {
            println!("Will use maximum of {} threads in parallel", n_threads);
            p
        }
        Err(e) => {
            return Err(format!("Error while creating ThreadPool: {}", e));
        }
    };

    Ok(pool)
}

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

pub fn base10_to_base2(b10: u16, one_based: bool) -> Vec<u8> {
    let one: u8 = if one_based {1} else {0};
    let mut d = b10 / 2;
    let mut r = b10 % 2;
    let mut tmp: Vec<u8> = Vec::new();
    let mut res: Vec<u8> = Vec::new();

    while d > 0 || r > 0 {
        tmp.push(r as u8);
        r = d % 2;
        d /= 2;
    }

    for i in 0..tmp.len() {
        if tmp[i] > 0 {
            res.push(i as u8 + one);
        }
    }
    res
}

pub fn sort_key(value: u16) -> u64 {
    let a = (16 - value.count_ones()) * 100000;
    let b = 32767 - (value as u32 ^ 32767);
    (a + b) as u64
}

pub fn initcap(data: String) -> String {
    let mut result = String::new();
    let mut first = true;
    for value in data.chars() {
        if first {
            result.push(value.to_ascii_uppercase());
            first = false;
        } else {
            result.push(value);
            if value == ' ' {
                first = true;
            }
        }
    }
    result
}
