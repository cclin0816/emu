use crate::{decode::common::*, uop::*, utils::Maybe, xlen::XlenT};

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
    Ok(Instr::COpImm(rd, GP_SP, imm, BinaryOp::Add))
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
    Ok(Instr::COpImm(
        rhigh(ins),
        GP_ZERO,
        op_imm6(ins),
        BinaryOp::Add,
    ))
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
        Ok(Instr::COpImm(GP_SP, GP_SP, imm, BinaryOp::Add))
    } else {
        // lui
        let imm = shuffle_bits!(ins, 12, 6, 2, 12, 12);
        let imm = sext(imm, 17);
        if imm == 0 {
            return Err(());
        }
        Ok(Instr::COpImm(rd, GP_ZERO, imm, BinaryOp::Add))
    }
}

fn dec16_andi(ins: u16, rd_rs1: u8) -> Maybe<Instr> {
    Ok(Instr::COpImm(rd_rs1, rd_rs1, op_imm6(ins), BinaryOp::And))
}

fn dec_j(ins: u16) -> Maybe<Instr> {
    Ok(Instr::CJal(GP_ZERO, j_imm(ins)))
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
    Ok(Instr::CLoad(rd, GP_SP, lsp4b_uimm(ins), MemWidth::W))
}

fn dec16_misc(ins: u16) -> Maybe<Instr> {
    let rs1 = rhigh(ins);
    let rs2 = rlow(ins);
    Ok(match (test_bit(ins, 12), rs1, rs2) {
        (false, 0, 0) => return Err(()),
        (false, rs1, 0) => Instr::CJalr(GP_ZERO, rs1),
        (false, rd, rs1) => Instr::COpImm(rd, rs1, 0, BinaryOp::Add),
        (true, 0, 0) => Instr::Trap(Exception::Ebreak),
        (true, rs1, 0) => Instr::CJalr(GP_RA, rs1),
        (true, rd_rs1, rs2) => Instr::COp(rd_rs1, rs2, BinaryOp::Add),
    })
}

fn dec16_swsp(ins: u16) -> Maybe<Instr> {
    Ok(Instr::CStore(
        GP_SP,
        rlow(ins),
        ssp4b_uimm(ins),
        MemWidth::W,
    ))
}

impl<Xlen: XlenT> Isa<Xlen> {
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
            Instr::CJal(GP_RA, j_imm(ins))
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
            if_ext_d!(
                self,
                Instr::CLoadFp(rd, GP_SP, lsp8b_uimm(ins), Precision::D)
            )
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
                Ok(Instr::CLoad(rd, GP_SP, lsp8b_uimm(ins), MemWidth::D))
            },
            // flwsp
            if_ext_f!(
                self,
                Instr::CLoadFp(rd, GP_SP, lsp4b_uimm(ins), Precision::S)
            )
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
                Instr::CStoreFp(GP_SP, rs2, ssp8b_uimm(ins), Precision::D)
            )
        )
    }

    fn dec16_sd_fsw_sp(&self, ins: u16) -> Maybe<Instr> {
        let rs2 = rlow(ins);
        if_ge_rv64!(
            // sdsp
            Ok(Instr::CStore(GP_SP, rs2, ssp8b_uimm(ins), MemWidth::D)),
            // fswsp
            if_ext_f!(
                self,
                Instr::CStoreFp(GP_SP, rs2, ssp4b_uimm(ins), Precision::S)
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
        .unwrap_or(Instr::Trap(Exception::IllegalInstr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type RV32 = Isa<u32>;
    #[cfg(feature = "RV64")]
    type RV64 = Isa<u64>;

    fn all_pass<Xlen: XlenT>(hart: &Isa<Xlen>, ins_raw: &[u16], ins_expect: &[Instr]) -> bool {
        for (&raw, &expect) in ins_raw.iter().zip(ins_expect.iter()) {
            let result = hart.dec16(raw);
            if result != expect {
                println!(
                    " raw: {:04x},\n expected: {:?},\n result: {:?}",
                    raw, expect, result
                );
                return false;
            }
        }
        true
    }

    #[test]
    fn sanity() {
        // rv32 == rv64
        let ins_raw = [
            0x0ac8u16, 0x5aa8u16, 0xdaa8u16, 0x1aa9u16, 0x5aa9u16, 0x710du16, 0x7aa9u16, 0x8155u16,
            0x8555u16, 0x8955u16, 0x8d15u16, 0x8d35u16, 0x8d55u16, 0x8d75u16, 0xb46du16, 0xd931u16,
            0xf931u16, 0x0ad6u16, 0x5aceu16, 0x8a82u16, 0x8aeeu16, 0x9002u16, 0x9a82u16, 0x9aeeu16,
            0xd9d6u16,
        ];
        let ins_expect = [
            Instr::COpImm(10, GP_SP, 340, BinaryOp::Add),
            Instr::CLoad(10, 13, 112, MemWidth::W),
            Instr::CStore(13, 10, 112, MemWidth::W),
            Instr::COpImm(21, 21, -22, BinaryOp::Add),
            Instr::COpImm(21, GP_ZERO, -22, BinaryOp::Add),
            Instr::COpImm(GP_SP, GP_SP, -352, BinaryOp::Add),
            Instr::COpImm(21, GP_ZERO, -90112, BinaryOp::Add),
            Instr::COpImm(10, 10, 21, BinaryOp::Srl),
            Instr::COpImm(10, 10, 21, BinaryOp::Sra),
            Instr::COpImm(10, 10, 21, BinaryOp::And),
            Instr::COp(10, 13, BinaryOp::Sub),
            Instr::COp(10, 13, BinaryOp::Xor),
            Instr::COp(10, 13, BinaryOp::Or),
            Instr::COp(10, 13, BinaryOp::And),
            Instr::CJal(GP_ZERO, -1366),
            Instr::CBranch(10, -172, CmpCond::Eq),
            Instr::CBranch(10, -172, CmpCond::Ne),
            Instr::COpImm(21, 21, 21, BinaryOp::Sll),
            Instr::CLoad(21, GP_SP, 240, MemWidth::W),
            Instr::CJalr(GP_ZERO, 21),
            Instr::COpImm(21, 27, 0, BinaryOp::Add),
            Instr::Trap(Exception::Ebreak),
            Instr::CJalr(GP_RA, 21),
            Instr::COp(21, 27, BinaryOp::Add),
            Instr::CStore(GP_SP, 21, 240, MemWidth::W),
        ];
        assert!(all_pass(&RV32::default(), &ins_raw, &ins_expect));

        #[cfg(feature = "D")]
        {
            let ins_raw = [0x3aa8u16, 0xbaa8u16, 0x3aceu16, 0xb9d6u16];
            let ins_expect = [
                Instr::CLoadFp(10, 13, 112, Precision::D),
                Instr::CStoreFp(13, 10, 112, Precision::D),
                Instr::CLoadFp(21, GP_SP, 240, Precision::D),
                Instr::CStoreFp(GP_SP, 21, 240, Precision::D),
            ];
            assert!(all_pass(&RV32::default(), &ins_raw, &ins_expect));
        }

        // rv32 != rv64
        let ins_raw = [
            0x3801u16, 0x7ea8u16, 0xfea8u16, 0x7aceu16, 0xf9d6u16, 0x9115u16, 0x9515u16, 0x9d15u16,
            0x9d35u16, 0x1516u16,
        ];
        let ins_expect = [Instr::CJal(1, -2032)];
        assert!(all_pass(&RV32::default(), &ins_raw[..1], &ins_expect));

        #[cfg(feature = "RV64")]
        {
            let ins_expect = [
                Instr::COpImm(16, 16, -32, BinaryOp::AddW),
                Instr::CLoad(10, 13, 120, MemWidth::D),
                Instr::CStore(13, 10, 120, MemWidth::D),
                Instr::CLoad(21, GP_SP, 240, MemWidth::D),
                Instr::CStore(GP_SP, 21, 240, MemWidth::D),
                Instr::COpImm(10, 10, 37, BinaryOp::Srl),
                Instr::COpImm(10, 10, 37, BinaryOp::Sra),
                Instr::COp(10, 13, BinaryOp::SubW),
                Instr::COp(10, 13, BinaryOp::AddW),
                Instr::COpImm(10, 10, 37, BinaryOp::Sll),
            ];
            assert!(all_pass(&RV64::default(), &ins_raw, &ins_expect));
        }

        #[cfg(feature = "F")]
        {
            let ins_expect32 = [
                Instr::CLoadFp(10, 13, 120, Precision::S),
                Instr::CStoreFp(13, 10, 120, Precision::S),
                Instr::CLoadFp(21, GP_SP, 240, Precision::S),
                Instr::CStoreFp(GP_SP, 21, 240, Precision::S),
            ];
            assert!(all_pass(&RV32::default(), &ins_raw[1..6], &ins_expect32));
        }
    }
}
