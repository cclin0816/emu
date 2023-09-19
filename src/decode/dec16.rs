use crate::{decode::common::*, hart::HartIsa, uop::*, utils::Maybe, xlen::XlenT};

fn rhigh(ins: u16) -> u8 {
    select_bits(ins, 11, 7) as u8
}

fn crhigh(ins: u16) -> u8 {
    select_bits(ins, 9, 7) as u8 + 8
}

fn rlow(ins: u16) -> u8 {
    select_bits(ins, 6, 2) as u8
}

fn crlow(ins: u16) -> u8 {
    select_bits(ins, 4, 2) as u8 + 8
}

/// (crl, crh)
fn cregs(ins: u16) -> (u8, u8) {
    (crlow(ins), crhigh(ins))
}

fn fn3(ins: u16) -> u8 {
    select_bits(ins, 15, 13) as u8
}

fn j_imm(ins: u16) -> i32 {
    let imm = shuffle_bits!(ins, 1, 5, 3, 11, 11, 2, 2, 7, 7, 6, 6, 10, 9, 8, 8, 12, 12);
    sext(imm, 11)
}

fn ls4b_uimm(ins: u16) -> i32 {
    shuffle_bits!(ins, 2, 6, 6, 12, 10, 5, 5) as i32
}

fn ls8b_uimm(ins: u16) -> i32 {
    shuffle_bits!(ins, 3, 12, 10, 6, 5) as i32
}

fn op_imm6(ins: u16) -> i32 {
    let imm = shuffle_bits!(ins, 0, 6, 2, 12, 12);
    sext(imm, 5)
}

fn lsp4b_uimm(ins: u16) -> i32 {
    shuffle_bits!(ins, 2, 6, 4, 12, 12, 3, 2) as i32
}

fn lsp8b_uimm(ins: u16) -> i32 {
    shuffle_bits!(ins, 3, 6, 5, 12, 12, 4, 2) as i32
}

fn ssp4b_uimm(ins: u16) -> i32 {
    shuffle_bits!(ins, 2, 12, 9, 8, 7) as i32
}

fn ssp8b_uimm(ins: u16) -> i32 {
    shuffle_bits!(ins, 3, 12, 10, 9, 7) as i32
}

fn dec16_addi4spn(ins: u16) -> Maybe<Instr> {
    let rd = crlow(ins);
    let imm = shuffle_bits!(ins, 2, 6, 6, 5, 5, 12, 11, 10, 7) as i32;
    if imm == 0 {
        // canonical illegal instruction
        return Err(());
    }
    Ok(Instr::COpImm(rd, SP, imm, BinaryOp::Add))
}

fn dec16_lw(ins: u16) -> Maybe<Instr> {
    let (rd, rs1) = cregs(ins);
    Ok(Instr::CLoad(rd, rs1, ls4b_uimm(ins), MemWidth::W))
}

fn dec_sw(ins: u16) -> Maybe<Instr> {
    let (rs2, rs1) = cregs(ins);
    Ok(Instr::CStore(rs1, rs2, ls4b_uimm(ins), MemWidth::W))
}

fn dec16_addi(ins: u16) -> Maybe<Instr> {
    let rd_rs1 = rhigh(ins);
    Ok(Instr::COpImm(rd_rs1, rd_rs1, op_imm6(ins), BinaryOp::Add))
}

fn dec16_li(ins: u16) -> Maybe<Instr> {
    Ok(Instr::COpImm(rhigh(ins), ZERO, op_imm6(ins), BinaryOp::Add))
}

fn dec16_addi16sp_lui(ins: u16) -> Maybe<Instr> {
    let rd = rhigh(ins);
    if rd == 2 {
        // addi16sp
        let imm = shuffle_bits!(ins, 4, 6, 6, 2, 2, 5, 5, 4, 3, 12, 12);
        let imm = sext(imm, 9);
        if imm == 0 {
            return Err(());
        }
        Ok(Instr::COpImm(SP, SP, imm, BinaryOp::Add))
    } else {
        // lui
        let imm = shuffle_bits!(ins, 12, 6, 2, 12, 12);
        let imm = sext(imm, 17);
        if imm == 0 {
            return Err(());
        }
        Ok(Instr::COpImm(rd, ZERO, imm, BinaryOp::Add))
    }
}

fn dec16_andi(ins: u16, rd_rs1: u8) -> Maybe<Instr> {
    Ok(Instr::COpImm(rd_rs1, rd_rs1, op_imm6(ins), BinaryOp::And))
}

fn dec_j(ins: u16) -> Maybe<Instr> {
    Ok(Instr::CJal(ZERO, j_imm(ins)))
}

fn dec_branch(ins: u16, cond: CmpCond) -> Maybe<Instr> {
    let rs1 = crhigh(ins);
    let imm = shuffle_bits!(ins, 1, 4, 3, 11, 10, 2, 2, 6, 5, 12, 12);
    let imm = sext(imm, 8);
    Ok(Instr::CBranch(rs1, imm, cond))
}

fn dec16_lwsp(ins: u16) -> Maybe<Instr> {
    let rd = rhigh(ins);
    if rd == 0 {
        return Err(());
    }
    Ok(Instr::CLoad(rd, SP, lsp4b_uimm(ins), MemWidth::W))
}

fn dec16_misc(ins: u16) -> Maybe<Instr> {
    let rs1 = rhigh(ins);
    let rs2 = rlow(ins);
    Ok(match (test_bit(ins, 12), rs1, rs2) {
        (false, 0, 0) => return Err(()),
        (false, rs1, 0) => Instr::CJalr(ZERO, rs1),
        (false, rd, rs1) => Instr::COpImm(rd, rs1, 0, BinaryOp::Add),
        (true, 0, 0) => Instr::CTrap(Exception::Ebreak),
        (true, rs1, 0) => Instr::CJalr(RA, rs1),
        (true, rd_rs1, rs2) => Instr::COp(rd_rs1, rs2, BinaryOp::Add),
    })
}

fn dec16_swsp(ins: u16) -> Maybe<Instr> {
    Ok(Instr::CStore(SP, rlow(ins), ssp4b_uimm(ins), MemWidth::W))
}

impl<Xlen: XlenT> HartIsa<Xlen> {
    fn dec16_lq_fld(&self, ins: u16) -> Maybe<Instr> {
        let (rd, rs1) = cregs(ins);
        if_rv128!(
            // lq
            unimplemented!("rv128 not implemented yet"),
            // fld
            if_ext_d!(self, Instr::CLoadFp(rd, rs1, ls8b_uimm(ins), Precision::D))
        )
    }

    fn dec16_ld_flw(&self, ins: u16) -> Maybe<Instr> {
        let (rd, rs1) = cregs(ins);
        if_ge_rv64!(
            // ld
            Ok(Instr::CLoad(rd, rs1, ls8b_uimm(ins), MemWidth::D)),
            // flw
            if_ext_f!(self, Instr::CLoadFp(rd, rs1, ls4b_uimm(ins), Precision::S))
        )
    }

    fn dec16_sq_fsd(&self, ins: u16) -> Maybe<Instr> {
        let (rs2, rs1) = cregs(ins);
        if_rv128!(
            // sq
            unimplemented!("rv128 not implemented yet"),
            // fsd
            if_ext_d!(
                self,
                Instr::CStoreFp(rs1, rs2, ls8b_uimm(ins), Precision::D)
            )
        )
    }

    fn dec16_sd_fsw(&self, ins: u16) -> Maybe<Instr> {
        let (rs2, rs1) = cregs(ins);
        if_ge_rv64!(
            // sd
            Ok(Instr::CStore(rs1, rs2, ls8b_uimm(ins), MemWidth::D)),
            // fsw
            if_ext_f!(
                self,
                Instr::CStoreFp(rs1, rs2, ls4b_uimm(ins), Precision::S)
            )
        )
    }

    fn dec16_c0(&self, ins: u16) -> Maybe<Instr> {
        match fn3(ins) {
            0b000 => dec16_addi4spn(ins),
            0b001 => self.dec16_lq_fld(ins),
            0b010 => dec16_lw(ins),
            0b011 => self.dec16_ld_flw(ins),
            0b101 => self.dec16_sq_fsd(ins),
            0b110 => dec_sw(ins),
            0b111 => self.dec16_sd_fsw(ins),
            _ => Err(()),
        }
    }

    fn dec16_jal_addiw(ins: u16) -> Maybe<Instr> {
        Ok(if_ge_rv64!(
            {
                // addiw
                let rd_rs1 = rhigh(ins);
                if rd_rs1 == 0 {
                    return Err(());
                }
                Instr::COpImm(rd_rs1, rd_rs1, op_imm6(ins), BinaryOp::AddW)
            },
            // jal
            Instr::CJal(RA, j_imm(ins))
        ))
    }

    fn dec16_shamt(ins: u16) -> Maybe<i32> {
        let imm = shuffle_bits!(ins, 0, 6, 2, 12, 12) as i32;
        Ok(if_rv128!(
            {
                if imm == 0 {
                    64
                } else if imm >= 32 {
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

    fn dec16_srli(ins: u16, rd_rs1: u8) -> Maybe<Instr> {
        let imm = Self::dec16_shamt(ins)?;
        Ok(Instr::COpImm(rd_rs1, rd_rs1, imm, BinaryOp::Srl))
    }

    fn dec16_srai(ins: u16, rd_rs1: u8) -> Maybe<Instr> {
        let imm = Self::dec16_shamt(ins)?;
        Ok(Instr::COpImm(rd_rs1, rd_rs1, imm, BinaryOp::Sra))
    }

    fn dec16_op(ins: u16, rd_rs1: u8) -> Maybe<Instr> {
        let rs2 = crlow(ins);
        let op = match (test_bit(ins, 12), select_bits(ins, 6, 5)) {
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

    fn dec16_misc_alu(ins: u16) -> Maybe<Instr> {
        let rd_rs1 = crhigh(ins);
        match select_bits(ins, 11, 10) {
            0b00 => Self::dec16_srli(ins, rd_rs1),
            0b01 => Self::dec16_srai(ins, rd_rs1),
            0b10 => dec16_andi(ins, rd_rs1),
            0b11 => Self::dec16_op(ins, rd_rs1),
            _ => unreachable!(),
        }
    }

    fn dec16_c1(ins: u16) -> Maybe<Instr> {
        match fn3(ins) {
            0b000 => dec16_addi(ins),
            0b001 => Self::dec16_jal_addiw(ins),
            0b010 => dec16_li(ins),
            0b011 => dec16_addi16sp_lui(ins),
            0b100 => Self::dec16_misc_alu(ins),
            0b101 => dec_j(ins),
            0b110 => dec_branch(ins, CmpCond::Eq),
            0b111 => dec_branch(ins, CmpCond::Ne),
            _ => unreachable!(),
        }
    }

    fn dec16_slli(ins: u16) -> Maybe<Instr> {
        let rd_rs1 = rhigh(ins);
        let imm = Self::dec16_shamt(ins)?;
        Ok(Instr::COpImm(rd_rs1, rd_rs1, imm, BinaryOp::Sll))
    }

    fn dec16_lq_fld_sp(&self, ins: u16) -> Maybe<Instr> {
        let rd = rhigh(ins);
        if_rv128!(
            // lqsp
            unimplemented!("rv128 not implemented yet"),
            // fldsp
            if_ext_d!(self, Instr::CLoadFp(rd, SP, lsp8b_uimm(ins), Precision::D))
        )
    }

    fn dec16_ld_flw_sp(&self, ins: u16) -> Maybe<Instr> {
        let rd = rhigh(ins);
        if_ge_rv64!(
            {
                // ldsp
                if rd == 0 {
                    return Err(());
                }
                Ok(Instr::CLoad(rd, SP, lsp8b_uimm(ins), MemWidth::D))
            },
            // flwsp
            if_ext_f!(self, Instr::CLoadFp(rd, SP, lsp4b_uimm(ins), Precision::S))
        )
    }

    fn dec16_sq_fsd_sp(&self, ins: u16) -> Maybe<Instr> {
        let rs2 = rlow(ins);
        if_rv128!(
            // sqsp
            unimplemented!("rv128 not implemented yet"),
            // fsdsp
            if_ext_d!(
                self,
                Instr::CStoreFp(SP, rs2, ssp8b_uimm(ins), Precision::D)
            )
        )
    }

    fn dec16_sd_fsw_sp(&self, ins: u16) -> Maybe<Instr> {
        let rs2 = rlow(ins);
        if_ge_rv64!(
            // sdsp
            Ok(Instr::CStore(SP, rs2, ssp8b_uimm(ins), MemWidth::D)),
            // fswsp
            if_ext_f!(
                self,
                Instr::CStoreFp(SP, rs2, ssp4b_uimm(ins), Precision::S)
            )
        )
    }

    fn dec16_c2(&self, ins: u16) -> Maybe<Instr> {
        match fn3(ins) {
            0b000 => Self::dec16_slli(ins),
            0b001 => self.dec16_lq_fld_sp(ins),
            0b010 => dec16_lwsp(ins),
            0b011 => self.dec16_ld_flw_sp(ins),
            0b100 => dec16_misc(ins),
            0b101 => self.dec16_sq_fsd_sp(ins),
            0b110 => dec16_swsp(ins),
            0b111 => self.dec16_sd_fsw_sp(ins),
            _ => unreachable!(),
        }
    }

    pub fn dec16(&self, ins: u16) -> Instr {
        match select_bits(ins, 1, 0) {
            0b00 => self.dec16_c0(ins),
            0b01 => Self::dec16_c1(ins),
            0b10 => self.dec16_c2(ins),
            _ => Err(()),
        }
        .unwrap_or(Instr::CTrap(Exception::IllegalInstr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type RV32I = HartIsa<u32>;
    type RV64I = HartIsa<u64>;
    const ILL: Instr = Instr::CTrap(Exception::IllegalInstr);

    fn all_pass<Xlen: XlenT>(hart: &HartIsa<Xlen>, ins_raw: &[u16], ins_dec: &[Instr]) -> bool {
        for (raw, dec) in ins_raw.iter().zip(ins_dec.iter()) {
            if hart.dec16(*raw) != *dec {
                println!(
                    "raw: {:04x}, exp: {:?}, dec: {:?}",
                    raw,
                    dec,
                    hart.dec16(*raw)
                );
                return false;
            }
        }
        true
    }
    fn all_fail<Xlen: XlenT>(hart: &HartIsa<Xlen>, ins_raw: &[u16]) -> bool {
        for raw in ins_raw.iter() {
            if hart.dec16(*raw) != ILL {
                println!("raw: {:04x}, exp: ILL, dec: {:?}", raw, hart.dec16(*raw));
                return false;
            }
        }
        true
    }

    #[test]
    fn t1() {
        let ins_raw = [
            0x0ac8u16, 0x5aa8u16, 0xdaa8u16, 0x1aa9u16, 0x5aa9u16, 0x710du16, 0x7aa9u16, 0x8155u16,
            0x8555u16, 0x8955u16, 0x8d15u16, 0x8d35u16, 0x8d55u16, 0x8d75u16, 0xb46du16, 0xd931u16,
            0xf931u16, 0x0ad6u16, 0x5aceu16, 0x8a82u16, 0x8aeeu16, 0x9002u16, 0x9a82u16, 0x9aeeu16,
            0xd9d6u16,
        ];
        let ins_dec = [
            Instr::COpImm(10, 2, 340, BinaryOp::Add),
            Instr::CLoad(10, 13, 112, MemWidth::W),
            Instr::CStore(13, 10, 112, MemWidth::W),
            Instr::COpImm(21, 21, -22, BinaryOp::Add),
            Instr::COpImm(21, 0, -22, BinaryOp::Add),
            Instr::COpImm(2, 2, -352, BinaryOp::Add),
            Instr::COpImm(21, 0, -90112, BinaryOp::Add),
            Instr::COpImm(10, 10, 21, BinaryOp::Srl),
            Instr::COpImm(10, 10, 21, BinaryOp::Sra),
            Instr::COpImm(10, 10, 21, BinaryOp::And),
            Instr::COp(10, 13, BinaryOp::Sub),
            Instr::COp(10, 13, BinaryOp::Xor),
            Instr::COp(10, 13, BinaryOp::Or),
            Instr::COp(10, 13, BinaryOp::And),
            Instr::CJal(0, -1366),
            Instr::CBranch(10, -172, CmpCond::Eq),
            Instr::CBranch(10, -172, CmpCond::Ne),
            Instr::COpImm(21, 21, 21, BinaryOp::Sll),
            Instr::CLoad(21, 2, 240, MemWidth::W),
            Instr::CJalr(0, 21),
            Instr::COpImm(21, 27, 0, BinaryOp::Add),
            Instr::CTrap(Exception::Ebreak),
            Instr::CJalr(1, 21),
            Instr::COp(21, 27, BinaryOp::Add),
            Instr::CStore(2, 21, 240, MemWidth::W),
        ];
        assert!(all_pass(&RV32I::default(), &ins_raw, &ins_dec));
        assert!(all_pass(&RV64I::default(), &ins_raw, &ins_dec));

        #[cfg(feature = "D")]
        {
            let ins_raw = [0x3aa8u16, 0xbaa8u16, 0x3aceu16, 0xb9d6u16];
            let ins_dec = [
                Instr::CLoadFp(10, 13, 112, Precision::D),
                Instr::CStoreFp(13, 10, 112, Precision::D),
                Instr::CLoadFp(21, SP, 240, Precision::D),
                Instr::CStoreFp(SP, 21, 240, Precision::D),
            ];
            let mut hart = RV32I::default();
            hart.D = true;
            assert!(all_pass(&hart, &ins_raw, &ins_dec));
            let mut hart = RV64I::default();
            hart.D = true;
            assert!(all_pass(&hart, &ins_raw, &ins_dec));
        }
    }

    #[test]
    fn t2() {
        let ins_raw = [
            0x3801u16, 0x7ea8u16, 0xfea8u16, 0x7aceu16, 0xf9d6u16, 0x9115u16, 0x9515u16, 0x9d15u16,
            0x9d35u16, 0x1516u16,
        ];
        let ins_dec_64 = [
            Instr::COpImm(16, 16, -32, BinaryOp::AddW),
            Instr::CLoad(10, 13, 120, MemWidth::D),
            Instr::CStore(13, 10, 120, MemWidth::D),
            Instr::CLoad(21, 2, 240, MemWidth::D),
            Instr::CStore(2, 21, 240, MemWidth::D),
            Instr::COpImm(10, 10, 37, BinaryOp::Srl),
            Instr::COpImm(10, 10, 37, BinaryOp::Sra),
            Instr::COp(10, 13, BinaryOp::SubW),
            Instr::COp(10, 13, BinaryOp::AddW),
            Instr::COpImm(10, 10, 37, BinaryOp::Sll),
        ];
        assert!(all_pass(&RV64I::default(), &ins_raw, &ins_dec_64));
        assert!(all_fail(&RV32I::default(), &ins_raw[1..]));
        assert_eq!(RV32I::default().dec16(ins_raw[0]), Instr::CJal(1, -2032));

        #[cfg(feature = "F")]
        {
            let ins_dec_32 = [
                Instr::CLoadFp(10, 13, 120, Precision::S),
                Instr::CStoreFp(13, 10, 120, Precision::S),
                Instr::CLoadFp(21, 2, 240, Precision::S),
                Instr::CStoreFp(2, 21, 240, Precision::S),
            ];
            let mut hart = RV32I::default();
            hart.F = true;
            assert!(all_pass(&hart, &ins_raw[1..6], &ins_dec_32));
        }
    }
}
