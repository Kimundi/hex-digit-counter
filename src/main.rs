use original::Original;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::iter::FromIterator;
use std::{hash::BuildHasherDefault, path::PathBuf, time::Instant};
use structopt::StructOpt;
use variant::Variant;

mod original;
mod variant;

// NB: We will not exhaust a u64 with modern computers as long as its
// counted up by 1 at a time.
type Counter = u64;

//type HashFn = std::collections::hash_map::DefaultHasher;
type HashFn = fxhash::FxHasher;

type FastHashMap<K, V> = HashMap<K, V, BuildHasherDefault<HashFn>>;

const DEBUG_PRINT_COUNTS: bool = false;
const DEBUG_PRINT_COUNTS2: bool = false;
const DEBUG_HEX_HISTOGRAM: bool = false;
const DEBUG_COMPUTE_OUTPUT_HASH: bool = true;

const ERRMSG: &str =
    "Usage: <algorithm> <path> <digit> [buffer (MiB)]\nNote that maximum supported file size is 2^128-1 bytes.";
// NB: Capacity used per default by std::BufReader
const STD_CAPACITY: usize = 8 * 1024;
// const CAPACITY: usize = 2048;
const INTERVAL: Counter = 10000000;

trait Process {
    fn new(digit: usize) -> Self;
    fn on_byte(&mut self, b: u8);
    fn finalize(&mut self);
    fn into_count(self) -> FastHashMap<Vec<u8>, Counter>;
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct CliOptions {
    #[structopt(name = "ALGORITHM")]
    algorithm: String,

    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,

    #[structopt(name = "DIGIT")]
    digit: usize,

    #[structopt(name = "CAPACITY")]
    capacity: Option<usize>,

    #[structopt(short, long)]
    unmapped: bool,
}

fn main() {
    let opt = CliOptions::from_args();
    println!("{:#?}", opt);

    match &opt.algorithm[..] {
        "original" => generic_main::<Original<original::StdNumeric>>(opt),
        "original-hex" => generic_main::<Original<original::HexDigit>>(opt),
        "variant-1" => generic_main::<Variant<variant::EarlyCount, variant::HashMapCounter>>(opt),
        "variant-2" => generic_main::<Variant<variant::LateCount, variant::HashMapCounter>>(opt),
        "variant-3" => generic_main::<Variant<variant::EarlyCount, variant::VecCounter>>(opt),
        "variant-4" => generic_main::<Variant<variant::LateCount, variant::VecCounter>>(opt),
        other => {
            panic!("Unsupported algorithm {}\n{}", other, ERRMSG);
        }
    }
}

fn main_loop<T: Process>(
    imp: &mut T,
    mut iter: impl Iterator<Item = u8>,
    mut count_callback: impl FnMut(&Instant),
) -> Instant {
    for byte in &mut iter {
        if byte == b'.' {
            break;
        }
    }
    let mut hex_count = [0u64; 16];
    let now = std::time::Instant::now();
    for byte in iter {
        imp.on_byte(byte);
        count_callback(&now);
        if DEBUG_HEX_HISTOGRAM {
            let n = if b'0' <= byte && byte <= b'9' {
                byte - b'0'
            } else if b'a' <= byte && byte <= b'f' {
                byte - b'a' + 10
            } else if b'A' <= byte && byte <= b'F' {
                byte - b'A' + 10
            } else {
                continue;
            };
            hex_count[n as usize] += 1;
        }
    }
    if DEBUG_HEX_HISTOGRAM {
        println!("debug hex histogram: = {:?}", hex_count);
    }
    now
}

fn generic_main<T: Process>(opt: CliOptions) {
    let path = opt.file;
    let digit = opt.digit;

    let mut imp = T::new(digit);

    let mut cnt: Counter = 0;
    let mut cntp: Counter = 0;
    let do_count = |now: &Instant| {
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
    };

    let filestream = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(error) => panic!("{}\n{}", error, ERRMSG),
    };
    let now = if !opt.unmapped {
        println!("Memory mapped read");
        let memmap = unsafe { memmap::Mmap::map(&filestream).unwrap() };
        let iter = memmap.iter().copied();
        main_loop(&mut imp, iter, do_count)
    } else {
        println!("Buffered read");
        let capacity = opt.capacity.unwrap_or_else(|| {
            println!("Using default capacity {}", STD_CAPACITY);
            STD_CAPACITY
        });
        let bufstream = std::io::BufReader::with_capacity(capacity, filestream);
        let iter = bufstream.bytes().map(|b| b.unwrap());
        main_loop(&mut imp, iter, do_count)
    };

    println!(
        "Digits: {}, Eof Time: {}",
        cntp * INTERVAL + cnt,
        now.elapsed().as_secs_f64()
    );
    imp.finalize();
    println!(
        "Digits: {}, Final Time: {}",
        cntp * INTERVAL + cnt,
        now.elapsed().as_secs_f64()
    );

    let outpath = format!(
        "{}/{}_result.txt",
        path.parent().unwrap().display(),
        path.file_stem().unwrap().to_str().unwrap()
    );
    let count = imp.into_count();

    if DEBUG_PRINT_COUNTS {
        let mut count = count.iter().collect::<Vec<_>>();
        count.sort();
        for (k, v) in count {
            println!("[{}]: {:?}", std::str::from_utf8(&k).unwrap(), v);
        }
    }

    if DEBUG_PRINT_COUNTS2 {
        let mut count = count.iter().collect::<Vec<_>>();
        count.sort();

        let file_len = std::fs::metadata(&path).unwrap().len();
        println!("File size: {}", file_len);

        let mut counts = Vec::<Counter>::new();

        for (k, _) in count {
            while counts.len() <= k.len() {
                counts.push(0);
            }
            counts[k.len()] += 1;
            //println!("[{}]: {:?}", std::str::from_utf8(&k).unwrap(), v);
        }
        for (n, counts) in counts.iter().enumerate() {
            println!("Numeric substrings of len={}: {}", n, counts);
        }
    }

    {
        println!("Output path: {}", outpath);
        let mut output = match std::fs::File::create(&outpath) {
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

    if DEBUG_COMPUTE_OUTPUT_HASH {
        let out_bytes = std::fs::read(outpath).unwrap();

        let mut hasher = Sha256::new();

        hasher.update(out_bytes);

        let result = hasher.finalize();
        let result = &format!("{:x}", result)[..8];
        println!("Output hash: {}", result);
    }
}
