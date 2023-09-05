impl UnaryOps {
    pub fn exec<Xlen: XlenT>(self, lhs: Xlen, rhs: Xlen) -> Xlen {
        match self {
            UnaryOps::Add => lhs.add(rhs),
            UnaryOps::Sub => lhs.sub(rhs),
            UnaryOps::Slt => {
                if lhs.scmp(rhs) == Ordering::Less {
                    Xlen::from(1)
                } else {
                    Xlen::from(0)
                }
            }
            UnaryOps::SltU => {
                if lhs < rhs {
                    Xlen::from(1)
                } else {
                    Xlen::from(0)
                }
            }
            UnaryOps::And => lhs & rhs,
            UnaryOps::Or => lhs | rhs,
            UnaryOps::Xor => lhs ^ rhs,
            UnaryOps::Sll => {
                let shamt: u32 = rhs.into();
                lhs << (shamt % Xlen::xlen())
            }
            UnaryOps::Srl => {
                let shamt: u32 = rhs.into();
                lhs >> (shamt % Xlen::xlen())
            }
            UnaryOps::Sra => {
                let shamt: u32 = rhs.into();
                lhs.sra(shamt % Xlen::xlen())
            }
            UnaryOps::AddW => lhs.add(rhs).sext32(),
            UnaryOps::SubW => lhs.sub(rhs).sext32(),
            UnaryOps::SllW => {
                let shamt: u32 = rhs.into();
                (lhs << (shamt % 32)).sext32()
            }
            UnaryOps::SrlW => {
                let shamt: u32 = rhs.into();
                (lhs.trunc32() >> (shamt % 32)).sext32()
            }
            UnaryOps::SraW => {
                let lhs: i32 = lhs.into();
                let shamt: u32 = rhs.into();
                Xlen::from(lhs >> (shamt % 32)).sext32()
            }
            // UnaryOps::AddD => lhs.add(rhs).sext64(),
            // UnaryOps::SubD => lhs.sub(rhs).sext64(),
            // UnaryOps::SllD => {
            //     let shamt: u32 = rhs.into();
            //     (lhs << (shamt % 64)).sext64()
            // }
            // UnaryOps::SrlD => {
            //     let shamt: u32 = rhs.into();
            //     (lhs.trunc64() >> (shamt % 64)).sext64()
            // }
            // UnaryOps::SraD => {
            //     let lhs: i64 = lhs.into();
            //     let shamt: u32 = rhs.into();
            //     Xlen::from(lhs >> (shamt % 64)).sext64()
            // }
            UnaryOps::Mul => lhs.mul(rhs),
            UnaryOps::Mulh => lhs.mulh(rhs),
            UnaryOps::MulhU => lhs.mulhu(rhs),
            UnaryOps::MulhSU => lhs.mulhsu(rhs),
            UnaryOps::Div => lhs.div(rhs),
            UnaryOps::DivU => lhs.divu(rhs),
            UnaryOps::Rem => lhs.rem(rhs),
            UnaryOps::RemU => lhs.remu(rhs),
            UnaryOps::MulW => {
                let lhs: u32 = lhs.into();
                let rhs: u32 = rhs.into();
                Xlen::from(lhs.wrapping_mul(rhs)).sext32()
            }
            UnaryOps::DivW => {
                let lhs: u32 = lhs.into();
                let rhs: u32 = rhs.into();
                Xlen::from(<u32 as XlenT>::div(lhs, rhs)).sext32()
            }
            UnaryOps::DivUW => {
                let lhs: u32 = lhs.into();
                let rhs: u32 = rhs.into();
                Xlen::from(<u32 as XlenT>::divu(lhs, rhs)).sext32()
            }
            UnaryOps::RemW => {
                let lhs: u32 = lhs.into();
                let rhs: u32 = rhs.into();
                Xlen::from(<u32 as XlenT>::rem(lhs, rhs)).sext32()
            }
            UnaryOps::RemUW => {
                let lhs: u32 = lhs.into();
                let rhs: u32 = rhs.into();
                Xlen::from(<u32 as XlenT>::remu(lhs, rhs)).sext32()
            }
            UnaryOps::Second => rhs,
            UnaryOps::Max => {
                if lhs.scmp(rhs) == Ordering::Greater {
                    lhs
                } else {
                    rhs
                }
            }
            UnaryOps::MaxU => lhs.max(rhs),
            UnaryOps::Min => {
                if lhs.scmp(rhs) == Ordering::Less {
                    lhs
                } else {
                    rhs
                }
            }
            UnaryOps::MinU => lhs.min(rhs),
            UnaryOps::MaxW => {
                let lhs: i32 = lhs.into();
                let rhs: i32 = rhs.into();
                Xlen::from(lhs.max(rhs)).sext32()
            }
            UnaryOps::MaxUW => {
                let lhs: u32 = lhs.into();
                let rhs: u32 = rhs.into();
                Xlen::from(lhs.max(rhs)).sext32()
            }
            UnaryOps::MinW => {
                let lhs: i32 = lhs.into();
                let rhs: i32 = rhs.into();
                Xlen::from(lhs.min(rhs)).sext32()
            }
            UnaryOps::MinUW => {
                let lhs: u32 = lhs.into();
                let rhs: u32 = rhs.into();
                Xlen::from(lhs.min(rhs)).sext32()
            }
            _ => todo!(),
        }
    }
}

impl CmpCond {
    pub fn test<Xlen: XlenT>(self, lhs: Xlen, rhs: Xlen) -> bool {
        match self {
            CmpCond::Eq => lhs == rhs,
            CmpCond::Ne => lhs != rhs,
            CmpCond::Lt => lhs.scmp(rhs) == Ordering::Less,
            CmpCond::LtU => lhs < rhs,
            CmpCond::Ge => lhs.scmp(rhs) != Ordering::Less,
            CmpCond::GeU => lhs >= rhs,
        }
    }
}

impl Instr {
    pub fn exec<Xlen: XlenT>(&mut self, hart: &mut Hart<Xlen>) {
        match *self {
            Instr::RdRs1Rs2(rd, rs1, rs2, op) => {
                let lhs = hart.get_gp(rs1);
                let rhs = hart.get_gp(rs2);
                let res = op.exec(lhs, rhs);
                hart.set_gp(rd, res);
                hart.add_pc(4);
            }
            Instr::RdRs1Imm(rd, rs1, imm, op) => {
                let lhs = hart.get_gp(rs1);
                let rhs = Xlen::from(imm);
                let res = op.exec(lhs, rhs);
                hart.set_gp(rd, res);
                hart.add_pc(4);
            }
            Instr::BranchCond(rs1, rs2, offset, cond) => {
                let lhs = hart.get_gp(rs1);
                let rhs = hart.get_gp(rs2);
                if cond.test(lhs, rhs) {
                    hart.add_pc(offset);
                } else {
                    hart.add_pc(4);
                }
            }
            Instr::Jal(rd, offset) => {
                hart.set_gp(rd, hart.pc.add(4));
                hart.add_pc(offset);
            }
            Instr::Jalr(rd, rs1, offset) => {
                let addr = hart.get_gp(rs1).add(offset);
                hart.set_gp(rd, hart.pc.add(4));
                hart.pc = addr;
            }
            Instr::Lui(rd, imm) => {
                hart.set_gp(rd, Xlen::from(imm));
                hart.add_pc(4);
            }
            Instr::Auipc(rd, imm) => {
                let addr = hart.pc.add(imm);
                hart.set_gp(rd, addr);
                hart.add_pc(4);
            }
            _ => todo!(),
        }
    }
}
