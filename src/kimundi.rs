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

const HEX_MASKS: &[u64; 17] = &[
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

struct Context {
    count_maps: Vec<FastHashMap<u64, Counter>>,
    digits: usize,
}

type Number = u64;

impl Context {
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

        Self { count_maps, digits }
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
            *self.count_maps[width].entry(v).or_default() += 1;
            //println!("  count {:0width$x}", v, width = width);
            v >>= 4;
        }
    }

    fn count_number_end(&mut self, v: u64, mut width: usize) {
        while width != 0 {
            self.count_number(v, width);
            width -= 1;
        }
        //println!();
    }

    fn count_number_mid(&mut self, v: u64, width: usize) {
        self.count_number(v, width);
        //println!();
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

pub struct Variant1 {
    ctx: Context,
    state: BytesState,
}

impl Process for Variant1 {
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

    impl Context {
        fn count_digits(&mut self, data: &[u8]) -> &mut Self {
            let mut state = BytesState::new(self.digits);

            let mut j = 0;
            while j < data.len() {
                self.count_digit(data[j], &mut state);
                j += 1;
            }
            self.count_digit_end(&mut state);

            self
        }

        fn compute_sub_counts(&mut self) -> &mut Self {
            self
        }

        fn output(&self) {
            for digits in (1..(self.digits + 1)).rev() {
                println!("Digit counts for width = {}", digits);
                let map = &self.count_maps[digits];
                for (number, count) in map {
                    println!("  {:0width$x}: {}", number, count, width = digits);
                }
            }
        }
    }

    #[test]
    fn test_main() {
        println!("-1----------------------------");
        Context::new(2)
            .count_digits(b"1234567890")
            .compute_sub_counts()
            .output();
        println!("-2----------------------------");
        Context::new(2)
            .count_digits(b"_1234_5678_90_7")
            .compute_sub_counts()
            .output();
        println!("-3----------------------------");
        Context::new(5)
            .count_digits(b"1234567890_ffff")
            .compute_sub_counts()
            .output();
        println!("-4----------------------------");
        Context::new(5)
            .count_digits(b"1_23_456_7890_abcde_f01234")
            .compute_sub_counts()
            .output();
        println!("-5----------------------------");
        Context::new(5)
            .count_digits(b"1_23_456_7890_abcde_f012341_23_456_123")
            .compute_sub_counts()
            .output();
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
