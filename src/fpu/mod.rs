use crate::xlen::XlenT;

#[derive(Debug, Clone)]
pub struct Fpu<Xlen: XlenT> {
    pd: std::marker::PhantomData<Xlen>,
}
