use crate::{Counter, FastHashMap, Process};

// by @ehf

fn is_not_numeric<T: NumericType>(vec: &Vec<u8>) -> bool {
    for &c in vec {
        if !T::is_numeric(c) {
            return true;
        }
    }
    return false;
}

pub trait NumericType: Default {
    fn is_numeric(v: u8) -> bool;
}

#[derive(Default)]
pub struct StdNumeric;
impl NumericType for StdNumeric {
    fn is_numeric(v: u8) -> bool {
        char::from(v).is_numeric()
    }
}
#[derive(Default)]
pub struct HexDigit;
impl NumericType for HexDigit {
    fn is_numeric(v: u8) -> bool {
        (v >= b'0' && v <= b'9') || (v >= b'a' && v <= b'f') || (v >= b'A' && v <= b'F')
    }
}

pub struct Original<T> {
    count: FastHashMap<Vec<u8>, Counter>,
    buffer: Vec<Vec<u8>>,
    _numeric_type: T,
}

fn _debug_print(b: &[u8], i: usize) {
    println!("Buffer[{}] = [{}]", i, std::str::from_utf8(b).unwrap());
}

impl<T: NumericType> Process for Original<T> {
    fn new(digit: usize) -> Self {
        let count: FastHashMap<Vec<u8>, Counter> = FastHashMap::default();
        let mut buffer: Vec<Vec<u8>> = Vec::with_capacity(digit);
        for i in 0..digit {
            let b = b"_".repeat(i + 1);
            //_debug_print(&b, buffer.len());
            buffer.push(b);
        }

        let _numeric_type = T::default();

        Self {
            count,
            buffer,
            _numeric_type,
        }
    }
    fn on_byte(&mut self, byte: u8) {
        //println!("Byte '{}'", byte as char);
        for i in 0..(self.buffer.len()) {
            self.buffer[i].remove(0);
            self.buffer[i].push(byte);
            //_debug_print(&self.buffer[i], i);
            *self.count.entry(self.buffer[i].clone()).or_insert(0) += 1;
        }
    }
    fn finalize(&mut self) {
        for key in self.count.clone().keys() {
            if is_not_numeric::<T>(key) {
                self.count.remove(key);
            }
        }
    }
    fn into_count(self) -> FastHashMap<Vec<u8>, Counter> {
        self.count
    }
}
