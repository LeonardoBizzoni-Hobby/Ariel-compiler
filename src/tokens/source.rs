use std::fs::File;

use memmap2::Mmap;

use super::error::Error;

#[allow(dead_code)]
pub struct SourceFile {
    pub name: String,
    pub line: usize,
    pub column: usize,
    size: usize,

    pub start: usize,
    pub current: usize,
    pub mmap: Mmap,
}

impl SourceFile {
    pub fn new(path: &str) -> Result<Self, Error> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => return Err(Error::FileNotFound(path.to_owned(), e.to_string())),
        };

        let mmap = unsafe {
            match Mmap::map(&file) {
                Ok(map) => map,
                Err(e) => return Err(Error::MemoryMapFiled(path.to_owned(), e.to_string())),
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

    #[inline(always)]
    pub fn peek(&self) -> u8 {
        self.peek_at(0)
    }

    #[inline(always)]
    pub fn peek_next(&self) -> u8 {
        self.peek_at(1)
    }

    #[inline(always)]
    fn peek_at(&self, index: usize) -> u8 {
        match self.mmap.get(self.current + index) {
            Some(value) => *value,
            None => 0,
        }
    }
}
