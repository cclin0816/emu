use crate::{
    hart::Hart,
    uop::{Exception, Instr, MemProtect},
    utils::{is_aligned, Maybe},
    xlen::XlenT,
};

#[macro_use]
mod common;
mod cache;
#[cfg(feature = "C")]
mod dec16;
mod dec32;

#[derive(Debug, Clone, Default)]
pub struct FrontEnd<Xlen: XlenT> {
    isa: common::Isa<Xlen>,
    cache: cache::UopCache<Xlen>,
}

fn is_c_ins(ins: u16) -> bool {
    ins & 3 != 3
}
fn combine(low: u16, high: u16) -> u32 {
    ((high as u32) << 16) | low as u32
}

impl<Xlen: XlenT> Hart<Xlen> {
    pub fn fetch_uop(&mut self) -> Maybe<Instr> {
        let pc = self.get_pc();
        if let Some(ins) = self.fe.cache.read(pc) {
            return Ok(ins);
        }
        let ins = if is_aligned(pc, 4) {
            let ins = self.fetch_mem32(pc)?;
            let isa = &self.fe.isa;
            if is_c_ins(ins as u16) {
                if_ext_c!(isa, isa.dec16(ins as u16))
                    .unwrap_or_else(|_| Instr::Trap(Exception::IllegalInstr))
            } else {
                isa.dec32(ins)
            }
        } else {
            #[cfg(feature = "C")]
            if is_aligned(pc, 2) && self.fe.isa.C {
                self.fetch_uop_align2()?
            } else {
                self.fetch_uop_misalign()?
            }
            #[cfg(not(feature = "C"))]
            self.fetch_uop_misalign()?
        };
        self.fe.cache.alloc(pc, ins);
        Ok(ins)
    }
    #[cfg(feature = "C")]
    fn fetch_uop_align2(&mut self) -> Maybe<Instr> {
        let pc = self.get_pc();
        let ins = self.fetch_mem16(pc)?;
        Ok(if is_c_ins(ins) {
            self.fe.isa.dec16(ins)
        } else {
            // spec doesn't mandate atomicity of misaligned access
            // so seperate fetch is fine
            let high = self.fetch_mem16(pc.add(2))?;
            let ins = combine(ins, high);
            self.fe.isa.dec32(ins)
        })
    }
    fn fetch_uop_misalign(&mut self) -> Maybe<Instr> {
        self.fetch_check(self.get_pc())?;
        self.raise(Exception::AddrMisalign(MemProtect::X))?;
        Err(()) // unreachable
    }
}
