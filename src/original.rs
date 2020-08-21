use crate::Process;
use std::collections::HashMap;

fn is_not_numeric(vec: &Vec<u8>) -> bool {
    for &c in vec {
        if !char::from(c).is_numeric() {
            return true;
        }
    }
    return false;
}

pub struct Original {
    count: HashMap<Vec<u8>, u128>,
    buffer: Vec<Vec<u8>>,
}

impl Process for Original {
    fn new(digit: usize) -> Self {
        let count: HashMap<Vec<u8>, u128> = HashMap::new();
        let mut buffer: Vec<Vec<u8>> = Vec::with_capacity(digit);
        for i in 0..digit {
            buffer.push(b"_".repeat(i + 1));
        }
        Self { count, buffer }
    }
    fn on_byte(&mut self, byte: u8) {
        for i in 0..(self.buffer.len()) {
            self.buffer[i].remove(0);
            self.buffer[i].push(byte);
            *self.count.entry(self.buffer[i].clone()).or_insert(0) += 1;
        }
    }
    fn finalize(&mut self) {
        for key in self.count.clone().keys() {
            if is_not_numeric(key) {
                self.count.remove(key);
            }
        }
    }
    fn into_count(self) -> HashMap<Vec<u8>, u128> {
        self.count
    }
}
