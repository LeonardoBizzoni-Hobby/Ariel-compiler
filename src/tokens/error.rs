#[derive(Debug)]
pub enum Error<'ctx> {
    FileNotFound(&'ctx str, String),
    MemoryMapFiled(&'ctx str, String),
    MissingSourceFileReference,
}
