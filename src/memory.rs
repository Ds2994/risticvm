pub trait Addressable {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, value: u8) -> bool;

    fn read2(&self, addr: u16) -> Option<u16> {
        if let Some(lower) = self.read(addr) {
            if let Some(upper) = self.read(addr + 1) {
                return Some((lower as u16) | ((upper as u16) << 8));
            }
        };
        None
    }

    fn write2(&mut self, addr: u16, value: u16) -> bool {
        let lower = value & 0xff;
        let upper = (value & 0xff00) >> 8;
        return self.write(addr, lower as u8) && self.write(addr + 1, upper as u8);
    }

    fn copy(&mut self, from: u16, to: u16, n: usize) -> bool {
        for i in 0..n {
            if let Some(x) = self.read(from + (i as u16)) {
                if !self.write(to + (i as u16), x) {
                    return false;
                }
            } else {
                return false;
            }
        }
        return true;
    }

    fn load_from_vec(&mut self, from: &[u8], addr: u16) -> bool {
        for (idx, data) in from.into_iter().enumerate() {
            if !self.write(addr + (idx as u16), *data) {
                return false;
            }
        };
        return true;
    }
}

pub struct LinearMemory {
    bytes: Vec<u8>,
    size: usize,
}

impl LinearMemory {
    pub fn new(n: usize) -> Self {
        Self {
            bytes: vec![0; n],
            size: n,
        }
    }
}

impl Addressable for LinearMemory {

    fn read(&self, addr: u16) -> Option<u8> {
        if (addr as usize) < self.size {
            return Some(self.bytes[addr as usize]);
        } else {
            return None;
        }
    }

    fn write(&mut self, addr: u16, value: u8) -> bool {
        if (addr as usize) < self.size {
            self.bytes[addr as usize] = value;
            return true;
        } else {
            return false;
        }
    }
}