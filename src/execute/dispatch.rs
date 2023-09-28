use crate::{
    hart::Hart,
    uop::*,
    utils::Maybe,
    xlen::{Cast, XlenT},
};

#[cfg(feature = "F")]
use crate::fpu::Fpu;

impl Instr {
    pub fn exec<Xlen: XlenT>(self, hart: &mut Hart<Xlen>) -> Maybe<()> {
        match self {
            Instr::Undecoded => panic!("exec on undecoded"),
            Instr::Trap(reason) => hart.raise(reason),
            Instr::OpImm(rd, rs1, imm, op) => {
                hart.op_imm(rd, rs1, imm, op);
                hart.advance_pc(4)
            }
            Instr::Op(rd, rs1, rs2, op) => {
                hart.op(rd, rs1, rs2, op);
                hart.advance_pc(4)
            }
            Instr::Auipc(rd, imm) => {
                hart.wr_gpr(rd, hart.get_pc().add(imm));
                hart.advance_pc(4)
            }
            Instr::Load(rd, rs1, offset, width) => {
                hart.load(rd, rs1, offset, width)?;
                hart.advance_pc(4)
            }
            Instr::Store(rs1, rs2, offset, width) => {
                hart.store(rs1, rs2, offset, width)?;
                hart.advance_pc(4)
            }
            Instr::MiscMem(op) => {
                match op {
                    MiscMemOp::Fence(pred, succ) => hart.fence(pred, succ),
                    MiscMemOp::FenceTso => hart.fence_tso(),
                    #[cfg(feature = "Zifencei")]
                    MiscMemOp::FenceI => hart.fence_i(),
                }
                hart.advance_pc(4)
            }
            Instr::Branch(rs1, rs2, offset, cond) => hart.branch(rs1, rs2, offset, cond, 4),
            Instr::Jal(rd, offset) => hart.jal(rd, offset, 4),
            Instr::Jalr(rd, rs1, offset) => hart.jalr(rd, rs1, offset, 4),

            #[cfg(feature = "Zicsr")]
            Instr::Csr(rd, rs1_uimm, addr, op) => {
                let val = match op {
                    CsrOp::Rw | CsrOp::Rs | CsrOp::Rc => hart.rd_gpr(rs1_uimm),
                    CsrOp::Rwi | CsrOp::Rsi | CsrOp::Rci => Xlen::from(rs1_uimm),
                };
                let res = match op {
                    CsrOp::Rw | CsrOp::Rwi => hart.csr_wr(addr, val)?,
                    CsrOp::Rs | CsrOp::Rsi => hart.csr_set(addr, val)?,
                    CsrOp::Rc | CsrOp::Rci => hart.csr_clr(addr, val)?,
                };
                hart.wr_gpr(rd, res);
                hart.advance_pc(4)
            }

            #[cfg(feature = "A")]
            Instr::LoadReserved(rd, rs1, order, width) => {
                let addr = hart.rd_gpr(rs1);
                let res = match width {
                    MemWidth::W => Xlen::from(hart.load_rsrv32(addr, order)? as i32),
                    #[cfg(feature = "RV64")]
                    MemWidth::D => Xlen::from(hart.load_rsrv64(addr, order)? as i64),
                    _ => panic!("bad uop"),
                };
                hart.wr_gpr(rd, res);
                hart.advance_pc(4)
            }
            #[cfg(feature = "A")]
            Instr::StoreConditional(rd, rs1, rs2, order, width) => {
                let addr = hart.rd_gpr(rs1);
                let data = hart.rd_gpr(rs2);
                let res = match width {
                    MemWidth::W => Xlen::from(hart.store_cond32(addr, order, data.into())? as i32),
                    #[cfg(feature = "RV64")]
                    MemWidth::D => Xlen::from(hart.store_cond64(addr, order, data.into())? as i64),
                    _ => panic!("bad uop"),
                };
                hart.wr_gpr(rd, res);
                hart.advance_pc(4)
            }
            #[cfg(feature = "A")]
            Instr::Amo(rd, rs1, rs2, order, width, op) => {
                let addr = hart.rd_gpr(rs1);
                let data = hart.rd_gpr(rs2);
                let res = match width {
                    MemWidth::W => Xlen::from(hart.amo32(addr, order, data.into(), op)? as i32),
                    #[cfg(feature = "RV64")]
                    MemWidth::D => Xlen::from(hart.amo64(addr, order, data.into(), op)? as i64),
                    _ => panic!("bad uop"),
                };
                hart.wr_gpr(rd, res);
                hart.advance_pc(4)
            }

            #[cfg(feature = "F")]
            Instr::LoadFp(rd, rs1, offset, pr) => {
                hart.load_fp(rd, rs1, offset, pr)?;
                hart.advance_pc(4)
            }
            #[cfg(feature = "F")]
            Instr::StoreFp(rs1, rs2, offset, pr) => {
                hart.store_fp(rs1, rs2, offset, pr)?;
                hart.advance_pc(4)
            }
            #[cfg(feature = "F")]
            Instr::FpOp3(rd, rs1, rs2, rs3, rm, pr, op) => {
                hart.set_rt_rm(rm)?;
                hart.fpu.ternary_op(rd, rs1, rs2, rs3, pr, op);
                hart.advance_pc(4)
            }
            #[cfg(feature = "F")]
            Instr::FpOp2(rd, rs1, rs2, rm, pr, op) => {
                hart.set_rt_rm(rm)?;
                hart.fpu.binary_op(rd, rs1, rs2, pr, op);
                hart.advance_pc(4)
            }
            #[cfg(feature = "F")]
            Instr::FpOp(rd, rs1, rm, pr, op) => {
                hart.set_rt_rm(rm)?;
                hart.fpu.unary_op(rd, rs1, pr, op);
                hart.advance_pc(4)
            }
            #[cfg(feature = "F")]
            Instr::FpCvtGp(rd, rs1, rm, pr, op) => {
                hart.set_rt_rm(rm)?;
                let val = hart.fpu.fp_cvt_gp(rs1, pr, op);
                hart.wr_gpr(rd, val);
                hart.advance_pc(4)
            }
            #[cfg(feature = "F")]
            Instr::GpCvtFp(rd, rs1, rm, pr, op) => {
                hart.set_rt_rm(rm)?;
                let rs1 = hart.rd_gpr(rs1);
                hart.fpu.gp_cvt_fp(rd, rs1, pr, op);
                hart.advance_pc(4)
            }
            #[cfg(feature = "F")]
            Instr::FpCmp(rd, rs1, rs2, pr, op) => {
                let val = hart.fpu.fp_cmp(rs1, rs2, pr, op);
                hart.wr_gpr(rd, Xlen::from(val));
                hart.advance_pc(4)
            }
            #[cfg(feature = "D")]
            Instr::FpCvtFp(rd, rs1, rm, from, to) => {
                hart.set_rt_rm(rm)?;
                hart.fpu.fp_cvt_fp(rd, rs1, from, to);
                hart.advance_pc(4)
            }

            #[cfg(feature = "C")]
            Instr::COpImm(rd, rs1, imm, op) => {
                hart.op_imm(rd, rs1, imm, op);
                hart.advance_pc(2)
            }
            #[cfg(feature = "C")]
            Instr::COp(rd_rs1, rs2, op) => {
                hart.op(rd_rs1, rd_rs1, rs2, op);
                hart.advance_pc(2)
            }
            #[cfg(feature = "C")]
            Instr::CLoad(rd, rs1, offset, width) => {
                hart.load(rd, rs1, offset, width)?;
                hart.advance_pc(2)
            }
            #[cfg(feature = "C")]
            Instr::CStore(rs1, rs2, offset, width) => {
                hart.store(rs1, rs2, offset, width)?;
                hart.advance_pc(2)
            }
            #[cfg(feature = "C")]
            Instr::CBranch(rs1, offset, cond) => hart.branch(rs1, 0, offset, cond, 2),
            #[cfg(feature = "C")]
            Instr::CJal(rd, offset) => hart.jal(rd, offset, 2),
            #[cfg(feature = "C")]
            Instr::CJalr(rd, rs1) => hart.jalr(rd, rs1, 0, 2),
            #[cfg(all(feature = "C", feature = "F"))]
            Instr::CLoadFp(rd, rs1, offset, pr) => {
                hart.load_fp(rd, rs1, offset, pr)?;
                hart.advance_pc(2)
            }
            #[cfg(all(feature = "C", feature = "F"))]
            Instr::CStoreFp(rs1, rs2, offset, pr) => {
                hart.store_fp(rs1, rs2, offset, pr)?;
                hart.advance_pc(2)
            }
        }
    }
}

impl<Xlen: XlenT> Hart<Xlen> {
    fn rd_gpr(&self, reg: u8) -> Xlen {
        if reg == 0 {
            Xlen::from(0)
        } else {
            self.gprs[reg as usize]
        }
    }
    fn wr_gpr(&mut self, reg: u8, val: Xlen) {
        self.gprs[reg as usize] = val;
    }
    fn advance_pc<T>(&mut self, offset: T) -> Maybe<()>
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
    fn op_imm(&mut self, rd: u8, rs1: u8, imm: i32, op: BinaryOp) {
        let lhs = self.rd_gpr(rs1);
        let rhs = Xlen::from(imm);
        let res = op.exec(lhs, rhs);
        self.wr_gpr(rd, res);
    }
    fn op(&mut self, rd: u8, rs1: u8, rs2: u8, op: BinaryOp) {
        let lhs = self.rd_gpr(rs1);
        let rhs = self.rd_gpr(rs2);
        let res = op.exec(lhs, rhs);
        self.wr_gpr(rd, res);
    }
    fn branch(&mut self, rs1: u8, rs2: u8, offset: i32, cond: CmpCond, step: u8) -> Maybe<()> {
        let lhs = self.rd_gpr(rs1);
        let rhs = self.rd_gpr(rs2);
        if cond.test(lhs, rhs) {
            self.advance_pc(offset)
        } else {
            self.advance_pc(step)
        }
    }
    fn jal(&mut self, rd: u8, offset: i32, step: u8) -> Maybe<()> {
        self.wr_gpr(rd, self.get_pc().add(step));
        self.advance_pc(offset)
    }
    fn jalr(&mut self, rd: u8, rs1: u8, offset: i32, step: u8) -> Maybe<()> {
        let addr = self.rd_gpr(rs1).add(offset);
        self.wr_gpr(rd, self.get_pc().add(step));
        self.set_pc(addr)
    }
    fn load(&mut self, rd: u8, rs1: u8, offset: i32, width: MemWidth) -> Maybe<()> {
        let addr = self.rd_gpr(rs1).add(offset);
        let res = match width {
            MemWidth::B => Xlen::from(self.rd_mem8(addr)? as i8),
            MemWidth::H => Xlen::from(self.rd_mem16(addr)? as i16),
            MemWidth::W => Xlen::from(self.rd_mem32(addr)? as i32),
            #[cfg(feature = "RV64")]
            MemWidth::D => Xlen::from(self.rd_mem64(addr)? as i64),
            MemWidth::BU => Xlen::from(self.rd_mem8(addr)?),
            MemWidth::HU => Xlen::from(self.rd_mem16(addr)?),
            #[cfg(feature = "RV64")]
            MemWidth::WU => Xlen::from(self.rd_mem32(addr)?),
        };
        self.wr_gpr(rd, res);
        Ok(())
    }
    fn store(&mut self, rs1: u8, rs2: u8, offset: i32, width: MemWidth) -> Maybe<()> {
        let addr = self.rd_gpr(rs1).add(offset);
        let data = self.rd_gpr(rs2);
        match width {
            MemWidth::B => self.wr_mem8(addr, data.into()),
            MemWidth::H => self.wr_mem16(addr, data.into()),
            MemWidth::W => self.wr_mem32(addr, data.into()),
            #[cfg(feature = "RV64")]
            MemWidth::D => self.wr_mem64(addr, data.into()),
            _ => panic!("bad uop"),
        }
    }
    #[cfg(feature = "F")]
    fn load_fp(&mut self, rd: u8, rs1: u8, offset: i32, pr: Precision) -> Maybe<()> {
        let addr = self.rd_gpr(rs1).add(offset);
        match pr {
            Precision::S => {
                let val = self.rd_mem32(addr)?;
                self.fpu.u32_mv_f32(rd, val);
            }
            #[cfg(feature = "D")]
            Precision::D => {
                let val = self.rd_mem64(addr)?;
                self.fpu.u64_mv_f64(rd, val);
            }
        };
        Ok(())
    }
    #[cfg(feature = "F")]
    fn store_fp(&mut self, rs1: u8, rs2: u8, offset: i32, pr: Precision) -> Maybe<()> {
        let addr = self.rd_gpr(rs1).add(offset);
        match pr {
            Precision::S => {
                let data = self.fpu.f32_mv_u32(rs2);
                self.wr_mem32(addr, data)
            }
            #[cfg(feature = "D")]
            Precision::D => {
                let data = self.fpu.f64_mv_u64(rs2);
                self.wr_mem64(addr, data)
            }
        }
    }
}
