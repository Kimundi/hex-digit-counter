use original::Original;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::iter::FromIterator;

mod kimundi;
mod original;

pub trait Process {
    fn new(digit: usize) -> Self;
    fn on_byte(&mut self, b: u8);
    fn finalize(&mut self);
    fn into_count(self) -> HashMap<Vec<u8>, u128>;
}

fn main() {
    generic_main::<Original>();
}

static ERRMSG: &str =
    "Usage: <path> <digit> [buffer (MiB)]\nNote that maximum supported file size is 2^128-1 bytes.";
static CAPACITY: usize = 2048;
static INTERVAL: u128 = 10000000;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,

    #[structopt(name = "DIGIT")]
    digit: usize,

    #[structopt(name = "CAPACITY")]
    capacity: Option<usize>,
}

pub fn generic_main<T: Process>() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);

    let path = opt.file;
    let digit = opt.digit;
    let capacity = opt.capacity.unwrap_or_else(|| {
        println!("Using default capacity {}", CAPACITY);
        CAPACITY
    });

    let mut imp = T::new(digit);

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
        imp.on_byte(byte.unwrap());
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
    imp.finalize();
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
    let count = imp.into_count();
    for i in 0..digit {
        let mut filter = count.clone();
        filter.retain(|k, _| k.len() == i + 1);
        let tmap = std::collections::BTreeMap::from_iter(filter.iter());
        write!(output, "{} {:?}\n", tmap.len(), tmap.values()).unwrap();
    }
}
