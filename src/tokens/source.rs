use std::fs::File;

use memmap2::Mmap;

use super::error::Error;

#[allow(dead_code)]
pub struct Source {
    pub name: String,
    pub line: usize,
    pub column: usize,
    size: usize,

    pub start: usize,
    pub current: usize,
    pub mmap: Mmap,
}

impl Source {
    pub fn new(path: &str) -> Result<Self, Error> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => return Err(Error::FileNotFound(path.to_string(), e.to_string())),
        };

        let mmap = unsafe {
            match Mmap::map(&file) {
                Ok(map) => map,
                Err(e) => return Err(Error::MemoryMapFiled(path.to_string(), e.to_string())),
            }
        };

        Ok(Self {
            line: 1,
            column: 0,
            size: mmap.len(),
            start: 0,
            current: 0,
            name: path.to_string(),
            mmap,
        })
    }

    pub fn is_at_eof(&self) -> bool {
        self.current >= self.size
    }

    pub fn peek(&self) -> u8 {
        if !self.is_at_eof() {
            self.mmap[self.current]
        } else {
            0
        }
    }

    pub fn peek_next(&self) -> u8 {
        if self.current + 1 < self.size {
            self.mmap[self.current + 1]
        } else {
            0
        }
    }
}
