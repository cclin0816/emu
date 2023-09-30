use crate::{
    fpu::*,
    uop::{FpBinaryOp, FpUnaryOp, RoundMode},
    utils::flag_set,
};
use std::num::FpCategory as FpCat;

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64 as arch;
#[cfg(target_arch = "x86")]
use core::arch::x86 as arch;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64 as arch;

// these are used for hijacking / fixing broken fpu
impl Fpu {
    pub fn pre_fma<T: FpOp>(&mut self, _a: T, _b: T, _c: T) -> Option<T> {
        #[cfg(target_feature = "sse")]
        match (_a.classify(), _b.classify()) {
            // fix 0 * inf + qnan should raise exceptions
            (FpCat::Zero, FpCat::Infinite) | (FpCat::Infinite, FpCat::Zero) => {
                self.set_fpe(FPE_NV);
                return Some(T::canonical_nan());
            }
            _ => (),
        }
        None
    }

    pub fn pre_binary_op<T: FpOp>(&mut self, _a: T, _b: T, _op: FpBinaryOp) -> Option<T> {
        if matches!(_op, FpBinaryOp::Min | FpBinaryOp::Max) {
            #[cfg(target_feature = "sse")]
            {
                // fix (qnan, !snan) (!snan, qnan) should not set exceptions
                match (_a.is_nan_safe(), _b.is_nan_safe()) {
                    (false, false) => (),
                    _ => {}
                }
            }
            // fix (+0, -0) (-0, +0) should return -0
        }
        None
    }

    pub fn pre_unary_op<T: FpOp>(&mut self, _a: T, _op: FpUnaryOp) -> Option<T> {
        None
    }
}

#[cfg(target_feature = "sse")]
pub fn get_fpe() -> FpExcept {
    let stat = unsafe { arch::_MM_GET_EXCEPTION_STATE() };
    FpExcept {
        nv: flag_set(stat, arch::_MM_EXCEPT_INVALID),
        dz: flag_set(stat, arch::_MM_EXCEPT_DIV_ZERO),
        of: flag_set(stat, arch::_MM_EXCEPT_OVERFLOW),
        uf: flag_set(stat, arch::_MM_EXCEPT_UNDERFLOW),
        nx: flag_set(stat, arch::_MM_EXCEPT_INEXACT),
    }
}

#[cfg(target_feature = "sse")]
pub fn clr_fpe() {
    unsafe { arch::_MM_SET_EXCEPTION_STATE(0) }
}

#[cfg(target_feature = "sse")]
pub fn set_rm(rm: RoundMode) {
    let rc = match rm {
        RoundMode::Rne | RoundMode::Rmm => arch::_MM_ROUND_NEAREST,
        RoundMode::Rtz => arch::_MM_ROUND_TOWARD_ZERO,
        RoundMode::Rdn => arch::_MM_ROUND_DOWN,
        RoundMode::Rup => arch::_MM_ROUND_UP,
        _ => panic!("bad round mode"),
    };
    unsafe {
        arch::_MM_SET_ROUNDING_MODE(rc);
    }
}
