use crate::{hart::Hart, uop::BinaryOp, utils::Maybe, xlen::XlenT};

#[cfg(feature = "A")]
use crate::uop::MemOrder;

/// holds state of memory subsystem  
#[derive(Debug, Clone, Default)]
pub struct Mem {
    /// is big endian
    be: bool,
    #[cfg(test)]
    pub hook_mem: Option<Box<[u8]>>,
}

impl<Xlen: XlenT> Hart<Xlen> {
    pub fn rd_mem8(&mut self, addr: Xlen) -> Maybe<u8> {
        todo!()
    }
    pub fn rd_mem16(&mut self, addr: Xlen) -> Maybe<u16> {
        todo!()
    }
    pub fn rd_mem32(&mut self, addr: Xlen) -> Maybe<u32> {
        #[cfg(test)]
        if let Ok(res) = self.mem.hook_rd(addr.into()) {
            return Ok(res);
        }
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
        #[cfg(test)]
        if let Ok(res) = self.mem.hook_wr(addr.into(), data) {
            return Ok(res);
        }
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
        #[cfg(test)]
        if let Ok(res) = self.mem.hook_rd(addr.into()) {
            return Ok(res);
        }
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

impl Mem {
    #[cfg(test)]
    fn hook_rd(&self, addr: usize) -> Maybe<u32> {
        if let Some(mem_region) = self.hook_mem.as_ref() {
            if addr + 4 > mem_region.len() {
                return Err(());
            }
            let mut buf = [0u8; 4];
            buf.copy_from_slice(&mem_region[addr..addr + 4]);
            return Ok(u32::from_le_bytes(buf));
        }
        Err(())
    }
    #[cfg(test)]
    fn hook_wr(&mut self, addr: usize, data: u32) -> Maybe<()> {
        if let Some(mem_region) = self.hook_mem.as_mut() {
            if addr + 4 > mem_region.len() {
                return Err(());
            }
            let buf = data.to_le_bytes();
            mem_region[addr..addr + 4].copy_from_slice(&buf);
            return Ok(())
        }
        Err(())
    }
}
