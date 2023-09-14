use crate::{
    micro_op::{CmpCond, BinaryOp},
    xlen::XlenT,
};
use std::cmp::Ordering;

impl CmpCond {
    fn test<Xlen: XlenT>(self, lhs: Xlen, rhs: Xlen) -> bool {
        match self {
            CmpCond::Eq => lhs == rhs,
            CmpCond::Ne => lhs != rhs,
            CmpCond::Lt => lhs.scmp(rhs) == Ordering::Less,
            CmpCond::Ge => lhs.scmp(rhs) != Ordering::Less,
            CmpCond::LtU => lhs < rhs,
            CmpCond::GeU => lhs >= rhs,
        }
    }
}

impl BinaryOp {
    fn exec<Xlen: XlenT>(self, lhs: Xlen, rhs: Xlen) -> Xlen {
        match self {
            BinaryOp::Add => lhs.add(rhs),
            BinaryOp::Sll => {
                let shamt: u32 = rhs.into();
                lhs << (shamt % Xlen::xlen())
            }
            BinaryOp::Slt => {
                if lhs.scmp(rhs) == Ordering::Less {
                    Xlen::from(1)
                } else {
                    Xlen::from(0)
                }
            }
            BinaryOp::SltU => {
                if lhs < rhs {
                    Xlen::from(1)
                } else {
                    Xlen::from(0)
                }
            }
            BinaryOp::Xor => lhs ^ rhs,
            BinaryOp::Srl => {
                let shamt: u32 = rhs.into();
                lhs >> (shamt % Xlen::xlen())
            }
            BinaryOp::Or => lhs | rhs,
            BinaryOp::And => lhs & rhs,
            BinaryOp::Sub => lhs.sub(rhs),
            BinaryOp::Sra => {
                let shamt: u32 = rhs.into();
                lhs.sra(shamt % Xlen::xlen())
            }
            #[cfg(feature = "RV64")]
            BinaryOp::AddW => lhs.add(rhs).sext32(),
            #[cfg(feature = "RV64")]
            BinaryOp::SllW => {
                let shamt: u32 = rhs.into();
                (lhs << (shamt % 32)).sext32()
            }
            #[cfg(feature = "RV64")]
            BinaryOp::SrlW => {
                let shamt: u32 = rhs.into();
                (lhs.trunc32() >> (shamt % 32)).sext32()
            }
            #[cfg(feature = "RV64")]
            BinaryOp::SubW => lhs.sub(rhs).sext32(),
            #[cfg(feature = "RV64")]
            BinaryOp::SraW => {
                let lhs: i32 = lhs.into();
                let shamt: u32 = rhs.into();
                Xlen::from(lhs >> (shamt % 32))
            }
            #[cfg(feature = "M")]
            BinaryOp::Mul => lhs.mul(rhs),
            #[cfg(feature = "M")]
            BinaryOp::Mulh => lhs.mulh(rhs),
            #[cfg(feature = "M")]
            BinaryOp::MulhU => lhs.mulhu(rhs),
            #[cfg(feature = "M")]
            BinaryOp::MulhSU => lhs.mulhsu(rhs),
            #[cfg(feature = "M")]
            BinaryOp::Div => lhs.div(rhs),
            #[cfg(feature = "M")]
            BinaryOp::DivU => lhs.divu(rhs),
            #[cfg(feature = "M")]
            BinaryOp::Rem => lhs.rem(rhs),
            #[cfg(feature = "M")]
            BinaryOp::RemU => lhs.remu(rhs),
            #[cfg(all(feature = "M", feature = "RV64"))]
            BinaryOp::MulW => {
                let lhs: u32 = lhs.into();
                let rhs: u32 = rhs.into();
                Xlen::from(lhs.wrapping_mul(rhs)).sext32()
            }
            #[cfg(all(feature = "M", feature = "RV64"))]
            BinaryOp::DivW => {
                let lhs: u32 = lhs.into();
                let rhs: u32 = rhs.into();
                Xlen::from(<u32 as XlenT>::div(lhs, rhs)).sext32()
            }
            #[cfg(all(feature = "M", feature = "RV64"))]
            BinaryOp::DivUW => {
                let lhs: u32 = lhs.into();
                let rhs: u32 = rhs.into();
                Xlen::from(<u32 as XlenT>::divu(lhs, rhs)).sext32()
            }
            #[cfg(all(feature = "M", feature = "RV64"))]
            BinaryOp::RemW => {
                let lhs: u32 = lhs.into();
                let rhs: u32 = rhs.into();
                Xlen::from(<u32 as XlenT>::rem(lhs, rhs)).sext32()
            }
            #[cfg(all(feature = "M", feature = "RV64"))]
            BinaryOp::RemUW => {
                let lhs: u32 = lhs.into();
                let rhs: u32 = rhs.into();
                Xlen::from(<u32 as XlenT>::remu(lhs, rhs)).sext32()
            }
            #[cfg(feature = "A")]
            BinaryOp::Second => rhs,
            #[cfg(feature = "A")]
            BinaryOp::Max => {
                if lhs.scmp(rhs) == Ordering::Greater {
                    lhs
                } else {
                    rhs
                }
            }
            #[cfg(feature = "A")]
            BinaryOp::MaxU => lhs.max(rhs),
            #[cfg(feature = "A")]
            BinaryOp::Min => {
                if lhs.scmp(rhs) == Ordering::Less {
                    lhs
                } else {
                    rhs
                }
            }
            #[cfg(feature = "A")]
            BinaryOp::MinU => lhs.min(rhs),
            // #[cfg(all(feature = "A", feature = "RV64"))]
            // BinaryOp::MaxW => {
            //     let lhs: i32 = lhs.into();
            //     let rhs: i32 = rhs.into();
            //     Xlen::from(lhs.max(rhs))
            // }
            // #[cfg(all(feature = "A", feature = "RV64"))]
            // BinaryOp::MaxUW => {
            //     let lhs: u32 = lhs.into();
            //     let rhs: u32 = rhs.into();
            //     Xlen::from(lhs.max(rhs)).sext32()
            // }
            // #[cfg(all(feature = "A", feature = "RV64"))]
            // BinaryOp::MinW => {
            //     let lhs: i32 = lhs.into();
            //     let rhs: i32 = rhs.into();
            //     Xlen::from(lhs.min(rhs))
            // }
            // #[cfg(all(feature = "A", feature = "RV64"))]
            // BinaryOp::MinUW => {
            //     let lhs: u32 = lhs.into();
            //     let rhs: u32 = rhs.into();
            //     Xlen::from(lhs.min(rhs)).sext32()
            // }
        }
    }
}
