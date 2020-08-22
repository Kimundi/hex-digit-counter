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

struct Context<T> {
    count_maps: Vec<FastHashMap<Number, Counter>>,
    digits: usize,
    _strat: T,
}

impl<T: CountStrat> Context<T> {
    fn new(digits: usize) -> Self {
        assert!(std::mem::size_of::<Number>() * 2 >= digits);

        let mut count_maps = Vec::new();
        for _number_width in 0..(digits + 1) {
            count_maps.push(FastHashMap::default());
        }
        //dbg!(digits);

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

        Self {
            count_maps,
            digits,
            _strat: T::default(),
        }
    }

    fn count_digit(&mut self, byte: u8, state: &mut BytesState) {
        let v = DIGIT_MAP[byte as usize];

        if v == 0xff {
            if state.current_digits != 0 {
                self.count_number_end(state.current_number, state.current_digits);
                state.current_digits = 0;
            }
            return;
        }

        if state.current_digits == state.digits {
            self.count_number_mid(state.current_number, state.current_digits);
            state.current_digits -= 1;
        }

        state.current_number <<= 4;
        state.current_number |= v as u64;
        state.current_digits += 1;
    }

    fn count_digit_end(&mut self, state: &mut BytesState) {
        if state.current_digits != 0 {
            self.count_number_end(state.current_number, state.current_digits);
        }
    }

    fn count_number(&mut self, v: u64, width: usize) {
        let mut v = v & HEX_MASKS[width];
        for width in (1..width + 1).rev() {
            Self::count_number_single_masked(&mut self.count_maps[width], v, 1);
            Self::print_count_number_single_masked(v, 1, width, self.digits);
            v >>= 4;
            if T::COUNT_LATE {
                break;
            }
        }
    }

    fn count_number_single_masked(
        count_map: &mut FastHashMap<Number, Counter>,
        v: u64,
        delta: Counter,
    ) {
        *count_map.entry(v).or_default() += delta;
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

    fn count_number_end(&mut self, v: u64, mut width: usize) {
        while width != 0 {
            self.count_number(v, width);
            width -= 1;
        }
    }

    fn count_number_mid(&mut self, v: u64, width: usize) {
        self.count_number(v, width);
    }

    fn compute_sub_counts(&mut self) -> &mut Self {
        if T::COUNT_LATE {
            //println!("Count all prefixes of numbers");
            for digits in (2..(self.digits + 1)).rev() {
                let (prev, current) = self.count_maps.split_at_mut(digits);
                let current = current.first().unwrap();
                let prev = prev.last_mut().unwrap();

                //println!("  Numbers with {} digits", digits);
                for (&number, &count) in current {
                    //println!("  prefix of {:0width$x}: {}", number, count, width = digits);
                    let prefix_number = number >> 4;
                    Self::count_number_single_masked(prev, prefix_number, count);
                    Self::print_count_number_single_masked(
                        prefix_number,
                        count,
                        digits - 1,
                        self.digits,
                    );
                }
            }
        }

        self
    }

    fn debug_output(&self) {
        for digits in (1..(self.digits + 1)).rev() {
            println!("Digit counts for width = {}", digits);
            let map = &self.count_maps[digits];
            for (number, count) in map {
                println!("  {:0width$x}: {}", number, count, width = digits);
            }
        }
    }
}

struct BytesState {
    digits: usize,
    current_number: u64,
    current_digits: usize,
}

impl BytesState {
    fn new(digits: usize) -> Self {
        let current_number: u64 = 0;
        let current_digits = 0;

        BytesState {
            digits,
            current_number,
            current_digits,
        }
    }
}

pub struct Variant<T> {
    ctx: Context<T>,
    state: BytesState,
}

impl<T: CountStrat> Process for Variant<T> {
    fn new(digits: usize) -> Self {
        Self {
            ctx: Context::new(digits),
            state: BytesState::new(digits),
        }
    }
    fn on_byte(&mut self, b: u8) {
        self.ctx.count_digit(b, &mut self.state);
    }
    fn finalize(&mut self) {
        self.ctx.count_digit_end(&mut self.state);
        self.ctx.compute_sub_counts();
    }
    fn into_count(self) -> FastHashMap<Vec<u8>, Counter> {
        let mut map = FastHashMap::default();
        let Self { ctx, state: _ } = self;
        let digits = ctx.digits;

        for digits in 1..digits + 1 {
            let m = &ctx.count_maps[digits];

            for (number, count) in m {
                let mut number = *number;
                let mut vec = Vec::new();
                for _ in 0..digits {
                    let c = (number & 0xf) as u8;
                    let c = if c < 10 { b'0' + c } else { b'a' + (c - 10) };
                    vec.push(c);
                    number >>= 4;
                }
                vec.reverse();
                assert!(map.insert(vec, *count as Counter).is_none());
            }
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
        a.ctx.debug_output();
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
