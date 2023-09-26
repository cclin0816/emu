use crate::{uop::Instr, xlen::XlenT};

#[derive(Debug, Clone, Default)]
pub struct UopCache<Xlen: XlenT> {
    xlen: std::marker::PhantomData<Xlen>,
}

impl<Xlen: XlenT> UopCache<Xlen> {
    pub fn read(&self, addr: Xlen) -> Option<Instr> {
        None
    }
    pub fn alloc(&mut self, addr: Xlen, ins: Instr) {}
    pub fn flush(&mut self) {}
    pub fn flush_page(&mut self, addr: Xlen) {}
}
