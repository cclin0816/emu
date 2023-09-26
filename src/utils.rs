use crate::xlen::XlenT;

pub type Maybe<T> = Result<T, ()>;

pub fn is_aligned<Xlen: XlenT>(addr: Xlen, align: u32) -> bool {
    addr & Xlen::from(align - 1) == Xlen::from(0)
}
