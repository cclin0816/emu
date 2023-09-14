use crate::{decode::common::*, hart::HartIsa, micro_op::*, utils::*, xlen::XlenT};

fn funct3(instr: u32) -> u8 {
    select_bits(instr, 14, 12) as u8
}

fn funct7(instr: u32) -> u8 {
    select_bits(instr, 31, 25) as u8
}

fn funct2(instr: u32) -> u8 {
    select_bits(instr, 26, 25) as u8
}

fn sext(imm: u32, sign_bit: u32) -> i32 {
    let len = 31 - sign_bit;
    let imm = (imm << len) as i32;
    imm >> len
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

fn dec_rd(instr: u32) -> u8 {
    select_bits(instr, 11, 7) as u8
}

fn dec_rs1(instr: u32) -> u8 {
    select_bits(instr, 19, 15) as u8
}

fn dec_rs2(instr: u32) -> u8 {
    select_bits(instr, 24, 20) as u8
}

fn rs3(instr: u32) -> u8 {
    select_bits(instr, 31, 27) as u8
}
/// (rd, funct3, rs1, rs2, funct7)
fn r_type(instr: u32) -> (u8, u8, u8, u8, u8) {
    (
        dec_rd(instr),
        funct3(instr),
        dec_rs1(instr),
        dec_rs2(instr),
        funct7(instr),
    )
}
/// (rd, funct3, rs1, imm)
fn i_type(instr: u32) -> (u8, u8, u8, i32) {
    (dec_rd(instr), funct3(instr), dec_rs1(instr), i_imm(instr))
}
/// (funct3, rs1, rs2, imm)
fn s_type(instr: u32) -> (u8, u8, u8, i32) {
    (funct3(instr), dec_rs1(instr), dec_rs2(instr), s_imm(instr))
}
/// (funct3, rs1, rs2, imm)
fn b_type(instr: u32) -> (u8, u8, u8, i32) {
    (funct3(instr), dec_rs1(instr), dec_rs2(instr), b_imm(instr))
}
/// (rd, imm)
fn u_type(instr: u32) -> (u8, i32) {
    (dec_rd(instr), u_imm(instr))
}
/// (rd, imm)
fn j_type(instr: u32) -> (u8, i32) {
    (dec_rd(instr), j_imm(instr))
}
/// (rd, funct3, rs1, rs2, funct2, rs3 / funct5)
fn r4_type(instr: u32) -> (u8, u8, u8, u8, u8, u8) {
    (
        dec_rd(instr),
        funct3(instr),
        dec_rs1(instr),
        dec_rs2(instr),
        funct2(instr),
        rs3(instr),
    )
}

fn op_sl_imm(imm: i32, xlen: u32) -> Maybe<BinaryOp> {
    let imm = imm as u32;
    if imm >= xlen {
        Err(())
    } else {
        Ok(BinaryOp::Sll)
    }
}

fn op_sr_imm(imm: &mut i32, xlen: u32) -> Maybe<BinaryOp> {
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
fn dec_round_mode(fn3: u8) -> Maybe<RoundMode> {
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
    fn dec_load(instr: u32) -> Maybe<Instr> {
        let (rd, fn3, rs1, imm) = i_type(instr);
        check_gp_regs!(rd, rs1);
        let mem_width = match fn3 {
            0b000 => MemWidth::B,
            0b001 => MemWidth::H,
            0b010 => MemWidth::W,
            0b011 => en_if_ge_rv64!(MemWidth::D)?,
            0b100 => MemWidth::BU,
            0b101 => MemWidth::HU,
            0b110 => en_if_ge_rv64!(MemWidth::WU)?,
            _ => return Err(()),
        };
        Ok(Instr::Load(rd, rs1, imm, mem_width))
    }

    fn dec_load_fp(&self, instr: u32) -> Maybe<Instr> {
        en_if_ext_f!(self, {
            let (rd, fn3, rs1, imm) = i_type(instr);
            check_gp_regs!(rs1);
            let pr = match fn3 {
                0b010 => Precision::S,
                0b011 => en_if_ext_d!(self, Precision::D)?,
                _ => return Err(()),
            };
            Instr::LoadFp(rd, rs1, imm, pr)
        })
    }

    fn dec_misc_mem(&self, instr: u32) -> Maybe<Instr> {
        match funct3(instr) {
            0b000 => {
                let fm = match select_bits(instr, 31, 28) {
                    0b0000 => FenceMode::Normal,
                    0b1000 => FenceMode::Tso,
                    // Base implementations shall treat all such reserved
                    // configurations as normal fences
                    _ => FenceMode::Normal,
                };
                let pred_succ = select_bits(instr, 27, 20) as u8;
                Ok(Instr::MiscMem(MiscMemOp::Fence(pred_succ, fm)))
            }
            0b001 => en_if_ext_zifencei!(self, Instr::MiscMem(MiscMemOp::FenceI)),
            _ => Err(()),
        }
    }

    fn dec_op_imm(instr: u32) -> Maybe<Instr> {
        let (rd, fn3, rs1, mut imm) = i_type(instr);
        check_gp_regs!(rd, rs1);
        let op = match fn3 {
            0b000 => BinaryOp::Add,
            0b001 => op_sl_imm(imm, Xlen::xlen())?,
            0b010 => BinaryOp::Slt,
            0b011 => BinaryOp::SltU,
            0b100 => BinaryOp::Xor,
            0b101 => op_sr_imm(&mut imm, Xlen::xlen())?,
            0b110 => BinaryOp::Or,
            0b111 => BinaryOp::And,
            _ => unreachable!(),
        };
        Ok(Instr::OpImm(rd, rs1, imm, op))
    }

    fn dec_auipc(instr: u32) -> Maybe<Instr> {
        let (rd, imm) = u_type(instr);
        check_gp_regs!(rd);
        Ok(Instr::Auipc(rd, imm))
    }

    fn dec_op_imm_32(instr: u32) -> Maybe<Instr> {
        en_if_ge_rv64!({
            let (rd, fn3, rs1, mut imm) = i_type(instr);
            check_gp_regs!(rd, rs1);
            let op = match fn3 {
                0b000 => BinaryOp::AddW,
                0b001 => {
                    op_sl_imm(imm, 32)?;
                    BinaryOp::SllW
                }
                0b101 => match op_sr_imm(&mut imm, 32)? {
                    BinaryOp::Srl => BinaryOp::SrlW,
                    BinaryOp::Sra => BinaryOp::SraW,
                    _ => unreachable!(),
                },
                _ => return Err(()),
            };
            Instr::OpImm(rd, rs1, imm, op)
        })
    }

    fn dec_store(instr: u32) -> Maybe<Instr> {
        let (fn3, rs1, rs2, imm) = s_type(instr);
        check_gp_regs!(rs1, rs2);
        let mem_width = match fn3 {
            0b000 => MemWidth::B,
            0b001 => MemWidth::H,
            0b010 => MemWidth::W,
            0b011 => en_if_ge_rv64!(MemWidth::D)?,
            _ => return Err(()),
        };
        Ok(Instr::Store(rs1, rs2, imm, mem_width))
    }

    fn dec_store_fp(&self, instr: u32) -> Maybe<Instr> {
        en_if_ext_f!(self, {
            let (fn3, rs1, rs2, imm) = s_type(instr);
            check_gp_regs!(rs1);
            let pr = match fn3 {
                0b010 => Precision::S,
                0b011 => en_if_ext_d!(self, Precision::D)?,
                _ => return Err(()),
            };
            Instr::StoreFp(rs1, rs2, imm, pr)
        })
    }

    fn dec_amo(&self, instr: u32) -> Maybe<Instr> {
        en_if_ext_a!(self, {
            let (rd, fn3, rs1, rs2, fn2, fn5) = r4_type(instr);
            check_gp_regs!(rd, rs1, rs2);
            let mem_width = match fn3 {
                0b010 => MemWidth::W,
                0b011 => en_if_ge_rv64!(MemWidth::D)?,
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
                    if rd != 0 {
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

    fn dec_op(&self, instr: u32) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, fn7) = r_type(instr);
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
            0b0000001 => en_if_ext_m!(self, {
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

    fn dec_lui(instr: u32) -> Maybe<Instr> {
        let (rd, imm) = u_type(instr);
        check_gp_regs!(rd);
        Ok(Instr::Lui(rd, imm))
    }

    fn dec_op_32(&self, instr: u32) -> Maybe<Instr> {
        en_if_ge_rv64!({
            let (rd, fn3, rs1, rs2, fn7) = r_type(instr);
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
                0b0000001 => en_if_ext_m!(self, {
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
    fn dec_precision(&self, fmt: u8) -> Maybe<Precision> {
        match fmt {
            0b00 => Ok(Precision::S),
            0b01 => en_if_ext_d!(self, Precision::D),
            _ => Err(()),
        }
    }

    #[cfg(feature = "F")]
    fn dec_fp_op3(&self, instr: u32, op: FpTenaryOp) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, fn2, rs3) = r4_type(instr);
        let rm = dec_round_mode(fn3)?;
        let pr = self.dec_precision(fn2)?;
        Ok(Instr::FpOp3(rd, rs1, rs2, rs3, rm, pr, op))
    }

    fn dec_madd(&self, instr: u32) -> Maybe<Instr> {
        en_if_ext_f!(self, self.dec_fp_op3(instr, FpTenaryOp::MAdd)?)
    }

    fn dec_msub(&self, instr: u32) -> Maybe<Instr> {
        en_if_ext_f!(self, self.dec_fp_op3(instr, FpTenaryOp::MSub)?)
    }

    fn dec_nmsub(&self, instr: u32) -> Maybe<Instr> {
        en_if_ext_f!(self, self.dec_fp_op3(instr, FpTenaryOp::NMSub)?)
    }

    fn dec_nmadd(&self, instr: u32) -> Maybe<Instr> {
        en_if_ext_f!(self, self.dec_fp_op3(instr, FpTenaryOp::NMAdd)?)
    }

    #[cfg(feature = "F")]
    fn dec_fp_op2(instr: u32, pr: Precision, op: FpBinaryOp) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(instr);
        let rm = dec_round_mode(fn3)?;
        Ok(Instr::FpOp2(rd, rs1, rs2, rm, pr, op))
    }

    #[cfg(feature = "F")]
    fn dec_fp_sgnj(instr: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(instr);
        let op = match fn3 {
            0b000 => FpBinaryOp::SgnJ,
            0b001 => FpBinaryOp::SgnJN,
            0b010 => FpBinaryOp::SgnJX,
            _ => return Err(()),
        };
        Ok(Instr::FpOp2(rd, rs1, rs2, RoundMode::None, pr, op))
    }

    #[cfg(feature = "F")]
    fn dec_fp_minmax(instr: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(instr);
        let op = match fn3 {
            0b000 => FpBinaryOp::Min,
            0b001 => FpBinaryOp::Max,
            _ => return Err(()),
        };
        Ok(Instr::FpOp2(rd, rs1, rs2, RoundMode::None, pr, op))
    }

    #[cfg(feature = "D")]
    fn dec_fp_cvt_fp(&self, instr: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(instr);
        let rm = dec_round_mode(fn3)?;
        let from_pr = self.dec_precision(rs2)?;
        Ok(Instr::FpCvtFp(rd, rs1, rm, from_pr, pr))
    }

    #[cfg(feature = "F")]
    fn dec_fp_op(instr: u32, pr: Precision, op: FpUnaryOp) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(instr);
        let rm = dec_round_mode(fn3)?;
        if rs2 != 0 {
            return Err(());
        }
        Ok(Instr::FpOp(rd, rs1, rm, pr, op))
    }

    #[cfg(feature = "F")]
    fn dec_fp_cmp(instr: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(instr);
        let cond = match fn3 {
            0b000 => FpCmpCond::Le,
            0b001 => FpCmpCond::Lt,
            0b010 => FpCmpCond::Eq,
            _ => return Err(()),
        };
        Ok(Instr::FpCmp(rd, rs1, rs2, pr, cond))
    }

    #[cfg(feature = "F")]
    fn dec_fp_cvt_gp(instr: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(instr);
        check_gp_regs!(rd);
        let rm = dec_round_mode(fn3)?;
        let op = match rs2 {
            0b00 => FpGpOp::W,
            0b01 => FpGpOp::WU,
            0b10 => en_if_ge_rv64!(FpGpOp::L)?,
            0b11 => en_if_ge_rv64!(FpGpOp::LU)?,
            _ => return Err(()),
        };
        Ok(Instr::FpCvtGp(rd, rs1, rm, pr, op))
    }

    #[cfg(feature = "F")]
    fn dec_gp_cvt_fp(instr: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(instr);
        check_gp_regs!(rs1);
        let rm = dec_round_mode(fn3)?;
        let op = match rs2 {
            0b00 => GpFpOp::W,
            0b01 => GpFpOp::WU,
            0b10 => en_if_ge_rv64!(GpFpOp::L)?,
            0b11 => en_if_ge_rv64!(GpFpOp::LU)?,
            _ => return Err(()),
        };
        Ok(Instr::GpCvtFp(rd, rs1, rm, pr, op))
    }

    #[cfg(feature = "F")]
    fn dec_fp_mv_gp(instr: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(instr);
        check_gp_regs!(rd);
        let op = match fn3 {
            0b0 => match pr {
                Precision::S => FpGpOp::MV,
                #[cfg(feature = "D")]
                Precision::D => en_if_ge_rv64!(FpGpOp::MV)?,
            },
            0b1 => FpGpOp::Class,
            _ => return Err(()),
        };
        if rs2 != 0 {
            return Err(());
        }
        Ok(Instr::FpCvtGp(rd, rs1, RoundMode::None, pr, op))
    }

    #[cfg(feature = "F")]
    fn dec_gp_mv_fp(instr: u32, pr: Precision) -> Maybe<Instr> {
        let (rd, fn3, rs1, rs2, _) = r_type(instr);
        check_gp_regs!(rs1);
        if fn3 != 0 || rs2 != 0 {
            return Err(());
        }
        Ok(Instr::GpCvtFp(rd, rs1, RoundMode::None, pr, GpFpOp::MV))
    }

    fn dec_op_fp(&self, instr: u32) -> Maybe<Instr> {
        en_if_ext_f!(self, {
            let pr = self.dec_precision(funct2(instr))?;
            match rs3(instr) {
                0b0_0000 => Self::dec_fp_op2(instr, pr, FpBinaryOp::Add),
                0b0_0001 => Self::dec_fp_op2(instr, pr, FpBinaryOp::Sub),
                0b0_0010 => Self::dec_fp_op2(instr, pr, FpBinaryOp::Mul),
                0b0_0011 => Self::dec_fp_op2(instr, pr, FpBinaryOp::Div),
                0b0_0100 => Self::dec_fp_sgnj(instr, pr),
                0b0_0101 => Self::dec_fp_minmax(instr, pr),
                0b0_1000 => en_if_ext_d!(self, self.dec_fp_cvt_fp(instr, pr)?),
                0b0_1011 => Self::dec_fp_op(instr, pr, FpUnaryOp::Sqrt),
                0b1_0100 => Self::dec_fp_cmp(instr, pr),
                0b1_1000 => Self::dec_fp_cvt_gp(instr, pr),
                0b1_1010 => Self::dec_gp_cvt_fp(instr, pr),
                0b1_1100 => Self::dec_fp_mv_gp(instr, pr),
                0b1_1110 => Self::dec_gp_mv_fp(instr, pr),
                _ => Err(()),
            }
        }?)
    }

    fn dec_branch(instr: u32) -> Maybe<Instr> {
        let (fn3, rs1, rs2, imm) = b_type(instr);
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

    fn dec_jalr(instr: u32) -> Maybe<Instr> {
        let (rd, fn3, rs1, imm) = i_type(instr);
        check_gp_regs!(rd, rs1);
        if fn3 != 0 {
            return Err(());
        }
        Ok(Instr::Jalr(rd, rs1, imm))
    }

    fn dec_jal(instr: u32) -> Maybe<Instr> {
        let (rd, imm) = j_type(instr);
        check_gp_regs!(rd);
        Ok(Instr::Jal(rd, imm))
    }

    #[cfg(feature = "Zicsr")]
    fn dec_csr(&self, instr: u32) -> Maybe<Instr> {
        let rd = dec_rd(instr);
        let fn3 = funct3(instr);
        let rs1 = dec_rs1(instr);
        let addr = select_bits(instr, 31, 20) as u16;
        let csr_op = match fn3 {
            0b001 => CsrOp::Rw,
            0b010 => CsrOp::Rs,
            0b011 => CsrOp::Rc,
            0b101 => CsrOp::Rwi,
            0b110 => CsrOp::Rsi,
            0b111 => CsrOp::Rci,
            _ => return Err(()),
        };
        check_gp_regs!(rd);
        if fn3 < 0b100 {
            check_gp_regs!(rs1);
        }
        Ok(Instr::Csr(rd, rs1, addr, csr_op))
    }

    fn dec_system(&self, instr: u32) -> Maybe<Instr> {
        if funct3(instr) == 0 {
            match instr >> 7 {
                0b0 => Ok(Instr::Trap(Exception::Ecall)),
                0b10_0000_0000_0000 => Ok(Instr::Trap(Exception::Ebreak)),
                _ => Err(()),
            }
        } else {
            en_if_ext_zicsr!(self, self.dec_csr(instr)?)
        }
    }

    pub fn dec32(&self, instr: u32) -> Instr {
        match select_bits(instr, 6, 2) {
            0b0_0000 => Self::dec_load(instr),
            0b0_0001 => self.dec_load_fp(instr),
            0b0_0011 => self.dec_misc_mem(instr),
            0b0_0100 => Self::dec_op_imm(instr),
            0b0_0101 => Self::dec_auipc(instr),
            0b0_0110 => Self::dec_op_imm_32(instr),
            0b0_1000 => Self::dec_store(instr),
            0b0_1001 => self.dec_store_fp(instr),
            0b0_1011 => self.dec_amo(instr),
            0b0_1100 => self.dec_op(instr),
            0b0_1101 => Self::dec_lui(instr),
            0b0_1110 => self.dec_op_32(instr),
            0b1_0000 => self.dec_madd(instr),
            0b1_0001 => self.dec_msub(instr),
            0b1_0010 => self.dec_nmsub(instr),
            0b1_0011 => self.dec_nmadd(instr),
            0b1_0100 => self.dec_op_fp(instr),
            0b1_1000 => Self::dec_branch(instr),
            0b1_1001 => Self::dec_jalr(instr),
            0b1_1011 => Self::dec_jal(instr),
            0b1_1100 => self.dec_system(instr),
            _ => Err(()),
        }
        .unwrap_or(Instr::Trap(Exception::IllegalInstr))
    }
}
