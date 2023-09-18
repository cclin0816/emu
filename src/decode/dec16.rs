use crate::{decode::common::*, hart::HartIsa, uop::*, utils::Maybe, xlen::XlenT};

fn dec_rs1(instr: u16) -> u8 {
    select_bits(instr, 11, 7) as u8
}

fn dec_crs1(instr: u16) -> u8 {
    select_bits(instr, 9, 7) as u8 + 8
}

fn dec_rs2(instr: u16) -> u8 {
    select_bits(instr, 6, 2) as u8
}

fn dec_crs2(instr: u16) -> u8 {
    select_bits(instr, 4, 2) as u8 + 8
}

fn funct3(instr: u16) -> u8 {
    select_bits(instr, 15, 13) as u8
}

fn dec_addi4spn(instr: u16) -> Maybe<Instr> {
    let rd = dec_crs2(instr);
    let imm = shuffle_bits!(instr, 2, 6, 6, 5, 5, 12, 11, 10, 7);
    if imm == 0 {
        return Err(());
    }
    Ok(Instr::COpImm(rd, 2, imm as i32, BinaryOp::Add))
}

fn dec_lw(instr: u16) -> Maybe<Instr> {
    let rd = dec_crs2(instr);
    let rs1 = dec_crs1(instr);
    let imm = shuffle_bits!(instr, 2, 6, 6, 12, 10, 5, 5);
    Ok(Instr::CLoad(rd, rs1, imm as i32, MemWidth::W))
}

fn dec_sw(instr: u16) -> Maybe<Instr> {
    let rs2 = dec_crs2(instr);
    let rs1 = dec_crs1(instr);
    let imm = shuffle_bits!(instr, 2, 6, 6, 12, 10, 5, 5);
    Ok(Instr::CStore(rs2, rs1, imm as i32, MemWidth::W))
}

fn dec_j(instr: u16) -> Maybe<Instr> {
    let imm = shuffle_bits!(instr, 1, 5, 3, 11, 11, 2, 2, 7, 7, 6, 6, 10, 9, 8, 8, 12, 12);
    let imm = sext(imm as u32, 11);
    Ok(Instr::CJal(0, imm))
}

fn dec_branch(instr: u16, cond: CmpCond) -> Maybe<Instr> {
    let rs1 = dec_crs1(instr);
    let imm = shuffle_bits!(instr, 1, 4, 3, 11, 10, 2, 2, 6, 5, 12, 12);
    let imm = sext(imm as u32, 8);
    Ok(Instr::CBranch(rs1, imm, cond))
}

impl<Xlen: XlenT, const EMB: bool> HartIsa<Xlen, EMB> {
    fn dec16_lq_fld(&self, instr: u16) -> Maybe<Instr> {
        let rd = dec_crs2(instr);
        let rs1 = dec_crs1(instr);
        Ok(if_rv128!(
            unimplemented!("rv128 not implemented yet"),
            if_ext_d!(self, {
                let imm = shuffle_bits!(instr, 3, 12, 10, 6, 5);
                Instr::CLoadFp(rd, rs1, imm as i32, Precision::D)
            })?
        ))
    }

    fn dec16_ld_flw(&self, instr: u16) -> Maybe<Instr> {
        let rd = dec_crs2(instr);
        let rs1 = dec_crs1(instr);
        Ok(if_ge_rv64!(
            {
                let imm = shuffle_bits!(instr, 3, 12, 10, 6, 5);
                Instr::CLoad(rd, rs1, imm as i32, MemWidth::D)
            },
            if_ext_f!(self, {
                let imm = shuffle_bits!(instr, 2, 6, 6, 12, 10, 5, 5);
                Instr::CLoadFp(rd, rs1, imm as i32, Precision::S)
            })?
        ))
    }

    fn dec16_sq_fsd(&self, instr: u16) -> Maybe<Instr> {
        let rs2 = dec_crs2(instr);
        let rs1 = dec_crs1(instr);
        Ok(if_rv128!(
            unimplemented!("rv128 not implemented yet"),
            if_ext_d!(self, {
                let imm = shuffle_bits!(instr, 3, 12, 10, 6, 5);
                Instr::CStoreFp(rs2, rs1, imm as i32, Precision::D)
            })?
        ))
    }

    fn dec16_sd_fsw(&self, instr: u16) -> Maybe<Instr> {
        let rs2 = dec_crs2(instr);
        let rs1 = dec_crs1(instr);
        Ok(if_ge_rv64!(
            {
                let imm = shuffle_bits!(instr, 3, 12, 10, 6, 5);
                Instr::CStore(rs2, rs1, imm as i32, MemWidth::D)
            },
            if_ext_f!(self, {
                let imm = shuffle_bits!(instr, 2, 6, 6, 12, 10, 5, 5);
                Instr::CStoreFp(rs2, rs1, imm as i32, Precision::S)
            })?
        ))
    }

    fn dec16_c0(&self, instr: u16) -> Maybe<Instr> {
        match funct3(instr) {
            0b000 => dec_addi4spn(instr),
            0b001 => self.dec16_lq_fld(instr),
            0b010 => dec_lw(instr),
            0b011 => self.dec16_ld_flw(instr),
            0b101 => self.dec16_sq_fsd(instr),
            0b110 => dec_sw(instr),
            0b111 => self.dec16_sd_fsw(instr),
            _ => Err(()),
        }
    }

    fn dec16_addi(instr: u16) -> Maybe<Instr> {
        let rd_rs1 = dec_rs1(instr);
        let imm = shuffle_bits!(instr, 0, 6, 2, 12, 12) as u32;
        let imm = sext(imm, 5);
        if imm == 0 {
            return Ok(Instr::Nop);
        }
        check_gp_regs!(rd_rs1);
        Ok(Instr::COpImm(rd_rs1, rd_rs1, imm, BinaryOp::Add))
    }

    fn dec16_jal_addiw(instr: u16) -> Maybe<Instr> {
        Ok(if_ge_rv64!(
            {
                let rd_rs1 = dec_rs1(instr);
                if rd_rs1 == 0 {
                    return Err(());
                }
                check_gp_regs!(rd_rs1);
                let imm = shuffle_bits!(instr, 0, 6, 2, 12, 12) as u32;
                let imm = sext(imm, 5);
                Instr::COpImm(rd_rs1, rd_rs1, imm, BinaryOp::AddW)
            },
            {
                let imm =
                    shuffle_bits!(instr, 1, 5, 3, 11, 11, 2, 2, 7, 7, 6, 6, 10, 9, 8, 8, 12, 12);
                let imm = sext(imm as u32, 11);
                Instr::CJal(1, imm)
            }
        ))
    }

    fn dec16_li(instr: u16) -> Maybe<Instr> {
        let rd = dec_rs1(instr);
        check_gp_regs!(rd);
        let imm = shuffle_bits!(instr, 0, 6, 2, 12, 12) as u32;
        let imm = sext(imm, 5);
        Ok(Instr::COpImm(rd, 0, imm, BinaryOp::Add))
    }

    fn dec16_addi16sp_lui(instr: u16) -> Maybe<Instr> {
        let rd = dec_rs1(instr);
        if rd == 2 {
            let imm = shuffle_bits!(instr, 5, 6, 6, 2, 2, 5, 5, 4, 3, 12, 12) as u32;
            let imm = sext(imm, 9);
            if imm == 0 {
                return Err(());
            }
            Ok(Instr::COpImm(2, 2, imm, BinaryOp::Add))
        } else {
            check_gp_regs!(rd);
            let imm = shuffle_bits!(instr, 12, 6, 2, 12, 12) as u32;
            let imm = sext(imm, 17);
            if imm == 0 {
                return Err(());
            }
            Ok(Instr::COpImm(rd, 0, imm, BinaryOp::Add))
        }
    }

    fn dec16_shamt(instr: u16) -> Maybe<i32> {
        let imm = shuffle_bits!(instr, 0, 6, 2, 12, 12) as i32;
        Ok(if_rv128!(
            {
                if imm == 0 {
                    64
                } else if test_bit(instr, 12) {
                    imm + 64
                } else {
                    imm
                }
            },
            if_ge_rv64!(imm, {
                if imm >= 32 {
                    return Err(());
                }
                imm
            })
        ))
    }

    fn dec16_srli(instr: u16, rd_rs1: u8) -> Maybe<Instr> {
        let imm = Self::dec16_shamt(instr)?;
        Ok(Instr::COpImm(rd_rs1, rd_rs1, imm, BinaryOp::Srl))
    }

    fn dec16_srai(instr: u16, rd_rs1: u8) -> Maybe<Instr> {
        let imm = Self::dec16_shamt(instr)?;
        Ok(Instr::COpImm(rd_rs1, rd_rs1, imm, BinaryOp::Sra))
    }

    fn dec16_andi(instr: u16, rd_rs1: u8) -> Maybe<Instr> {
        let imm = shuffle_bits!(instr, 0, 6, 2, 12, 12) as u32;
        let imm = sext(imm, 5);
        Ok(Instr::COpImm(rd_rs1, rd_rs1, imm, BinaryOp::And))
    }

    fn dec16_op(instr: u16, rd_rs1: u8) -> Maybe<Instr> {
        let rs2 = dec_crs2(instr);
        let op = match (test_bit(instr, 12), select_bits(instr, 6, 5)) {
            (false, 0b00) => BinaryOp::Sub,
            (false, 0b01) => BinaryOp::Xor,
            (false, 0b10) => BinaryOp::Or,
            (false, 0b11) => BinaryOp::And,
            (true, 0b00) => if_ge_rv64!(BinaryOp::SubW)?,
            (true, 0b01) => if_ge_rv64!(BinaryOp::AddW)?,
            _ => return Err(()),
        };
        Ok(Instr::COp(rd_rs1, rs2, op))
    }

    fn dec16_misc_alu(instr: u16) -> Maybe<Instr> {
        let rd_rs1 = dec_crs1(instr);
        match select_bits(instr, 11, 10) {
            0b00 => Self::dec16_srli(instr, rd_rs1),
            0b01 => Self::dec16_srai(instr, rd_rs1),
            0b10 => Self::dec16_andi(instr, rd_rs1),
            0b11 => Self::dec16_op(instr, rd_rs1),
            _ => unreachable!(),
        }
    }

    fn dec16_c1(instr: u16) -> Maybe<Instr> {
        match funct3(instr) {
            0b000 => Self::dec16_addi(instr),
            0b001 => Self::dec16_jal_addiw(instr),
            0b010 => Self::dec16_li(instr),
            0b011 => Self::dec16_addi16sp_lui(instr),
            0b100 => Self::dec16_misc_alu(instr),
            0b101 => dec_j(instr),
            0b110 => dec_branch(instr, CmpCond::Eq),
            0b111 => dec_branch(instr, CmpCond::Ne),
            _ => unreachable!(),
        }
    }

    fn dec16_slli(instr: u16) -> Maybe<Instr> {
        let rd_rs1 = dec_rs1(instr);
        check_gp_regs!(rd_rs1);
        let imm = Self::dec16_shamt(instr)?;
        Ok(Instr::COpImm(rd_rs1, rd_rs1, imm, BinaryOp::Sll))
    }

    fn dec16_lq_fld_sp(&self, instr: u16) -> Maybe<Instr> {
        let rd = dec_rs1(instr);
        Ok(if_rv128!(
            unimplemented!("rv128 not implemented yet"),
            if_ext_d!(self, {
                let imm = shuffle_bits!(instr, 3, 6, 5, 12, 12, 4, 2);
                Instr::CLoadFp(rd, 2, imm as i32, Precision::D)
            })?
        ))
    }

    fn dec16_lw_sp(instr: u16) -> Maybe<Instr> {
        let rd = dec_rs1(instr);
        check_gp_regs!(rd);
        if rd == 0 {
            return Err(());
        }
        let imm = shuffle_bits!(instr, 2, 6, 4, 12, 12, 3, 2);
        Ok(Instr::CLoad(rd, 2, imm as i32, MemWidth::W))
    }

    fn dec16_ld_flw_sp(&self, instr: u16) -> Maybe<Instr> {
        let rd = dec_rs1(instr);
        Ok(if_ge_rv64!(
            {
                check_gp_regs!(rd);
                if rd == 0 {
                    return Err(());
                }
                let imm = shuffle_bits!(instr, 3, 6, 5, 12, 12, 4, 2);
                Instr::CLoad(rd, 2, imm as i32, MemWidth::D)
            },
            if_ext_f!(self, {
                let imm = shuffle_bits!(instr, 2, 6, 4, 12, 12, 3, 2);
                Instr::CLoadFp(rd, 2, imm as i32, Precision::S)
            })?
        ))
    }

    fn dec16_misc(instr: u16) -> Maybe<Instr> {
        let rs1 = dec_rs1(instr);
        let rs2 = dec_rs2(instr);
        check_gp_regs!(rs1, rs2);
        Ok(match (test_bit(instr, 12), rs1, rs2) {
            (false, 0, 0) => return Err(()),
            (false, rs1, 0) => Instr::CJalr(0, rs1),
            (false, rd, rs1) => Instr::COpImm(rd, rs1, 0, BinaryOp::Add),
            (true, 0, 0) => Instr::CTrap(Exception::Ebreak),
            (true, rs1, 0) => Instr::CJalr(1, rs1),
            (true, rd_rs1, rs2) => Instr::COp(rd_rs1, rs2, BinaryOp::Add),
        })
    }

    fn dec16_sq_fsd_sp(&self, instr: u16) -> Maybe<Instr> {
        let rs2 = dec_rs2(instr);
        Ok(if_rv128!(
            unimplemented!("rv128 not implemented yet"),
            if_ext_d!(self, {
                let imm = shuffle_bits!(instr, 3, 12, 10, 9, 7);
                Instr::CStoreFp(rs2, 2, imm as i32, Precision::D)
            })?
        ))
    }

    fn dec16_sw_sp(instr: u16) -> Maybe<Instr> {
        let rs2 = dec_rs2(instr);
        check_gp_regs!(rs2);
        let imm = shuffle_bits!(instr, 2, 12, 9, 8, 7);
        Ok(Instr::CStore(rs2, 2, imm as i32, MemWidth::W))
    }

    fn dec16_sd_fsw_sp(&self, instr: u16) -> Maybe<Instr> {
        let rs2 = dec_rs2(instr);
        Ok(if_ge_rv64!(
            {
                check_gp_regs!(rs2);
                let imm = shuffle_bits!(instr, 3, 12, 10, 9, 7);
                Instr::CStore(rs2, 2, imm as i32, MemWidth::D)
            },
            if_ext_f!(self, {
                let imm = shuffle_bits!(instr, 2, 12, 9, 8, 7);
                Instr::CStoreFp(rs2, 2, imm as i32, Precision::S)
            })?
        ))
    }

    fn dec16_c2(&self, instr: u16) -> Maybe<Instr> {
        match funct3(instr) {
            0b000 => Self::dec16_slli(instr),
            0b001 => self.dec16_lq_fld_sp(instr),
            0b010 => Self::dec16_lw_sp(instr),
            0b011 => self.dec16_ld_flw_sp(instr),
            0b100 => Self::dec16_misc(instr),
            0b101 => self.dec16_sq_fsd_sp(instr),
            0b110 => Self::dec16_sw_sp(instr),
            0b111 => self.dec16_sd_fsw_sp(instr),
            _ => unreachable!(),
        }
    }

    pub fn dec16(&self, instr: u16) -> Instr {
        match select_bits(instr, 1, 0) {
            0b00 => self.dec16_c0(instr),
            0b01 => Self::dec16_c1(instr),
            0b10 => self.dec16_c2(instr),
            _ => Err(()),
        }
        .unwrap_or(Instr::Trap(Exception::IllegalInstr))
    }
}
