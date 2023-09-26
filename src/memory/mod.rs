use crate::{hart::Hart, uop::BinaryOp, utils::Maybe, xlen::XlenT};

#[cfg(feature = "A")]
use crate::uop::MemOrder;

/// holds state of memory subsystem  
#[derive(Debug, Clone)]
pub struct Mem {
    /// is big endian
    be: bool,
}

impl<Xlen: XlenT> Hart<Xlen> {
    pub fn rd_mem8(&mut self, addr: Xlen) -> Maybe<u8> {
        todo!()
    }
    pub fn rd_mem16(&mut self, addr: Xlen) -> Maybe<u16> {
        todo!()
    }
    pub fn rd_mem32(&mut self, addr: Xlen) -> Maybe<u32> {
        todo!()
    }
    #[cfg(any(feature = "RV64", feature = "D"))]
    pub fn rd_mem64(&mut self, addr: Xlen) -> Maybe<u64> {
        todo!()
    }
    pub fn wr_mem8(&mut self, addr: Xlen, data: u8) -> Maybe<()> {
        todo!()
    }
    pub fn wr_mem16(&mut self, addr: Xlen, data: u16) -> Maybe<()> {
        todo!()
    }
    pub fn wr_mem32(&mut self, addr: Xlen, data: u32) -> Maybe<()> {
        todo!()
    }
    #[cfg(any(feature = "RV64", feature = "D"))]
    pub fn wr_mem64(&mut self, addr: Xlen, data: u64) -> Maybe<()> {
        todo!()
    }
    /// assume align 2
    pub fn fetch_mem16(&mut self, addr: Xlen) -> Maybe<u16> {
        todo!()
    }
    /// assume align 4
    pub fn fetch_mem32(&mut self, addr: Xlen) -> Maybe<u32> {
        todo!()
    }
    /// page fault & access fault have higher priority then misalign
    pub fn fetch_check(&mut self, addr: Xlen) -> Maybe<()> {
        todo!()
    }

    #[cfg(feature = "A")]
    pub fn load_rsrv32(&mut self, addr: Xlen, ord: MemOrder) -> Maybe<u32> {
        todo!()
    }
    #[cfg(all(feature = "A", feature = "RV64"))]
    pub fn load_rsrv64(&mut self, addr: Xlen, ord: MemOrder) -> Maybe<u64> {
        todo!()
    }
    #[cfg(feature = "A")]
    pub fn store_cond32(&mut self, addr: Xlen, ord: MemOrder, data: u32) -> Maybe<u32> {
        todo!()
    }
    #[cfg(all(feature = "A", feature = "RV64"))]
    pub fn store_cond64(&mut self, addr: Xlen, ord: MemOrder, data: u64) -> Maybe<u64> {
        todo!()
    }
    #[cfg(feature = "A")]
    pub fn amo32(&mut self, addr: Xlen, ord: MemOrder, data: u32, op: BinaryOp) -> Maybe<u32> {
        todo!()
    }
    #[cfg(all(feature = "A", feature = "RV64"))]
    pub fn amo64(&mut self, addr: Xlen, ord: MemOrder, data: u64, op: BinaryOp) -> Maybe<u64> {
        todo!()
    }

    pub fn fence(&mut self, pred: u8, succ: u8) {}
    pub fn fence_tso(&mut self) {}
    #[cfg(feature = "Zifencei")]
    pub fn fence_i(&mut self) {}
}
