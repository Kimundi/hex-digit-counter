use crate::{Counter, FastHashMap, Process};

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

const HEX_MASKS: &[Number; 17] = &[
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

type Number = u64;

pub trait CountStrat: Default {
    const COUNT_LATE: bool;
}
#[derive(Default)]
pub struct EarlyCount;
impl CountStrat for EarlyCount {
    const COUNT_LATE: bool = false;
}

#[derive(Default)]
pub struct LateCount;
impl CountStrat for LateCount {
    const COUNT_LATE: bool = true;
}

pub trait CounterStrategy<'a> {
    type ForWidth: CounterForWidth<'a>;

    fn new(digits: usize) -> Self;
    fn width_and_prev_width(&'a mut self, width: usize) -> (Self::ForWidth, Self::ForWidth);
    fn width(&'a mut self, width: usize) -> Self::ForWidth {
        self.width_and_prev_width(width).0
    }
}

pub struct HashMapCounter {
    count_maps: Vec<FastHashMap<Number, Counter>>,
    digits: usize,
}

pub trait CounterForWidth<'a> {
    fn for_each(&self, f: impl FnMut(Number, Counter));
    fn count_number(&mut self, v: Number, delta: u64);
}

pub struct HashMapCounterWidth<'a> {
    map: &'a mut FastHashMap<Number, Counter>,
    digits: usize,
    width: usize,
}

impl<'a> CounterForWidth<'a> for HashMapCounterWidth<'a> {
    fn for_each(&self, mut f: impl FnMut(Number, Counter)) {
        for (k, v) in self.map.iter() {
            f(*k, *v);
        }
    }
    fn count_number(&mut self, v: Number, delta: u64) {
        *self.map.entry(v).or_default() += delta;
        print_count_number_single_masked(v, 1, self.width, self.digits);
    }
}

impl<'a> CounterStrategy<'a> for HashMapCounter {
    type ForWidth = HashMapCounterWidth<'a>;

    fn new(digits: usize) -> Self {
        let mut count_maps = Vec::new();
        for _number_width in 0..(digits + 1) {
            count_maps.push(FastHashMap::default());
        }
        Self { count_maps, digits }
    }
    fn width_and_prev_width(&'a mut self, width: usize) -> (Self::ForWidth, Self::ForWidth) {
        let (prev, current) = self.count_maps.split_at_mut(width);
        let current = current.first_mut().unwrap();
        let prev = prev.last_mut().unwrap();
        (
            HashMapCounterWidth {
                digits: self.digits,
                width: width,
                map: current,
            },
            HashMapCounterWidth {
                digits: self.digits,
                width: width - 1,
                map: prev,
            },
        )
    }
}

pub struct Variant<T> {
    count_maps: HashMapCounter,
    digits: usize,
    current_number: u64,
    current_digits: usize,
    _strat: T,
}

impl<T: CountStrat> Variant<T> {
    fn new_internal(digits: usize) -> Self {
        assert!(std::mem::size_of::<Number>() * 2 >= digits);

        let count_maps = HashMapCounter::new(digits);

        let mut masks = Vec::new();

        for mask_width in 0..digits + 1 {
            let mut mask: u64 = 0;
            for _ in 0..mask_width {
                mask <<= 4;
                mask |= 0xf;
            }
            // println!("mask[{}] = {:x}", masks.len(), mask);
            masks.push(mask);
        }

        let current_number: u64 = 0;
        let current_digits = 0;

        Self {
            count_maps,
            digits,
            current_number,
            current_digits,
            _strat: T::default(),
        }
    }

    fn count_digit(&mut self, byte: u8) {
        let v = DIGIT_MAP[byte as usize];

        if v == 0xff {
            if self.current_digits != 0 {
                self.count_number_end(self.current_number, self.current_digits);
                self.current_digits = 0;
            }
            return;
        }

        if self.current_digits == self.digits {
            self.count_number_mid(self.current_number, self.current_digits);
            self.current_digits -= 1;
        }

        self.current_number <<= 4;
        self.current_number |= v as u64;
        self.current_digits += 1;
    }

    fn count_digit_end(&mut self) {
        if self.current_digits != 0 {
            self.count_number_end(self.current_number, self.current_digits);
        }
    }

    fn count_number(&mut self, v: u64, width: usize) {
        let mut v = v & HEX_MASKS[width];
        for width in (1..width + 1).rev() {
            self.count_maps.width(width).count_number(v, 1);
            v >>= 4;
            if T::COUNT_LATE {
                break;
            }
        }
    }

    fn count_number_end(&mut self, v: u64, mut width: usize) {
        while width != 0 {
            self.count_number(v, width);
            width -= 1;
        }
    }

    fn count_number_mid(&mut self, v: u64, width: usize) {
        self.count_number(v, width);
    }

    fn do_late_counts(&mut self) {
        if T::COUNT_LATE {
            //println!("Count all prefixes of numbers");
            for digits in (2..(self.digits + 1)).rev() {
                let (current, mut prev) = self.count_maps.width_and_prev_width(digits);

                //println!("  Numbers with {} digits", digits);
                current.for_each(|number, count| {
                    //println!("  prefix of {:0width$x}: {}", number, count, width = digits);
                    let prefix_number = number >> 4;
                    prev.count_number(prefix_number, count);
                });
            }
        }
    }

    fn _debug_output(&mut self) {
        for digits in (1..(self.digits + 1)).rev() {
            println!("Digit counts for width = {}", digits);

            self.count_maps.width(digits).for_each(|number, count| {
                println!("  {:0width$x}: {}", number, count, width = digits);
            });
        }
    }
}

#[allow(unused_variables)]
fn print_count_number_single_masked(v: u64, delta: Counter, width: usize, digits: usize) {
    /*
    println!(
        "  count {:0width$x}{:width2$}+{}",
        v,
        " ",
        delta,
        width = width,
        width2 = (digits - width) + 1
    );
    */
}

impl<T: CountStrat> Process for Variant<T> {
    fn new(digits: usize) -> Self {
        Self::new_internal(digits)
    }
    fn on_byte(&mut self, b: u8) {
        self.count_digit(b);
    }
    fn finalize(&mut self) {
        self.count_digit_end();
        self.do_late_counts();
    }
    fn into_count(mut self) -> FastHashMap<Vec<u8>, Counter> {
        let mut map = FastHashMap::default();
        let digits = self.digits;

        for digits in 1..digits + 1 {
            self.count_maps.width(digits).for_each(|mut number, count| {
                let mut vec = Vec::new();
                for _ in 0..digits {
                    let c = (number & 0xf) as u8;
                    let c = if c < 10 { b'0' + c } else { b'a' + (c - 10) };
                    vec.push(c);
                    number >>= 4;
                }
                vec.reverse();
                assert!(map.insert(vec, count as Counter).is_none());
            });
        }

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_out(d: usize, b: &[u8]) {
        type Ctx = Variant<LateCount>;
        let mut a = Ctx::new(d);
        for b in b.iter().copied() {
            a.on_byte(b);
        }
        a.finalize();
        a._debug_output();
    }

    #[test]
    fn test_main() {
        println!("-1----------------------------");
        test_out(2, b"1234567890");
        println!("-2----------------------------");
        test_out(2, b"_1234_5678_90_7");
        println!("-3----------------------------");
        test_out(5, b"1234567890_ffff");
        println!("-4----------------------------");
        test_out(5, b"1_23_456_7890_abcde_f01234");
        println!("-5----------------------------");
        test_out(5, b"1_23_456_7890_abcde_987654_f012341_23_456_123");
        println!("-fin--------------------------");
    }

    #[test]
    fn test_map() {
        assert_eq!(DIGIT_MAP[b'a' as usize], 10);
        assert_eq!(DIGIT_MAP[b'f' as usize], 15);
        assert_eq!(DIGIT_MAP[b'A' as usize], 10);
        assert_eq!(DIGIT_MAP[b'F' as usize], 15);
        assert_eq!(DIGIT_MAP[b'0' as usize], 0);
        assert_eq!(DIGIT_MAP[b'9' as usize], 9);
        assert_eq!(DIGIT_MAP[b'_' as usize], 255);
    }
}
