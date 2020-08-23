use path_abs::{PathAbs, PathInfo};
use std::io::Write;
use std::{path::PathBuf, time::Instant};
use structopt::StructOpt;

const DIGIT_MAP: &[u8; 256] = &[
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255,
    255, 255, 255, 255, 10, 11, 12, 13, 14, 15, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 10, 11, 12, 13,
    14, 15, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255,
];

const HEX_MASKS: &[HexString; 17] = &[
    0x0000000000000000,
    0x000000000000000f,
    0x00000000000000ff,
    0x0000000000000fff,
    0x000000000000ffff,
    0x00000000000fffff,
    0x0000000000ffffff,
    0x000000000fffffff,
    0x00000000ffffffff,
    0x0000000fffffffff,
    0x000000ffffffffff,
    0x00000fffffffffff,
    0x0000ffffffffffff,
    0x000fffffffffffff,
    0x00ffffffffffffff,
    0x0fffffffffffffff,
    0xffffffffffffffff,
];

type HexString = u64;
type Counter = u64;

const ERRMSG: &str = "Usage: <path> <digit>\nCounts all digit occurences with length <digit> and under after a period inside a text file in <path>. Note that maximum supported file size is pegged to the architecture.";
const INTERVAL: Counter = 10000000;

#[derive(StructOpt)]
#[structopt(name = "basic")]
struct CliOptions {
    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,

    #[structopt(name = "DIGIT")]
    digit: usize,
}

struct Variant {
    map: Vec<Vec<Counter>>,
    digits: usize,
    current_hexstring: HexString,
    current_digits: usize,
}

impl Variant {
    fn new(digits: usize) -> Self {
        assert!(std::mem::size_of::<HexString>() * 2 >= digits);
        assert!(std::mem::size_of::<HexString>() <= std::mem::size_of::<usize>());

        let mut map = Vec::new();
        let mut vec_len = 1;
        for _number_width in 0..(digits + 1) {
            map.push(vec![0; vec_len]);
            vec_len *= 16;
        }

        let mut masks = Vec::new();

        for mask_width in 0..digits + 1 {
            let mut mask: HexString = 0;
            for _ in 0..mask_width {
                mask <<= 4;
                mask |= 0xf;
            }
            masks.push(mask);
        }

        let current_hexstring: HexString = 0;
        let current_digits = 0;

        Self {
            map,
            digits,
            current_hexstring,
            current_digits,
        }
    }

    fn count_digit(&mut self, byte: u8) {
        let v = DIGIT_MAP[byte as usize];

        if v == 0xff {
            self.end_of_hexstring_run();
            return;
        }

        if self.current_digits == self.digits {
            self.count_hexstring(self.current_digits);
            self.current_digits -= 1;
        }

        self.current_hexstring <<= 4;
        self.current_hexstring |= v as HexString;
        self.current_digits += 1;
    }

    fn end_of_hexstring_run(&mut self) {
        if self.current_digits != 0 {
            self.count_hexstring_and_suffixes();
            self.current_digits = 0;
        }
    }

    fn finalize(&mut self) {
        self.end_of_hexstring_run();
        self.do_late_counts();
    }

    fn count_hexstring(&mut self, width: usize) {
        let v = self.current_hexstring & HEX_MASKS[width];
        let counter = self.counter_for_width_and_prev_width(width).0;
        counter[v as usize] += 1;
    }

    fn count_hexstring_and_suffixes(&mut self) {
        let mut width = self.current_digits;
        while width != 0 {
            self.count_hexstring(width);
            width -= 1;
        }
    }

    fn counter_for_width_and_prev_width(
        &mut self,
        width: usize,
    ) -> (&mut [Counter], &mut [Counter]) {
        let (prev, current) = self.map.split_at_mut(width);
        let current = current.first_mut().unwrap();
        let prev = prev.last_mut().unwrap();
        (current, prev)
    }

    fn hexstring_counts(s: &[Counter]) -> impl Iterator<Item = (HexString, Counter)> + '_ {
        s.iter()
            .copied()
            .enumerate()
            .map(|(k, v)| (k as HexString, v))
    }

    fn do_late_counts(&mut self) {
        for digits in (2..(self.digits + 1)).rev() {
            let (current, prev) = self.counter_for_width_and_prev_width(digits);
            for (number, count) in Self::hexstring_counts(current) {
                let prefix_number = number >> 4;
                prev[prefix_number as usize] += count;
            }
        }
    }
}

fn main_loop(
    imp: &mut Variant,
    mut iter: impl Iterator<Item = u8>,
    mut count_callback: impl FnMut(&Instant),
) -> Instant {
    for byte in &mut iter {
        if byte == b'.' {
            break;
        }
    }
    let now = std::time::Instant::now();
    for byte in iter {
        imp.count_digit(byte);
        count_callback(&now);
    }
    now
}

pub fn main() {
    let opt = CliOptions::from_args();
    let path = PathAbs::new(opt.file).unwrap();
    let digit = opt.digit;
    let mut imp = Variant::new(digit);
    let mut cnt = 0;
    let mut cntp = 0;
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
    let now = {
        let memmap = unsafe { memmap::Mmap::map(&filestream).unwrap() };
        let iter = memmap.iter().copied();
        main_loop(&mut imp, iter, do_count)
    };
    imp.finalize();
    println!(
        "Digits: {}, Final Time: {}",
        cntp * INTERVAL + cnt,
        now.elapsed().as_secs_f64()
    );
    let outpath = PathAbs::new(format!(
        "{}/{}_result.txt",
        path.parent().unwrap().display(),
        path.file_stem().unwrap().to_str().unwrap()
    ))
    .unwrap();
    println!("Output Path: {}", outpath.display());
    let mut output = match std::fs::File::create(&outpath) {
        Ok(file) => file,
        Err(error) => panic!("{}", error),
    };
    for vec in &imp.map {
        let filter = vec.iter().filter(|v| **v > 0).collect::<Vec<_>>();
        write!(output, "{} {:?}\n", filter.len(), filter).unwrap();
    }
}
