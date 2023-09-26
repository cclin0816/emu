use crate::{
    decode::FrontEnd,
    memory::Mem,
    privilege::PrivCtrl,
    utils::Maybe,
    xlen::{Cast, XlenT},
};

#[cfg(feature = "F")]
use crate::fpu::Fpu;

/// holds state of the hart
#[derive(Debug, Clone)]
pub struct Hart<Xlen: XlenT> {
    /// general purpose registers
    gprs: [Xlen; 32],
    #[cfg(feature = "F")]
    pub fpu: Fpu,
    #[cfg(feature = "V")]
    pub vpu: Vpu,
    pub fe: FrontEnd<Xlen>,
    pub mem: Mem,
    pub priv_ctrl: PrivCtrl,
    /// program counter
    pc: Xlen,
    pub stop_tok: bool,
}

impl<Xlen: XlenT> Hart<Xlen> {
    pub fn new() -> Self {
        todo!()
    }
    pub fn rd_gpr(&self, reg: u8) -> Xlen {
        if reg == 0 {
            Xlen::from(0)
        } else {
            self.gprs[reg as usize]
        }
    }
    pub fn wr_gpr(&mut self, reg: u8, val: Xlen) {
        self.gprs[reg as usize] = val;
    }
    pub fn advance_pc<T>(&mut self, offset: T) -> Maybe<()>
    where
        Xlen: Cast<T>,
    {
        self.pc = self.pc.add(offset);
        Err(())
    }
    pub fn set_pc(&mut self, addr: Xlen) -> Maybe<()> {
        self.pc = addr;
        Err(())
    }
    pub fn get_pc(&self) -> Xlen {
        self.pc
    }
}
