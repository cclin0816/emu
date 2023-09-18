use crate::{decode::common::*, hart::HartIsa, uop::*, utils::Maybe, xlen::XlenT};

fn fn3(ins: u32) -> u8 {
    select_bits(ins, 14, 12) as u8
}

fn fn7(ins: u32) -> u8 {
    select_bits(ins, 31, 25) as u8
}

fn fn2(ins: u32) -> u8 {
    select_bits(ins, 26, 25) as u8
}

fn i_imm(ins: u32) -> i32 {
    let imm = select_bits(ins, 31, 20);
    sext(imm, 11)
}

fn s_imm(ins: u32) -> i32 {
    let imm = shuffle_bits!(ins, 0, 11, 7, 31, 25);
    sext(imm, 11)
}

fn b_imm(ins: u32) -> i32 {
    let imm = shuffle_bits!(ins, 1, 11, 8, 30, 25, 7, 7, 31, 31);
    sext(imm, 12)
}

fn u_imm(ins: u32) -> i32 {
    let imm = select_bits(ins, 31, 12);
    (imm << 12) as i32
}

fn j_imm(ins: u32) -> i32 {
    let imm = shuffle_bits!(ins, 1, 30, 21, 20, 20, 19, 12, 31, 31);
    sext(imm, 20)
}

fn rd(ins: u32) -> u8 {
    select_bits(ins, 11, 7) as u8
}

fn rs1(ins: u32) -> u8 {
    select_bits(ins, 19, 15) as u8
}

fn rs2(ins: u32) -> u8 {
    select_bits(ins, 24, 20) as u8
}

fn rs3(ins: u32) -> u8 {
    select_bits(ins, 31, 27) as u8
}
/// (rd, funct3, rs1, rs2, funct7)
fn r_type(ins: u32) -> (u8, u8, u8, u8, u8) {
    (rd(ins), fn3(ins), rs1(ins), rs2(ins), fn7(ins))
}
/// (rd, funct3, rs1, imm)
fn i_type(ins: u32) -> (u8, u8, u8, i32) {
    (rd(ins), fn3(ins), rs1(ins), i_imm(ins))
}
/// (funct3, rs1, rs2, imm)
fn s_type(ins: u32) -> (u8, u8, u8, i32) {
    (fn3(ins), rs1(ins), rs2(ins), s_imm(ins))
}
/// (funct3, rs1, rs2, imm)
fn b_type(ins: u32) -> (u8, u8, u8, i32) {
    (fn3(ins), rs1(ins), rs2(ins), b_imm(ins))
}
/// (rd, imm)
fn u_type(ins: u32) -> (u8, i32) {
    (rd(ins), u_imm(ins))
}
/// (rd, imm)
fn j_type(ins: u32) -> (u8, i32) {
    (rd(ins), j_imm(ins))
}
/// (rd, funct3, rs1, rs2, funct2, rs3 / funct5)
fn r4_type(ins: u32) -> (u8, u8, u8, u8, u8, u8) {
    (rd(ins), fn3(ins), rs1(ins), rs2(ins), fn2(ins), rs3(ins))
}

fn sl_imm(imm: i32, xlen: u32) -> Maybe<BinaryOp> {
    let imm = imm as u32;
    if imm >= xlen {
        Err(())
    } else {
        Ok(BinaryOp::Sll)
    }
}

fn sr_imm(imm: &mut i32, xlen: u32) -> Maybe<BinaryOp> {
    let tmp = *imm as u32;
    let shamt = tmp & !(1 << 10);
    *imm = shamt as i32;
    if shamt >= xlen {
        Err(())
    } else if tmp == shamt {
        Ok(BinaryOp::Srl)
    } else {
        Ok(BinaryOp::Sra)
    }
}

#[cfg(feature = "F")]
fn round_mode(fn3: u8) -> Maybe<RoundMode> {
    Ok(match fn3 {
        0b000 => RoundMode::Rne,
        0b001 => RoundMode::Rtz,
        0b010 => RoundMode::Rdn,
        0b011 => RoundMode::Rup,
        0b100 => RoundMode::Rmm,
        0b111 => RoundMode::Dyn,
        _ => return Err(()),
    })
}

impl<Xlen: XlenT, const EMB: bool> HartIsa<Xlen, EMB> {
    fn dec32_load(ins: u32) -> Maybe<Instr> {
        let (rd, fn3, rs1, imm) = i_type(ins);
        check_gp_regs!(rd, rs1);
        let mem_width = match fn3 {
            0b000 => MemWidth::B,
            0b001 => MemWidth::H,
            0b010 => MemWidth::W,
            0b011 => if_ge_rv64!(MemWidth::D)?,
            0b100 => MemWidth::BU,
            0b101 => MemWidth::HU,
            0b110 => if_ge_rv64!(MemWidth::WU)?,
            _ => unreachable!(),
        };
        Ok(Instr::Load(rd, rs1, imm, mem_width))
    }

    /// load store float precision
    #[cfg(feature = "F")]
    fn dec32_ls_pr(&self, fmt: u8) -> Maybe<Precision> {
        match fmt {
            0b010 => Ok(Precision::S),
            0b011 => if_ext_d!(self, Precision::D),
            _ => Err(()),
        }
    }

    fn dec32_load_fp(&self, ins: u32) -> Maybe<Instr> {
        if_ext_f!(self, {
            let (rd, fn3, rs1, imm) = i_type(ins);
            check_gp_regs!(rs1);
            let pr = self.dec32_ls_pr(fn3)?;
            Instr::LoadFp(rd, rs1, imm, pr)
        })
    }

    fn dec32_misc_mem(&self, ins: u32) -> Maybe<Instr> {
        match fn3(ins) {
            0b000 => {
                let pred = select_bits(ins, 27, 24) as u8;
                let succ = select_bits(ins, 23, 20) as u8;
                let fm = select_bits(ins, 31, 28);
                let fm = match (fm, pred, succ) {
                    (0b0000, _, _) => FenceMode::Normal,
                    (0b1000, 3, 3) => FenceMode::Tso,
                    // Base implementations shall treat all such reserved
                    // configurations as normal fences
                    _ => FenceMode::Normal,
                };
                Ok(Instr::MiscMem(MiscMemOp::Fence(pred, succ, fm)))
            }
            0b001 => if_ext_zifencei!(self, Instr::MiscMem(MiscMemOp::FenceI)),
            _ => Err(()),
        }
    }

    fn dec32_op_imm(ins: u32) -> Maybe<Instr> {
        let (rd, fn3, rs1, mut imm) = i_type(ins);
        check_gp_regs!(rd, rs1);
        let op = match fn3 {
            0b000 => BinaryOp::Add,
            0b001 => sl_imm(imm, Xlen::XLEN)?,
            0b010 => BinaryOp::Slt,
            0b011 => BinaryOp::SltU,
            0b100 => BinaryOp::Xor,
            0b101 => sr_imm(&mut imm, Xlen::XLEN)?,
            0b110 => BinaryOp::Or,
            0b111 => BinaryOp::And,
            _ => unreachable!(),
        };
        Ok(Instr::OpImm(rd, rs1, imm, op))
    }

    fn dec32_auipc(ins: u32) -> Maybe<Instr> {
        let (rd, imm) = u_type(ins);
        check_gp_regs!(rd);
        Ok(Instr::Auipc(rd, imm))
    }

    fn dec32_op_imm_32(ins: u32) -> Maybe<Instr> {
        if_ge_rv64!({
            let (rd, fn3, rs1, mut imm) = i_type(ins);
            check_gp_regs!(rd, rs1);
            let op = match fn3 {
                0b000 => BinaryOp::AddW,
                0b001 => {
                    sl_imm(imm, 32)?;
                    BinaryOp::SllW
                }
                0b101 => match sr_imm(&mut imm, 32)? {
                    BinaryOp::Srl => BinaryOp::SrlW,
                    BinaryOp::Sra => BinaryOp::SraW,
                    _ => unreachable!(),
                },
                _ => return Err(()),
            };
            Instr::OpImm(rd, rs1, imm, op)
        })
    }

    fn dec32_store(ins: u32) -> Maybe<Instr> {
        let (fn3, rs1, rs2, imm) = s_type(ins);
        check_gp_regs!(rs1, rs2);
        let mem_width = match fn3 {
            0b000 => MemWidth::B,
            0b001 => MemWidth::H,
            0b010 => MemWidth::W,
            0b011 => if_ge_rv64!(MemWidth::D)?,
            _ => return Err(()),
        };
        Ok(Instr::Store(rs1, rs2, imm, mem_width))
    }

    fn dec32_store_fp(&self, ins: u32) -> Maybe<Instr> {
        if_ext_f!(self, {
            let (fn3, rs1, rs2, imm) = s_type(ins);
            check_gp_regs!(rs1);
            let pr = self.dec32_ls_pr(fn3)?;
            Instr::StoreFp(rs1, rs2, imm, pr)
        })
    }

    fn dec32_amo(&self, ins: u32) -> Maybe<Instr> {
        if_ext_a!(self, {
            let (rd, fn3, rs1, rs2, fn2, fn5) = r4_type(ins);
            check_gp_regs!(rd, rs1, rs2);
            let mem_width = match fn3 {
                0b010 => MemWidth::W,
                0b011 => if_ge_rv64!(MemWidth::D)?,
                _ => return Err(()),
            };
            let mem_order = match fn2 {
                0b00 => MemOrder::Relaxed,
                0b01 => MemOrder::Release,
                0b10 => MemOrder::Acquire,
                0b11 => MemOrder::AcqRel,
                _ => unreachable!(),
            };
            let op = match fn5 {
                0b00000 => BinaryOp::Add,
                0b00001 => BinaryOp::Second,
                0b00010 => {
                    if rs2 != 0 {
                        return Err(());
                    } else {
                        return Ok(Instr::LoadReserved(rd, rs1, mem_order, mem_width));
                    }
                }
                0b00011 => return Ok(Instr::StoreConditional(rd, rs1, rs2, mem_order, mem_width)),
                0b00100 => BinaryOp::Xor,
                0b01000 => BinaryOp::Or,
                0b01100 => BinaryOp::And,
                0b10000 => BinaryOp::Min,
                0b10100 => BinaryOp::Max,
                0b11000 => BinaryOp::MinU,
                0b11100 => BinaryOp::MaxU,
                _ => return Err(()),
            };
            Instr::Amo(rd, rs1, rs2, mem_order, mem_width, op)
        })
    }

    fn dec32_op(&self, ins: u32) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, fn7) = r_type(ins);
        check_gp_regs!(rd, rs1, rs2);
        let op = match fn7 {
            0b0000000 => match fn3 {
                0b000 => BinaryOp::Add,
                0b001 => BinaryOp::Sll,
                0b010 => BinaryOp::Slt,
                0b011 => BinaryOp::SltU,
                0b100 => BinaryOp::Xor,
                0b101 => BinaryOp::Srl,
                0b110 => BinaryOp::Or,
                0b111 => BinaryOp::And,
                _ => unreachable!(),
            },
            0b0100000 => match fn3 {
                0b000 => BinaryOp::Sub,
                0b101 => BinaryOp::Sra,
                _ => return Err(()),
            },
            0b0000001 => if_ext_m!(self, {
                match fn3 {
                    0b000 => BinaryOp::Mul,
                    0b001 => BinaryOp::Mulh,
                    0b010 => BinaryOp::MulhSU,
                    0b011 => BinaryOp::MulhU,
                    0b100 => BinaryOp::Div,
                    0b101 => BinaryOp::DivU,
                    0b110 => BinaryOp::Rem,
                    0b111 => BinaryOp::RemU,
                    _ => unreachable!(),
                }
            })?,
            _ => return Err(()),
        };
        Ok(Instr::Op(rd, rs1, rs2, op))
    }

    fn dec32_lui(ins: u32) -> Maybe<Instr> {
        let (rd, imm) = u_type(ins);
        check_gp_regs!(rd);
        Ok(Instr::OpImm(rd, 0, imm, BinaryOp::Add))
    }

    fn dec32_op_32(&self, ins: u32) -> Maybe<Instr> {
        if_ge_rv64!({
            let (rd, fn3, rs1, rs2, fn7) = r_type(ins);
            check_gp_regs!(rd, rs1, rs2);
            let op = match fn7 {
                0b0000000 => match fn3 {
                    0b000 => BinaryOp::AddW,
                    0b001 => BinaryOp::SllW,
                    0b101 => BinaryOp::SrlW,
                    _ => return Err(()),
                },
                0b0100000 => match fn3 {
                    0b000 => BinaryOp::SubW,
                    0b101 => BinaryOp::SraW,
                    _ => return Err(()),
                },
                0b0000001 => if_ext_m!(self, {
                    match fn3 {
                        0b000 => BinaryOp::MulW,
                        0b100 => BinaryOp::DivW,
                        0b101 => BinaryOp::DivUW,
                        0b110 => BinaryOp::RemW,
                        0b111 => BinaryOp::RemUW,
                        _ => return Err(()),
                    }
                })?,
                _ => return Err(()),
            };
            Instr::Op(rd, rs1, rs2, op)
        })
    }

    #[cfg(feature = "F")]
    fn dec32_fp_pr(&self, fmt: u8) -> Maybe<Precision> {
        match fmt {
            0b00 => Ok(Precision::S),
            0b01 => if_ext_d!(self, Precision::D),
            _ => Err(()),
        }
    }

    #[cfg(feature = "F")]
    fn dec32_fp_op3(&self, ins: u32, op: FpTernaryOp) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, fn2, rs3) = r4_type(ins);
        let rm = round_mode(fn3)?;
        let pr = self.dec32_fp_pr(fn2)?;
        Ok(Instr::FpOp3(rd, rs1, rs2, rs3, rm, pr, op))
    }

    fn dec32_madd(&self, ins: u32) -> Maybe<Instr> {
        if_ext_f!(self, self.dec32_fp_op3(ins, FpTernaryOp::MAdd)?)
    }

    fn dec32_msub(&self, ins: u32) -> Maybe<Instr> {
        if_ext_f!(self, self.dec32_fp_op3(ins, FpTernaryOp::MSub)?)
    }

    fn dec32_nmsub(&self, ins: u32) -> Maybe<Instr> {
        if_ext_f!(self, self.dec32_fp_op3(ins, FpTernaryOp::NMSub)?)
    }

    fn dec32_nmadd(&self, ins: u32) -> Maybe<Instr> {
        if_ext_f!(self, self.dec32_fp_op3(ins, FpTernaryOp::NMAdd)?)
    }

    #[cfg(feature = "F")]
    fn dec32_fp_op2(ins: u32, pr: Precision, op: FpBinaryOp) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(ins);
        let rm = round_mode(fn3)?;
        Ok(Instr::FpOp2(rd, rs1, rs2, rm, pr, op))
    }

    #[cfg(feature = "F")]
    fn dec32_fp_sgnj(ins: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(ins);
        let op = match fn3 {
            0b000 => FpBinaryOp::SgnJ,
            0b001 => FpBinaryOp::SgnJN,
            0b010 => FpBinaryOp::SgnJX,
            _ => return Err(()),
        };
        Ok(Instr::FpOp2(rd, rs1, rs2, RoundMode::None, pr, op))
    }

    #[cfg(feature = "F")]
    fn dec32_fp_minmax(ins: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(ins);
        let op = match fn3 {
            0b000 => FpBinaryOp::Min,
            0b001 => FpBinaryOp::Max,
            _ => return Err(()),
        };
        Ok(Instr::FpOp2(rd, rs1, rs2, RoundMode::None, pr, op))
    }

    #[cfg(feature = "D")]
    fn dec32_fp_cvt_fp(&self, ins: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(ins);
        let rm = round_mode(fn3)?;
        let from_pr = self.dec32_fp_pr(rs2)?;
        Ok(Instr::FpCvtFp(rd, rs1, rm, from_pr, pr))
    }

    #[cfg(feature = "F")]
    fn dec32_fp_op(ins: u32, pr: Precision, op: FpUnaryOp) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(ins);
        let rm = round_mode(fn3)?;
        if rs2 != 0 {
            return Err(());
        }
        Ok(Instr::FpOp(rd, rs1, rm, pr, op))
    }

    #[cfg(feature = "F")]
    fn dec32_fp_cmp(ins: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(ins);
        let cond = match fn3 {
            0b000 => FpCmpCond::Le,
            0b001 => FpCmpCond::Lt,
            0b010 => FpCmpCond::Eq,
            _ => return Err(()),
        };
        Ok(Instr::FpCmp(rd, rs1, rs2, pr, cond))
    }

    #[cfg(feature = "F")]
    fn dec32_fp_cvt_gp(ins: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(ins);
        check_gp_regs!(rd);
        let rm = round_mode(fn3)?;
        let op = match rs2 {
            0b00 => FpGpOp::W,
            0b01 => FpGpOp::WU,
            0b10 => if_ge_rv64!(FpGpOp::L)?,
            0b11 => if_ge_rv64!(FpGpOp::LU)?,
            _ => return Err(()),
        };
        Ok(Instr::FpCvtGp(rd, rs1, rm, pr, op))
    }

    #[cfg(feature = "F")]
    fn dec32_gp_cvt_fp(ins: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(ins);
        check_gp_regs!(rs1);
        let rm = round_mode(fn3)?;
        let op = match rs2 {
            0b00 => GpFpOp::W,
            0b01 => GpFpOp::WU,
            0b10 => if_ge_rv64!(GpFpOp::L)?,
            0b11 => if_ge_rv64!(GpFpOp::LU)?,
            _ => return Err(()),
        };
        Ok(Instr::GpCvtFp(rd, rs1, rm, pr, op))
    }

    #[cfg(feature = "F")]
    fn check_fp_mv(pr: Precision) -> Maybe<()> {
        match pr {
            Precision::S => Ok(()),
            #[cfg(feature = "D")]
            Precision::D => if_ge_rv64!(()),
        }
    }

    #[cfg(feature = "F")]
    fn dec32_fp_mv_gp(ins: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(ins);
        check_gp_regs!(rd);
        if rs2 != 0 {
            return Err(());
        }
        let op = match fn3 {
            0b0 => {
                Self::check_fp_mv(pr)?;
                FpGpOp::MV
            }
            0b1 => FpGpOp::Class,
            _ => return Err(()),
        };
        Ok(Instr::FpCvtGp(rd, rs1, RoundMode::None, pr, op))
    }

    #[cfg(feature = "F")]
    fn dec32_gp_mv_fp(ins: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(ins);
        check_gp_regs!(rs1);
        if fn3 != 0 || rs2 != 0 {
            return Err(());
        }
        Self::check_fp_mv(pr)?;
        Ok(Instr::GpCvtFp(rd, rs1, RoundMode::None, pr, GpFpOp::MV))
    }

    fn dec32_op_fp(&self, ins: u32) -> Maybe<Instr> {
        if_ext_f!(self, {
            let pr = self.dec32_fp_pr(fn2(ins))?;
            match rs3(ins) {
                0b0_0000 => Self::dec32_fp_op2(ins, pr, FpBinaryOp::Add),
                0b0_0001 => Self::dec32_fp_op2(ins, pr, FpBinaryOp::Sub),
                0b0_0010 => Self::dec32_fp_op2(ins, pr, FpBinaryOp::Mul),
                0b0_0011 => Self::dec32_fp_op2(ins, pr, FpBinaryOp::Div),
                0b0_0100 => Self::dec32_fp_sgnj(ins, pr),
                0b0_0101 => Self::dec32_fp_minmax(ins, pr),
                0b0_1000 => if_ext_d!(self, self.dec32_fp_cvt_fp(ins, pr)?),
                0b0_1011 => Self::dec32_fp_op(ins, pr, FpUnaryOp::Sqrt),
                0b1_0100 => Self::dec32_fp_cmp(ins, pr),
                0b1_1000 => Self::dec32_fp_cvt_gp(ins, pr),
                0b1_1010 => Self::dec32_gp_cvt_fp(ins, pr),
                0b1_1100 => Self::dec32_fp_mv_gp(ins, pr),
                0b1_1110 => Self::dec32_gp_mv_fp(ins, pr),
                _ => Err(()),
            }
        }?)
    }

    fn dec32_branch(ins: u32) -> Maybe<Instr> {
        let (fn3, rs1, rs2, imm) = b_type(ins);
        check_gp_regs!(rs1, rs2);
        let cond = match fn3 {
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

    fn dec32_jalr(ins: u32) -> Maybe<Instr> {
        let (rd, fn3, rs1, imm) = i_type(ins);
        check_gp_regs!(rd, rs1);
        if fn3 != 0 {
            return Err(());
        }
        Ok(Instr::Jalr(rd, rs1, imm))
    }

    fn dec32_jal(ins: u32) -> Maybe<Instr> {
        let (rd, imm) = j_type(ins);
        check_gp_regs!(rd);
        Ok(Instr::Jal(rd, imm))
    }

    #[cfg(feature = "Zicsr")]
    fn dec32_csr(&self, ins: u32) -> Maybe<Instr> {
        let rd = rd(ins);
        let fn3 = fn3(ins);
        let rs1 = rs1(ins);
        let addr = select_bits(ins, 31, 20) as u16;
        let csr_op = match fn3 {
            0b001 => CsrOp::Rw,
            0b010 => CsrOp::Rs,
            0b011 => CsrOp::Rc,
            0b101 => CsrOp::Rwi,
            0b110 => CsrOp::Rsi,
            0b111 => CsrOp::Rci,
            _ => return Err(()),
        };
        if fn3 < 0b100 {
            check_gp_regs!(rs1);
        }
        check_gp_regs!(rd);
        Ok(Instr::Csr(rd, rs1, addr, csr_op))
    }

    fn dec32_system(&self, ins: u32) -> Maybe<Instr> {
        if fn3(ins) == 0 {
            match ins >> 7 {
                0b0 => Ok(Instr::Trap(Exception::Ecall)),
                0b10_0000_0000_0000 => Ok(Instr::Trap(Exception::Ebreak)),
                _ => Err(()),
            }
        } else {
            if_ext_zicsr!(self, self.dec32_csr(ins)?)
        }
    }

    pub fn dec32(&self, ins: u32) -> Instr {
        match select_bits(ins, 6, 2) {
            0b0_0000 => Self::dec32_load(ins),
            0b0_0001 => self.dec32_load_fp(ins),
            0b0_0011 => self.dec32_misc_mem(ins),
            0b0_0100 => Self::dec32_op_imm(ins),
            0b0_0101 => Self::dec32_auipc(ins),
            0b0_0110 => Self::dec32_op_imm_32(ins),
            0b0_1000 => Self::dec32_store(ins),
            0b0_1001 => self.dec32_store_fp(ins),
            0b0_1011 => self.dec32_amo(ins),
            0b0_1100 => self.dec32_op(ins),
            0b0_1101 => Self::dec32_lui(ins),
            0b0_1110 => self.dec32_op_32(ins),
            0b1_0000 => self.dec32_madd(ins),
            0b1_0001 => self.dec32_msub(ins),
            0b1_0010 => self.dec32_nmsub(ins),
            0b1_0011 => self.dec32_nmadd(ins),
            0b1_0100 => self.dec32_op_fp(ins),
            0b1_1000 => Self::dec32_branch(ins),
            0b1_1001 => Self::dec32_jalr(ins),
            0b1_1011 => Self::dec32_jal(ins),
            0b1_1100 => self.dec32_system(ins),
            _ => Err(()),
        }
        .unwrap_or(Instr::Trap(Exception::IllegalInstr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type RV32I = HartIsa<u32, false>;
    type RV32E = HartIsa<u32, true>;
    type RV64I = HartIsa<u64, false>;
    const ILL: Instr = Instr::Trap(Exception::IllegalInstr);

    fn all_pass<Xlen: XlenT, const EMB: bool>(
        hart: &HartIsa<Xlen, EMB>,
        ins_raw: &[u32],
        ins_dec: &[Instr],
    ) -> bool {
        for (raw, dec) in ins_raw.iter().zip(ins_dec.iter()) {
            if hart.dec32(*raw) != *dec {
                println!(
                    "raw: {:08x}, exp: {:?}, dec: {:?}",
                    raw,
                    dec,
                    hart.dec32(*raw)
                );
                return false;
            }
        }
        true
    }
    fn all_fail<Xlen: XlenT, const EMB: bool>(hart: &HartIsa<Xlen, EMB>, ins_raw: &[u32]) -> bool {
        for raw in ins_raw.iter() {
            if hart.dec32(*raw) != ILL {
                println!("raw: {:08x}, exp: ILL, dec: {:?}", raw, hart.dec32(*raw));
                return false;
            }
        }
        true
    }

    #[test]
    fn rv32i() {
        let ins_raw = [
            0xaaaaaab7u32,
            0xaaaaaa97u32,
            0xd5455aefu32,
            0xaaad8ae7u32,
            0xd4dd8a63u32,
            0xd4dd9a63u32,
            0xd4ddca63u32,
            0xd4ddda63u32,
            0xd4ddea63u32,
            0xd4ddfa63u32,
            0xaaad8a83u32,
            0xaaad9a83u32,
            0xaaadaa83u32,
            0xaaadca83u32,
            0xaaadda83u32,
            0xaadd8523u32,
            0xaadd9523u32,
            0xaadda523u32,
            0xaaad8a93u32,
            0xaaadaa93u32,
            0xaaadba93u32,
            0xaaadca93u32,
            0xaaadea93u32,
            0xaaadfa93u32,
            0x015d9a93u32,
            0x015dda93u32,
            0x415dda93u32,
            0x00dd8ab3u32,
            0x40dd8ab3u32,
            0x00dd9ab3u32,
            0x00ddaab3u32,
            0x00ddbab3u32,
            0x00ddcab3u32,
            0x00dddab3u32,
            0x40dddab3u32,
            0x00ddeab3u32,
            0x00ddfab3u32,
            0x0a50000fu32,
            0x8330000fu32,
            0x00000073u32,
            0x00100073u32,
        ];
        let ins_dec = [
            Instr::OpImm(21, 0, -1431658496, BinaryOp::Add),
            Instr::Auipc(21, -1431658496),
            Instr::Jal(21, -699052),
            Instr::Jalr(21, 27, -1366),
            Instr::Branch(27, 13, -2732, CmpCond::Eq),
            Instr::Branch(27, 13, -2732, CmpCond::Ne),
            Instr::Branch(27, 13, -2732, CmpCond::Lt),
            Instr::Branch(27, 13, -2732, CmpCond::Ge),
            Instr::Branch(27, 13, -2732, CmpCond::LtU),
            Instr::Branch(27, 13, -2732, CmpCond::GeU),
            Instr::Load(21, 27, -1366, MemWidth::B),
            Instr::Load(21, 27, -1366, MemWidth::H),
            Instr::Load(21, 27, -1366, MemWidth::W),
            Instr::Load(21, 27, -1366, MemWidth::BU),
            Instr::Load(21, 27, -1366, MemWidth::HU),
            Instr::Store(27, 13, -1366, MemWidth::B),
            Instr::Store(27, 13, -1366, MemWidth::H),
            Instr::Store(27, 13, -1366, MemWidth::W),
            Instr::OpImm(21, 27, -1366, BinaryOp::Add),
            Instr::OpImm(21, 27, -1366, BinaryOp::Slt),
            Instr::OpImm(21, 27, -1366, BinaryOp::SltU),
            Instr::OpImm(21, 27, -1366, BinaryOp::Xor),
            Instr::OpImm(21, 27, -1366, BinaryOp::Or),
            Instr::OpImm(21, 27, -1366, BinaryOp::And),
            Instr::OpImm(21, 27, 21, BinaryOp::Sll),
            Instr::OpImm(21, 27, 21, BinaryOp::Srl),
            Instr::OpImm(21, 27, 21, BinaryOp::Sra),
            Instr::Op(21, 27, 13, BinaryOp::Add),
            Instr::Op(21, 27, 13, BinaryOp::Sub),
            Instr::Op(21, 27, 13, BinaryOp::Sll),
            Instr::Op(21, 27, 13, BinaryOp::Slt),
            Instr::Op(21, 27, 13, BinaryOp::SltU),
            Instr::Op(21, 27, 13, BinaryOp::Xor),
            Instr::Op(21, 27, 13, BinaryOp::Srl),
            Instr::Op(21, 27, 13, BinaryOp::Sra),
            Instr::Op(21, 27, 13, BinaryOp::Or),
            Instr::Op(21, 27, 13, BinaryOp::And),
            Instr::MiscMem(MiscMemOp::Fence(10, 5, FenceMode::Normal)),
            Instr::MiscMem(MiscMemOp::Fence(3, 3, FenceMode::Tso)),
            Instr::Trap(Exception::Ecall),
            Instr::Trap(Exception::Ebreak),
        ];
        assert!(all_pass(&RV32I::default(), &ins_raw, &ins_dec));
        assert!(all_fail(&RV32E::default(), &ins_raw[..37]));
    }

    #[cfg(feature = "RV64")]
    #[test]
    fn rv64i() {
        let ins_raw = [
            0xaaadea83u32,
            0xaaadba83u32,
            0xaaddb523u32,
            0x02ad9a93u32,
            0x02adda93u32,
            0x42adda93u32,
            0xaaad8a9bu32,
            0x015d9a9bu32,
            0x015dda9bu32,
            0x415dda9bu32,
            0x00dd8abbu32,
            0x40dd8abbu32,
            0x00dd9abbu32,
            0x00dddabbu32,
            0x40dddabbu32,
        ];
        let ins_dec = [
            Instr::Load(21, 27, -1366, MemWidth::WU),
            Instr::Load(21, 27, -1366, MemWidth::D),
            Instr::Store(27, 13, -1366, MemWidth::D),
            Instr::OpImm(21, 27, 42, BinaryOp::Sll),
            Instr::OpImm(21, 27, 42, BinaryOp::Srl),
            Instr::OpImm(21, 27, 42, BinaryOp::Sra),
            Instr::OpImm(21, 27, -1366, BinaryOp::AddW),
            Instr::OpImm(21, 27, 21, BinaryOp::SllW),
            Instr::OpImm(21, 27, 21, BinaryOp::SrlW),
            Instr::OpImm(21, 27, 21, BinaryOp::SraW),
            Instr::Op(21, 27, 13, BinaryOp::AddW),
            Instr::Op(21, 27, 13, BinaryOp::SubW),
            Instr::Op(21, 27, 13, BinaryOp::SllW),
            Instr::Op(21, 27, 13, BinaryOp::SrlW),
            Instr::Op(21, 27, 13, BinaryOp::SraW),
        ];
        assert!(all_pass(&RV64I::default(), &ins_raw, &ins_dec));
        assert!(all_fail(&RV32I::default(), &ins_raw));
    }

    #[cfg(feature = "Zifencei")]
    #[test]
    fn zifencei_ext() {
        let ins_raw = [0x0000100fu32];
        let ins_dec = [Instr::MiscMem(MiscMemOp::FenceI)];
        let mut hart = RV32I::default();
        assert!(all_fail(&hart, &ins_raw));
        hart.Zifencei = true;
        assert!(all_pass(&hart, &ins_raw, &ins_dec));
    }

    #[cfg(feature = "Zicsr")]
    #[test]
    fn zicsr_ext() {
        let ins_raw = [
            0xaaad9af3u32,
            0xaaadaaf3u32,
            0xaaadbaf3u32,
            0xaaaddaf3u32,
            0xaaadeaf3u32,
            0xaaadfaf3u32,
        ];
        let ins_dec = [
            Instr::Csr(21, 27, 2730, CsrOp::Rw),
            Instr::Csr(21, 27, 2730, CsrOp::Rs),
            Instr::Csr(21, 27, 2730, CsrOp::Rc),
            Instr::Csr(21, 27, 2730, CsrOp::Rwi),
            Instr::Csr(21, 27, 2730, CsrOp::Rsi),
            Instr::Csr(21, 27, 2730, CsrOp::Rci),
        ];
        let mut hart = RV32I::default();
        assert!(all_fail(&hart, &ins_raw));
        hart.Zicsr = true;
        assert!(all_pass(&hart, &ins_raw, &ins_dec));
    }

    #[cfg(feature = "M")]
    #[test]
    fn m_ext() {
        let ins_raw = [
            0x02dd8ab3u32,
            0x02dd9ab3u32,
            0x02ddaab3u32,
            0x02ddbab3u32,
            0x02ddcab3u32,
            0x02dddab3u32,
            0x02ddeab3u32,
            0x02ddfab3u32,
        ];
        let ins_dec = [
            Instr::Op(21, 27, 13, BinaryOp::Mul),
            Instr::Op(21, 27, 13, BinaryOp::Mulh),
            Instr::Op(21, 27, 13, BinaryOp::MulhSU),
            Instr::Op(21, 27, 13, BinaryOp::MulhU),
            Instr::Op(21, 27, 13, BinaryOp::Div),
            Instr::Op(21, 27, 13, BinaryOp::DivU),
            Instr::Op(21, 27, 13, BinaryOp::Rem),
            Instr::Op(21, 27, 13, BinaryOp::RemU),
        ];
        let mut hart = RV32I::default();
        assert!(all_fail(&hart, &ins_raw));
        hart.M = true;
        assert!(all_pass(&hart, &ins_raw, &ins_dec));
    }

    #[cfg(all(feature = "M", feature = "RV64"))]
    #[test]
    fn m64_ext() {
        let ins_raw = [
            0x02dd8abbu32,
            0x02ddcabbu32,
            0x02dddabbu32,
            0x02ddeabbu32,
            0x02ddfabbu32,
        ];
        let ins_dec = [
            Instr::Op(21, 27, 13, BinaryOp::MulW),
            Instr::Op(21, 27, 13, BinaryOp::DivW),
            Instr::Op(21, 27, 13, BinaryOp::DivUW),
            Instr::Op(21, 27, 13, BinaryOp::RemW),
            Instr::Op(21, 27, 13, BinaryOp::RemUW),
        ];
        let mut hart = RV64I::default();
        assert!(all_fail(&hart, &ins_raw));
        hart.M = true;
        assert!(all_pass(&hart, &ins_raw, &ins_dec));
        let mut hart = RV32I::default();
        hart.M = true;
        assert!(all_fail(&hart, &ins_raw));
    }

    #[cfg(feature = "A")]
    #[test]
    fn a_ext() {
        let ins_raw = [
            0x140daaafu32,
            0x1addaaafu32,
            0x0eddaaafu32,
            0x06ddaaafu32,
            0x26ddaaafu32,
            0x66ddaaafu32,
            0x46ddaaafu32,
            0x86ddaaafu32,
            0xa6ddaaafu32,
            0xc6ddaaafu32,
            0xe6ddaaafu32,
        ];
        let ins_dec = [
            Instr::LoadReserved(21, 27, MemOrder::Acquire, MemWidth::W),
            Instr::StoreConditional(21, 27, 13, MemOrder::Release, MemWidth::W),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::W, BinaryOp::Second),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::W, BinaryOp::Add),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::W, BinaryOp::Xor),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::W, BinaryOp::And),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::W, BinaryOp::Or),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::W, BinaryOp::Min),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::W, BinaryOp::Max),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::W, BinaryOp::MinU),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::W, BinaryOp::MaxU),
        ];
        let mut hart = RV32I::default();
        assert!(all_fail(&hart, &ins_raw));
        hart.A = true;
        assert!(all_pass(&hart, &ins_raw, &ins_dec));
    }

    #[cfg(all(feature = "A", feature = "RV64"))]
    #[test]
    fn a64_ext() {
        let ins_raw = [
            0x140dbaafu32,
            0x1addbaafu32,
            0x0eddbaafu32,
            0x06ddbaafu32,
            0x26ddbaafu32,
            0x66ddbaafu32,
            0x46ddbaafu32,
            0x86ddbaafu32,
            0xa6ddbaafu32,
            0xc6ddbaafu32,
            0xe6ddbaafu32,
        ];
        let ins_dec = [
            Instr::LoadReserved(21, 27, MemOrder::Acquire, MemWidth::D),
            Instr::StoreConditional(21, 27, 13, MemOrder::Release, MemWidth::D),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::D, BinaryOp::Second),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::D, BinaryOp::Add),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::D, BinaryOp::Xor),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::D, BinaryOp::And),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::D, BinaryOp::Or),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::D, BinaryOp::Min),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::D, BinaryOp::Max),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::D, BinaryOp::MinU),
            Instr::Amo(21, 27, 13, MemOrder::AcqRel, MemWidth::D, BinaryOp::MaxU),
        ];
        let mut hart = RV64I::default();
        assert!(all_fail(&hart, &ins_raw));
        hart.A = true;
        assert!(all_pass(&hart, &ins_raw, &ins_dec));
        let mut hart = RV32I::default();
        hart.A = true;
        assert!(all_fail(&hart, &ins_raw));
    }

    #[cfg(feature = "F")]
    #[test]
    fn f_ext() {
        let ins_raw = [
            0xaaadaa87u32,
            0xaadda527u32,
            0x38dd8ac3u32,
            0x38dd9ac7u32,
            0x38ddaacbu32,
            0x38ddbacfu32,
            0x00ddcad3u32,
            0x08ddfad3u32,
            0x10ddfad3u32,
            0x18ddfad3u32,
            0x580dfad3u32,
            0x20dd8ad3u32,
            0x20dd9ad3u32,
            0x20ddaad3u32,
            0x28dd8ad3u32,
            0x28dd9ad3u32,
            0xc00dfad3u32,
            0xc01dfad3u32,
            0xe00d8ad3u32,
            0xa0ddaad3u32,
            0xa0dd9ad3u32,
            0xa0dd8ad3u32,
            0xe00d9ad3u32,
            0xd00dfad3u32,
            0xd01dfad3u32,
            0xf00d8ad3u32,
        ];
        let ins_dec = [
            Instr::LoadFp(21, 27, -1366, Precision::S),
            Instr::StoreFp(27, 13, -1366, Precision::S),
            Instr::FpOp3(
                21,
                27,
                13,
                7,
                RoundMode::Rne,
                Precision::S,
                FpTernaryOp::MAdd,
            ),
            Instr::FpOp3(
                21,
                27,
                13,
                7,
                RoundMode::Rtz,
                Precision::S,
                FpTernaryOp::MSub,
            ),
            Instr::FpOp3(
                21,
                27,
                13,
                7,
                RoundMode::Rdn,
                Precision::S,
                FpTernaryOp::NMSub,
            ),
            Instr::FpOp3(
                21,
                27,
                13,
                7,
                RoundMode::Rup,
                Precision::S,
                FpTernaryOp::NMAdd,
            ),
            Instr::FpOp2(21, 27, 13, RoundMode::Rmm, Precision::S, FpBinaryOp::Add),
            Instr::FpOp2(21, 27, 13, RoundMode::Dyn, Precision::S, FpBinaryOp::Sub),
            Instr::FpOp2(21, 27, 13, RoundMode::Dyn, Precision::S, FpBinaryOp::Mul),
            Instr::FpOp2(21, 27, 13, RoundMode::Dyn, Precision::S, FpBinaryOp::Div),
            Instr::FpOp(21, 27, RoundMode::Dyn, Precision::S, FpUnaryOp::Sqrt),
            Instr::FpOp2(21, 27, 13, RoundMode::None, Precision::S, FpBinaryOp::SgnJ),
            Instr::FpOp2(21, 27, 13, RoundMode::None, Precision::S, FpBinaryOp::SgnJN),
            Instr::FpOp2(21, 27, 13, RoundMode::None, Precision::S, FpBinaryOp::SgnJX),
            Instr::FpOp2(21, 27, 13, RoundMode::None, Precision::S, FpBinaryOp::Min),
            Instr::FpOp2(21, 27, 13, RoundMode::None, Precision::S, FpBinaryOp::Max),
            Instr::FpCvtGp(21, 27, RoundMode::Dyn, Precision::S, FpGpOp::W),
            Instr::FpCvtGp(21, 27, RoundMode::Dyn, Precision::S, FpGpOp::WU),
            Instr::FpCvtGp(21, 27, RoundMode::None, Precision::S, FpGpOp::MV),
            Instr::FpCmp(21, 27, 13, Precision::S, FpCmpCond::Eq),
            Instr::FpCmp(21, 27, 13, Precision::S, FpCmpCond::Lt),
            Instr::FpCmp(21, 27, 13, Precision::S, FpCmpCond::Le),
            Instr::FpCvtGp(21, 27, RoundMode::None, Precision::S, FpGpOp::Class),
            Instr::GpCvtFp(21, 27, RoundMode::Dyn, Precision::S, GpFpOp::W),
            Instr::GpCvtFp(21, 27, RoundMode::Dyn, Precision::S, GpFpOp::WU),
            Instr::GpCvtFp(21, 27, RoundMode::None, Precision::S, GpFpOp::MV),
        ];
        let mut hart = RV32I::default();
        assert!(all_fail(&hart, &ins_raw));
        hart.F = true;
        assert!(all_pass(&hart, &ins_raw, &ins_dec));
    }

    #[cfg(all(feature = "F", feature = "RV64"))]
    #[test]
    fn f64_ext() {
        let ins_raw = [0xc02dfad3u32, 0xc03dfad3u32, 0xd02dfad3u32, 0xd03dfad3u32];
        let ins_dec = [
            Instr::FpCvtGp(21, 27, RoundMode::Dyn, Precision::S, FpGpOp::L),
            Instr::FpCvtGp(21, 27, RoundMode::Dyn, Precision::S, FpGpOp::LU),
            Instr::GpCvtFp(21, 27, RoundMode::Dyn, Precision::S, GpFpOp::L),
            Instr::GpCvtFp(21, 27, RoundMode::Dyn, Precision::S, GpFpOp::LU),
        ];
        let mut hart = RV64I::default();
        assert!(all_fail(&hart, &ins_raw));
        hart.F = true;
        assert!(all_pass(&hart, &ins_raw, &ins_dec));
        let mut hart = RV32I::default();
        hart.F = true;
        assert!(all_fail(&hart, &ins_raw));
    }

    #[cfg(feature = "D")]
    #[test]
    fn d_ext() {
        let ins_raw = [
            0xaaadba87u32,
            0xaaddb527u32,
            0x3add8ac3u32,
            0x3add9ac7u32,
            0x3addaacbu32,
            0x3addbacfu32,
            0x02ddcad3u32,
            0x0addfad3u32,
            0x12ddfad3u32,
            0x1addfad3u32,
            0x5a0dfad3u32,
            0x22dd8ad3u32,
            0x22dd9ad3u32,
            0x22ddaad3u32,
            0x2add8ad3u32,
            0x2add9ad3u32,
            0x401dfad3u32,
            0x420d8ad3u32,
            0xa2ddaad3u32,
            0xa2dd9ad3u32,
            0xa2dd8ad3u32,
            0xe20d9ad3u32,
            0xc20dfad3u32,
            0xc21dfad3u32,
            0xd20d8ad3u32,
            0xd21d8ad3u32,
        ];
        let ins_dec = [
            Instr::LoadFp(21, 27, -1366, Precision::D),
            Instr::StoreFp(27, 13, -1366, Precision::D),
            Instr::FpOp3(
                21,
                27,
                13,
                7,
                RoundMode::Rne,
                Precision::D,
                FpTernaryOp::MAdd,
            ),
            Instr::FpOp3(
                21,
                27,
                13,
                7,
                RoundMode::Rtz,
                Precision::D,
                FpTernaryOp::MSub,
            ),
            Instr::FpOp3(
                21,
                27,
                13,
                7,
                RoundMode::Rdn,
                Precision::D,
                FpTernaryOp::NMSub,
            ),
            Instr::FpOp3(
                21,
                27,
                13,
                7,
                RoundMode::Rup,
                Precision::D,
                FpTernaryOp::NMAdd,
            ),
            Instr::FpOp2(21, 27, 13, RoundMode::Rmm, Precision::D, FpBinaryOp::Add),
            Instr::FpOp2(21, 27, 13, RoundMode::Dyn, Precision::D, FpBinaryOp::Sub),
            Instr::FpOp2(21, 27, 13, RoundMode::Dyn, Precision::D, FpBinaryOp::Mul),
            Instr::FpOp2(21, 27, 13, RoundMode::Dyn, Precision::D, FpBinaryOp::Div),
            Instr::FpOp(21, 27, RoundMode::Dyn, Precision::D, FpUnaryOp::Sqrt),
            Instr::FpOp2(21, 27, 13, RoundMode::None, Precision::D, FpBinaryOp::SgnJ),
            Instr::FpOp2(21, 27, 13, RoundMode::None, Precision::D, FpBinaryOp::SgnJN),
            Instr::FpOp2(21, 27, 13, RoundMode::None, Precision::D, FpBinaryOp::SgnJX),
            Instr::FpOp2(21, 27, 13, RoundMode::None, Precision::D, FpBinaryOp::Min),
            Instr::FpOp2(21, 27, 13, RoundMode::None, Precision::D, FpBinaryOp::Max),
            Instr::FpCvtFp(21, 27, RoundMode::Dyn, Precision::D, Precision::S),
            Instr::FpCvtFp(21, 27, RoundMode::Rne, Precision::S, Precision::D),
            Instr::FpCmp(21, 27, 13, Precision::D, FpCmpCond::Eq),
            Instr::FpCmp(21, 27, 13, Precision::D, FpCmpCond::Lt),
            Instr::FpCmp(21, 27, 13, Precision::D, FpCmpCond::Le),
            Instr::FpCvtGp(21, 27, RoundMode::None, Precision::D, FpGpOp::Class),
            Instr::FpCvtGp(21, 27, RoundMode::Dyn, Precision::D, FpGpOp::W),
            Instr::FpCvtGp(21, 27, RoundMode::Dyn, Precision::D, FpGpOp::WU),
            Instr::GpCvtFp(21, 27, RoundMode::Rne, Precision::D, GpFpOp::W),
            Instr::GpCvtFp(21, 27, RoundMode::Rne, Precision::D, GpFpOp::WU),
        ];
        let mut hart = RV32I::default();
        hart.F = true;
        assert!(all_fail(&hart, &ins_raw));
        hart.D = true;
        assert!(all_pass(&hart, &ins_raw, &ins_dec));
    }

    #[cfg(all(feature = "D", feature = "RV64"))]
    #[test]
    fn d64_ext() {
        let ins_raw = [
            0xc22dfad3u32,
            0xc23dfad3u32,
            0xe20d8ad3u32,
            0xd22dfad3u32,
            0xd23dfad3u32,
            0xf20d8ad3u32,
        ];
        let ins_dec = [
            Instr::FpCvtGp(21, 27, RoundMode::Dyn, Precision::D, FpGpOp::L),
            Instr::FpCvtGp(21, 27, RoundMode::Dyn, Precision::D, FpGpOp::LU),
            Instr::FpCvtGp(21, 27, RoundMode::None, Precision::D, FpGpOp::MV),
            Instr::GpCvtFp(21, 27, RoundMode::Dyn, Precision::D, GpFpOp::L),
            Instr::GpCvtFp(21, 27, RoundMode::Dyn, Precision::D, GpFpOp::LU),
            Instr::GpCvtFp(21, 27, RoundMode::None, Precision::D, GpFpOp::MV),
        ];
        let mut hart = RV64I::default();
        hart.F = true;
        assert!(all_fail(&hart, &ins_raw));
        hart.D = true;
        assert!(all_pass(&hart, &ins_raw, &ins_dec));
        let mut hart = RV32I::default();
        hart.F = true;
        hart.D = true;
        assert!(all_fail(&hart, &ins_raw));
    }
}
