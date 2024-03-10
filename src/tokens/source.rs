use std::fs::File;

use memmap2::Mmap;

use super::error::Error;

#[allow(dead_code)]
pub struct Source<'lexer> {
    pub line: usize,
    pub column: usize,
    pub start: usize,
    pub current: usize,
    pub finished: bool,
    pub name: &'lexer str,
    pub mmap: Mmap,
}

impl<'lexer> Source<'lexer> {
    pub fn new(path: &'lexer str) -> Result<Self, Error> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => return Err(Error::FileNotFound(path, e.to_string())),
        };

        let mmap = unsafe {
            match Mmap::map(&file) {
                Ok(map) => map,
                Err(e) => return Err(Error::MemoryMapFiled(path, e.to_string())),
            }
        };

        Ok(Self {
            line: 1,
            column: 0,
            start: 0,
            current: 0,
            finished: false,
            name: path,
            mmap,
        })
    }

    pub fn is_at_eof(&self) -> bool {
        self.current >= self.mmap.len()
    }

    pub fn peek(&self) -> u8 {
        match self.mmap.get(self.current) {
            Some(byte) => *byte,
            None => 0x00,
        }
    }

    pub fn peek_ahead(&self, n: usize) -> u8 {
        match self.mmap.get(self.current + n) {
            Some(byte) => *byte,
            None => 0x00,
        }
    }
}
