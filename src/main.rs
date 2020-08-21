use original::Original;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::iter::FromIterator;
use std::path::PathBuf;
use structopt::StructOpt;

mod kimundi;
mod original;

type Counter = u64;

trait Process {
    fn new(digit: usize) -> Self;
    fn on_byte(&mut self, b: u8);
    fn finalize(&mut self);
    fn into_count(self) -> HashMap<Vec<u8>, Counter>;
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(name = "ALGORITHM")]
    algorithm: String,

    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,

    #[structopt(name = "DIGIT")]
    digit: usize,

    #[structopt(name = "CAPACITY")]
    capacity: Option<usize>,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);

    match &opt.algorithm[..] {
        "original" => generic_main::<Original<original::StdNumeric>>(opt),
        "original-hex" => generic_main::<Original<original::HexDigit>>(opt),
        "kimundi-1" => generic_main::<kimundi::Variant1>(opt),
        other => {
            panic!("Unsupported algorithm {}\n{}", other, ERRMSG);
        }
    }
}

static ERRMSG: &str =
    "Usage: <algorithm> <path> <digit> [buffer (MiB)]\nNote that maximum supported file size is 2^128-1 bytes.";
static CAPACITY: usize = 2048;
static INTERVAL: u128 = 10000000;

fn generic_main<T: Process>(opt: Opt) {
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

    {
        let mut count = count.iter().collect::<Vec<_>>();
        count.sort();
        for (k, v) in count {
            println!("[{}]: {:?}", std::str::from_utf8(&k).unwrap(), v);
        }
    }

    for i in 0..digit {
        let mut filter = count.clone();
        filter.retain(|k, _| k.len() == i + 1);
        let tmap = std::collections::BTreeMap::from_iter(filter.iter());
        write!(output, "{} {:?}\n", tmap.len(), tmap.values()).unwrap();
    }
}
