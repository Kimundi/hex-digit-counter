use std::collections::HashMap;

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

struct Context {
    count_maps: Vec<HashMap<u64, usize>>,
    digits: usize,
    masks: Vec<u64>,
}

type Number = u64;

impl Context {
    fn new(digits: usize) -> Self {
        assert!(std::mem::size_of::<Number>() * 2 >= digits);

        let mut count_maps: Vec<HashMap<Number, usize>> = Vec::new();
        for _number_width in 0..(digits + 1) {
            count_maps.push(HashMap::new());
        }
        dbg!(digits);

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
            masks,
        }
    }

    fn count_digits(&mut self, data: &[u8]) -> &mut Self {
        let digits = self.digits;

        let mut j = 0;
        let mut current_number: u64 = 0;
        let mut current_digits = 0;

        while j < data.len() {
            let v = DIGIT_MAP[data[j] as usize];

            if v == 0xff {
                if current_digits != 0 {
                    self.count_number_end(current_number, current_digits);
                    current_digits = 0;
                }
                j += 1;
                continue;
            }

            if current_digits == digits {
                self.count_number_mid(current_number, current_digits);
                current_digits -= 1;
            }

            current_number <<= 4;
            current_number |= v as u64;
            current_digits += 1;
            j += 1;
        }
        if current_digits != 0 {
            self.count_number_end(current_number, current_digits);
        }

        self
    }

    fn count_number(&mut self, v: u64, width: usize) {
        let mut v = v & self.masks[width];
        for width in (1..width + 1).rev() {
            *self.count_maps[width].entry(v).or_default() += 1;
            println!("  count {:0width$x}", v, width = width);
            v >>= 4;
        }
    }

    fn count_number_end(&mut self, v: u64, mut width: usize) {
        while width != 0 {
            self.count_number(v, width);
            width -= 1;
        }
        println!();
    }

    fn count_number_mid(&mut self, v: u64, mut width: usize) {
        self.count_number(v, width);
        println!();
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

fn main() {
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