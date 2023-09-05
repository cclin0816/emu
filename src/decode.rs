use crate::hart::Hart;
use crate::micro_op::*;
use crate::utils::*;
use crate::xlen::XlenT;

impl<Xlen: XlenT> Hart<Xlen> {
    fn check_reg(&self, reg: u8) -> Maybe<u8> {
        #[cfg(feature = "E")]
        if self.isa.E && reg >= 16 {
            return Err(());
        }
        Ok(reg)
    }
    fn rd(&self, instr: u32) -> Maybe<u8> {
        self.check_reg(select_bits(instr, 11, 7) as u8)
    }
    fn rs1(&self, instr: u32) -> Maybe<u8> {
        self.check_reg(select_bits(instr, 19, 15) as u8)
    }
    fn rs2(&self, instr: u32) -> Maybe<u8> {
        self.check_reg(select_bits(instr, 24, 20) as u8)
    }
    fn op_sl_imm(&self, imm: i32, xlen: u32) -> Maybe<Ops> {
        let imm = imm as u32;
        if imm >= xlen {
            Err(())
        } else {
            Ok(Ops::Sll)
        }
    }
    fn op_sr_imm(&self, imm: &mut i32, xlen: u32) -> Maybe<Ops> {
        let tmp = *imm as u32;
        let shamt = tmp & !(1 << 10);
        *imm = shamt as i32;
        if shamt >= xlen {
            Err(())
        } else if tmp == shamt {
            Ok(Ops::Srl)
        } else {
            Ok(Ops::Sra)
        }
    }

    pub fn dec_16(instr: u16) {}

    pub fn dec_32(&self, instr: u32) -> Instr {
        match select_bits(instr, 6, 2) {
            0b0_0000 => self.dec_load(instr),
            #[cfg(feature = "F")]
            0b0_0001 => Err(()), // load-fp
            0b0_0011 => self.dec_misc_mem(instr),
            0b0_0100 => self.dec_op_imm(instr),
            0b0_0101 => self.dec_auipc(instr),
            #[cfg(feature = "RV64")]
            0b0_0110 => self.dec_op_imm_32(instr),
            0b0_1000 => self.dec_store(instr),
            #[cfg(feature = "F")]
            0b0_1001 => Err(()), // store-fp
            #[cfg(feature = "A")]
            0b0_1011 => self.dec_amo(instr),
            0b0_1100 => self.dec_op(instr),
            0b0_1101 => self.dec_lui(instr),
            #[cfg(feature = "RV64")]
            0b0_1110 => self.dec_op_32(instr),
            #[cfg(feature = "F")]
            0b1_0000 => Err(()), // madd
            #[cfg(feature = "F")]
            0b1_0001 => Err(()), // msub
            #[cfg(feature = "F")]
            0b1_0010 => Err(()), // nmsub
            #[cfg(feature = "F")]
            0b1_0011 => Err(()), // nmadd
            #[cfg(feature = "F")]
            0b1_0100 => Err(()), // op-fp
            0b1_1000 => self.dec_branch(instr),
            0b1_1001 => self.dec_jalr(instr),
            0b1_1011 => self.dec_jal(instr),
            0b1_1100 => self.dec_system(instr),
            _ => Err(()),
        }
        .unwrap_or(Instr::Invalid)
    }

    fn dec_load(&self, instr: u32) -> Maybe<Instr> {
        let rd = self.rd(instr)?;
        let rs1 = self.rs1(instr)?;
        let imm = i_imm(instr);
        let mem_width = match funct3(instr) {
            0b000 => MemWidth::B,
            0b001 => MemWidth::H,
            0b010 => MemWidth::W,
            #[cfg(feature = "RV64")]
            0b011 => {
                if Xlen::xlen() == 32 {
                    return Err(());
                } else {
                    MemWidth::D
                }
            }
            0b100 => MemWidth::BU,
            0b101 => MemWidth::HU,
            #[cfg(feature = "RV64")]
            0b110 => {
                if Xlen::xlen() == 32 {
                    return Err(());
                } else {
                    MemWidth::WU
                }
            }
            #[cfg(feature = "RV128")]
            0b111 => {
                if Xlen::xlen() != 128 {
                    return Err(());
                } else {
                    MemWidth::DU
                }
            }
            _ => return Err(()),
        };
        Ok(Instr::Load(rd, rs1, imm, mem_width))
    }
    fn dec_misc_mem(&self, instr: u32) -> Maybe<Instr> {
        match funct3(instr) {
            0b000 => {
                let fm = select_bits(instr, 31, 28) as u8;
                let pred_succ = select_bits(instr, 27, 20) as u8;
                Ok(Instr::MiscMem(MiscMemOps::Fence(fm, pred_succ)))
            }
            #[cfg(feature = "Zifencei")]
            0b001 => {
                if self.isa.Zifencei {
                    Ok(Instr::MiscMem(MiscMemOps::FenceI))
                } else {
                    Err(())
                }
            }
            // future 128
            _ => Err(()),
        }
    }
    fn dec_op_imm(&self, instr: u32) -> Maybe<Instr> {
        if instr >> 7 == 0 {
            return Ok(Instr::Nop);
        }
        let rd = self.rd(instr)?;
        let rs1 = self.rs1(instr)?;
        let mut imm = i_imm(instr);
        let op = match funct3(instr) {
            0b000 => Ops::Add,
            0b001 => self.op_sl_imm(imm, Xlen::xlen())?,
            0b010 => Ops::Slt,
            0b011 => Ops::SltU,
            0b100 => Ops::Xor,
            0b101 => self.op_sr_imm(&mut imm, Xlen::xlen())?,
            0b110 => Ops::Or,
            0b111 => Ops::And,
            _ => unreachable!(),
        };
        Ok(Instr::OpImm(rd, rs1, imm, op))
    }
    fn dec_auipc(&self, instr: u32) -> Maybe<Instr> {
        let rd = self.rd(instr)?;
        let imm = u_imm(instr);
        Ok(Instr::Auipc(rd, imm))
    }
    #[cfg(feature = "RV64")]
    fn dec_op_imm_32(&self, instr: u32) -> Maybe<Instr> {
        if Xlen::xlen() == 32 {
            return Err(());
        }
        let rd = self.rd(instr)?;
        let rs1 = self.rs1(instr)?;
        let mut imm = i_imm(instr);
        let op = match funct3(instr) {
            0b000 => Ops::AddW,
            0b001 => {
                self.op_sl_imm(imm, 32)?;
                Ops::SllW
            }
            0b101 => match self.op_sr_imm(&mut imm, 32)? {
                Ops::Srl => Ops::SrlW,
                Ops::Sra => Ops::SraW,
                _ => unreachable!(),
            },
            _ => return Err(()),
        };
        Ok(Instr::OpImm(rd, rs1, imm, op))
    }
    fn dec_store(&self, instr: u32) -> Maybe<Instr> {
        let rs1 = self.rs1(instr)?;
        let rs2 = self.rs2(instr)?;
        let imm = s_imm(instr);
        let mem_width = match funct3(instr) {
            0b000 => MemWidth::B,
            0b001 => MemWidth::H,
            0b010 => MemWidth::W,
            #[cfg(feature = "RV64")]
            0b011 => {
                if Xlen::xlen() == 32 {
                    return Err(());
                } else {
                    MemWidth::D
                }
            }
            // future 128
            _ => return Err(()),
        };
        Ok(Instr::Store(rs1, rs2, imm, mem_width))
    }
    #[cfg(feature = "A")]
    fn dec_amo(&self, instr: u32) -> Maybe<Instr> {
        if !self.isa.A {
            return Err(());
        }
        let rd = self.rd(instr)?;
        let rs1 = self.rs1(instr)?;
        let rs2 = self.rs2(instr)?;
        let mem_order = match select_bits(instr, 26, 25) {
            0b00 => MemOrder::Relaxed,
            0b01 => MemOrder::Release,
            0b10 => MemOrder::Acquire,
            0b11 => MemOrder::AcqRel,
            _ => unreachable!(),
        };
        let mem_width = match funct3(instr) {
            0b010 => MemWidth::W,
            #[cfg(feature = "RV64")]
            0b011 => {
                if Xlen::xlen() == 32 {
                    return Err(());
                } else {
                    MemWidth::D
                }
            }
            _ => return Err(()),
        };
        let op = match select_bits(instr, 31, 27) {
            0b00000 => Ops::Add,
            0b00001 => Ops::Second,
            0b00010 => {
                if rd != 0 {
                    return Err(());
                } else {
                    return Ok(Instr::LoadReserved(rd, rs1, mem_order, mem_width));
                }
            }
            0b00011 => return Ok(Instr::StoreConditional(rd, rs1, rs2, mem_order, mem_width)),
            0b00100 => Ops::Xor,
            0b01000 => Ops::Or,
            0b01100 => Ops::And,
            0b10000 => Ops::Min,
            0b10100 => Ops::Max,
            0b11000 => Ops::MinU,
            0b11100 => Ops::MaxU,
            _ => return Err(()),
        };
        Ok(Instr::Amo(rd, rs1, rs2, mem_order, mem_width, op))
    }
    fn dec_op(&self, instr: u32) -> Maybe<Instr> {
        let rd = self.rd(instr)?;
        let rs1 = self.rs1(instr)?;
        let rs2 = self.rs2(instr)?;
        let fn3 = funct3(instr);
        let fn7 = funct7(instr);
        let op = match fn7 {
            0b0000000 => match fn3 {
                0b000 => Ops::Add,
                0b001 => Ops::Sll,
                0b010 => Ops::Slt,
                0b011 => Ops::SltU,
                0b100 => Ops::Xor,
                0b101 => Ops::Srl,
                0b110 => Ops::Or,
                0b111 => Ops::And,
                _ => unreachable!(),
            },
            0b0100000 => match fn3 {
                0b000 => Ops::Sub,
                0b101 => Ops::Sra,
                _ => return Err(()),
            },
            #[cfg(feature = "M")]
            0b0000001 => {
                if !self.isa.M {
                    return Err(());
                }
                match fn3 {
                    0b000 => Ops::Mul,
                    0b001 => Ops::Mulh,
                    0b010 => Ops::MulhSU,
                    0b011 => Ops::MulhU,
                    0b100 => Ops::Div,
                    0b101 => Ops::DivU,
                    0b110 => Ops::Rem,
                    0b111 => Ops::RemU,
                    _ => unreachable!(),
                }
            }
            _ => return Err(()),
        };

        Ok(Instr::Op(rd, rs1, rs2, op))
    }
    fn dec_lui(&self, instr: u32) -> Maybe<Instr> {
        let rd = self.rd(instr)?;
        let imm = u_imm(instr);
        Ok(Instr::Lui(rd, imm))
    }
    fn dec_op_32(&self, instr: u32) -> Maybe<Instr> {
        if Xlen::xlen() != 32 {
            return Err(());
        }
        let rd = self.rd(instr)?;
        let rs1 = self.rs1(instr)?;
        let rs2 = self.rs2(instr)?;
        let fn3 = funct3(instr);
        let fn7 = funct7(instr);
        let op = match fn7 {
            0b0000000 => match fn3 {
                0b000 => Ops::AddW,
                0b001 => Ops::SllW,
                0b101 => Ops::SrlW,
                _ => return Err(()),
            },
            0b0100000 => match fn3 {
                0b000 => Ops::SubW,
                0b101 => Ops::SraW,
                _ => return Err(()),
            },
            #[cfg(feature = "M")]
            0b0000001 => {
                if !self.isa.M {
                    return Err(());
                }
                match fn3 {
                    0b000 => Ops::MulW,
                    0b100 => Ops::DivW,
                    0b101 => Ops::DivUW,
                    0b110 => Ops::RemW,
                    0b111 => Ops::RemUW,
                    _ => return Err(()),
                }
            }
            _ => return Err(()),
        };
        Ok(Instr::Op(rd, rs1, rs2, op))
    }
    fn dec_branch(&self, instr: u32) -> Maybe<Instr> {
        let rs1 = self.rs1(instr)?;
        let rs2 = self.rs2(instr)?;
        let imm = b_imm(instr);
        let cond = match funct3(instr) {
            0b000 => CmpCond::Eq,
            0b001 => CmpCond::Ne,
            0b100 => CmpCond::Lt,
            0b101 => CmpCond::Ge,
            0b110 => CmpCond::LtU,
            0b111 => CmpCond::GeU,
            _ => return Err(()),
        };
        Ok(Instr::Branch(rs1, rs2, imm, cond))
    }
    fn dec_jalr(&self, instr: u32) -> Maybe<Instr> {
        let rd = self.rd(instr)?;
        if funct3(instr) != 0 {
            return Err(());
        }
        let rs1 = self.rs1(instr)?;
        let imm = i_imm(instr);
        Ok(Instr::OpImm(rd, rs1, imm, Ops::Add))
    }
    fn dec_jal(&self, instr: u32) -> Maybe<Instr> {
        let rd = self.rd(instr)?;
        let imm = j_imm(instr);
        Ok(Instr::Jal(rd, imm))
    }
    fn dec_system(&self, instr: u32) -> Maybe<Instr> {
        let fn3 = funct3(instr);
        if fn3 == 0 {
            match instr >> 7 {
                0b0 => Ok(Instr::Ecall),
                0b10_0000_0000_0000 => Ok(Instr::Ebreak),
                _ => Err(()),
            }
        } else {
            #[cfg(feature = "Zicsr")]
            if self.isa.Zicsr {
                let rd = self.rd(instr)?;
                let uimm = select_bits(instr, 19, 15) as u8;
                let csr = select_bits(instr, 31, 20) as u16;
                let csr_op = match fn3 {
                    0b001 => CsrOp::Rw,
                    0b010 => CsrOp::Rs,
                    0b011 => CsrOp::Rc,
                    0b101 => CsrOp::Rwi,
                    0b110 => CsrOp::Rws,
                    0b111 => CsrOp::Rwc,
                    _ => return Err(()),
                };
                if fn3 < 0b100 {
                    self.check_reg(uimm)?;
                }
                Ok(Instr::Csr(rd, uimm, csr, csr_op))
            } else {
                Err(())
            }
            #[cfg(not(feature = "Zicsr"))]
            Err(())
        }
    }
}

fn funct3(instr: u32) -> u8 {
    select_bits(instr, 14, 12) as u8
}
fn funct7(instr: u32) -> u8 {
    select_bits(instr, 31, 25) as u8
}
fn sext(imm: u32, sign_bit: u32) -> i32 {
    let imm = (imm << (31 - sign_bit)) as i32;
    imm >> (31 - sign_bit)
}
fn i_imm(instr: u32) -> i32 {
    let imm = select_bits(instr, 31, 20);
    sext(imm, 11)
}
fn s_imm(instr: u32) -> i32 {
    let imm_11_5 = select_bits(instr, 31, 25);
    let imm_4_0 = select_bits(instr, 11, 7);
    let imm = (imm_11_5 << 5) | imm_4_0;
    sext(imm, 11)
}
fn b_imm(instr: u32) -> i32 {
    let imm_12 = select_bits(instr, 31, 31);
    let imm_10_5 = select_bits(instr, 30, 25);
    let imm_4_1 = select_bits(instr, 11, 8);
    let imm_11 = select_bits(instr, 7, 7);
    let imm = (imm_12 << 12) | (imm_11 << 11) | (imm_10_5 << 5) | (imm_4_1 << 1);
    sext(imm, 12)
}
fn u_imm(instr: u32) -> i32 {
    let imm = select_bits(instr, 31, 12);
    (imm << 12) as i32
}
fn j_imm(instr: u32) -> i32 {
    let imm_20 = select_bits(instr, 31, 31);
    let imm_10_1 = select_bits(instr, 30, 21);
    let imm_11 = select_bits(instr, 20, 20);
    let imm_19_12 = select_bits(instr, 19, 12);
    let imm = (imm_20 << 20) | (imm_19_12 << 12) | (imm_11 << 11) | (imm_10_1 << 1);
    sext(imm, 20)
}
