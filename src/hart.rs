use crate::{decode::FrontEnd, memory::Mem, privilege::PrivCtrl, utils::Maybe, xlen::XlenT};

#[cfg(feature = "F")]
use crate::fpu::Fpu;

/// holds state of the hart
#[derive(Debug, Clone, Default)]
pub struct Hart<Xlen: XlenT> {
    /// general purpose registers
    pub gprs: [Xlen; 32],
    #[cfg(feature = "F")]
    pub fpu: Fpu,
    #[cfg(feature = "V")]
    pub vpu: Vpu,
    pub fe: FrontEnd<Xlen>,
    pub mem: Mem,
    pub priv_ctrl: PrivCtrl,
    /// program counter
    pub pc: Xlen,
    pub stop_tok: bool,
}

impl<Xlen: XlenT> Hart<Xlen> {
    pub fn new() -> Self {
        todo!()
    }
}
