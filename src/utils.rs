use crate::xlen::XlenT;
use std::ops::BitAnd;

pub type Maybe<T> = Result<T, ()>;

pub fn is_aligned<Xlen: XlenT>(addr: Xlen, align: u32) -> bool {
    addr & Xlen::from(align - 1) == Xlen::from(0)
}

pub fn flag_set<T>(val: T, mask: T) -> bool
where
    T: BitAnd<Output = T> + Eq + Copy,
{
    val & mask == mask
}
