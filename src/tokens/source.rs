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
    mmap: Mmap,
}

impl SourceFile {
    pub fn new(path: &str) -> Result<Self, Error> {
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
            size: mmap.len(),
            start: 0,
            current: 0,
            name: path.to_string(),
            mmap,
        })
    }

    #[inline]
    pub fn build_lexeme(&self) -> String {
        String::from_utf8_lossy(&self.mmap[self.start..self.current]).to_string()
    }

    #[inline]
    pub fn peek(&self) -> u8 {
        self.peek_at(0)
    }

    #[inline]
    pub fn peek_next(&self) -> u8 {
        self.peek_at(1)
    }

    #[inline]
    fn peek_at(&self, index: usize) -> u8 {
        match self.mmap.get(self.current + index) {
            Some(value) => *value,
            None => 0,
        }
    }
}
