use std::collections::HashMap;
use std::io::{Read, Write};
use std::iter::FromIterator;

static ERRMSG: &str =
    "Usage: <path> <digit> [buffer (MiB)]\nNote that maximum supported file size is 2^128-1 bytes.";
static CAPACITY: usize = 2048;
static INTERVAL: u128 = 10000000;

fn is_not_numeric(vec: &Vec<u8>) -> bool {
    for &c in vec {
        if !char::from(c).is_numeric() {
            return true;
        }
    }
    return false;
}

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        panic!(ERRMSG);
    }
    let path = match std::fs::canonicalize(&args[1]) {
        Ok(result) => result,
        Err(_) => panic!(ERRMSG),
    };
    let digit: usize = match args[2].parse() {
        Ok(size) => size,
        Err(_) => {
            panic!(ERRMSG);
        }
    };
    let mut capacity: usize = CAPACITY;
    if args.len() > 3 {
        capacity = match args[3].parse() {
            Ok(size) => size,
            Err(_) => {
                println!("Using default capacity {}", CAPACITY);
                CAPACITY
            }
        };
    }
    let mut count: HashMap<Vec<u8>, u128> = HashMap::new();
    let mut buffer: Vec<Vec<u8>> = Vec::with_capacity(digit);
    for i in 0..digit {
        buffer.push(b"_".repeat(i + 1));
    }
    let filestream = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(error) => panic!("{}\n{}", error, ERRMSG),
    };
    let bufstream = std::io::BufReader::with_capacity(1048576 * capacity, filestream);
    let mut cnt: u128 = 0;
    let mut cntp: u128 = 0;
    let mut bytestream = bufstream.bytes();
    for byte in &mut bytestream {
        if byte.unwrap() == b'.' {
            break;
        }
    }
    let now = std::time::Instant::now();
    for byte in &mut bytestream {
        for i in 0..(buffer.len()) {
            buffer[i].remove(0);
            buffer[i].push(byte.as_ref().unwrap().clone());
            *count.entry(buffer[i].clone()).or_insert(0) += 1;
        }
        cnt += 1;
        if cnt == INTERVAL {
            cntp += 1;
            println!(
                "Digits: {}, Time: {}",
                cntp * INTERVAL,
                now.elapsed().as_secs_f64()
            );
            cnt = 0;
        }
    }
    println!(
        "Digits: {}, Final Time: {}",
        cntp * INTERVAL + cnt,
        now.elapsed().as_secs_f64()
    );
    for key in count.clone().keys() {
        if is_not_numeric(key) {
            count.remove(key);
        }
    }
    let outpath = format!(
        "{}/{}_result.txt",
        path.parent().unwrap().display(),
        path.file_stem().unwrap().to_str().unwrap()
    );
    println!("Output path: {}", outpath);
    let mut output = match std::fs::File::create(outpath) {
        Ok(file) => file,
        Err(error) => panic!("{}", error),
    };
    for i in 0..digit {
        let mut filter = count.clone();
        filter.retain(|k, _| k.len() == i + 1);
        let tmap = std::collections::BTreeMap::from_iter(filter.iter());
        write!(output, "{} {:?}\n", tmap.len(), tmap.values()).unwrap();
    }
}
