use crate::{hart::Hart, uop::*, utils::Maybe, xlen::XlenT};
use std::{
    cmp::PartialOrd,
    mem::size_of,
    num::FpCategory,
    ops::{Add, BitAnd, BitOr, Div, Mul, Neg, Not, Sub},
};

mod arch;
#[cfg(test)]
mod tests;

/// floating-point register size
const FPLEN: usize = {
    #[cfg(feature = "Q")]
    {
        16
    }
    #[cfg(feature = "D")]
    {
        8
    }
    #[cfg(not(any(feature = "Q", feature = "D")))]
    {
        4
    }
};

#[repr(align(16))]
#[derive(Debug, Clone)]
pub struct Fpu {
    /// floating-point register files
    /// represent as little endian bytes
    /// since its easier to support narrower value Nan boxing
    fprs: [[u8; FPLEN]; 32],
    /// system fpu exceptions
    fpe: FpExcept,
    /// dynamic rounding mode
    dyn_rm: RoundMode,
    /// system fpu rounding mode
    rm: RoundMode,
}

impl Default for Fpu {
    fn default() -> Self {
        Self {
            fprs: Default::default(),
            fpe: Default::default(),
            dyn_rm: RoundMode::Rne,
            rm: RoundMode::None,
        }
    }
}

#[repr(align(8))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FpExcept {
    /// invalid operation exception flag
    nv: bool,
    /// divide by zero exception flag
    dz: bool,
    /// overflow exception flag
    of: bool,
    /// underflow exception flag
    uf: bool,
    /// inexact exception flag
    nx: bool,
}

impl FpExcept {
    pub fn as_u8(self) -> u8 {
        let mut e = 0u8;
        if self.nx {
            e |= 1;
        }
        if self.uf {
            e |= 2;
        }
        if self.of {
            e |= 4;
        }
        if self.dz {
            e |= 8;
        }
        if self.nv {
            e |= 16;
        }
        e
    }
}

const FPE: FpExcept = FpExcept {
    nv: false,
    dz: false,
    of: false,
    uf: false,
    nx: false,
};
const FPE_NV: FpExcept = FpExcept { nv: true, ..FPE };
const FPE_DZ: FpExcept = FpExcept { dz: true, ..FPE };
const FPE_OF: FpExcept = FpExcept { of: true, ..FPE };
const FPE_UF: FpExcept = FpExcept { uf: true, ..FPE };
const FPE_NX: FpExcept = FpExcept { nx: true, ..FPE };

impl BitOr for FpExcept {
    type Output = FpExcept;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            nv: self.nv | rhs.nv,
            dz: self.dz | rhs.dz,
            of: self.of | rhs.of,
            uf: self.uf | rhs.uf,
            nx: self.nx | rhs.nx,
        }
    }
}

impl Not for FpExcept {
    type Output = FpExcept;
    fn not(self) -> Self::Output {
        Self {
            nv: !self.nv,
            dz: !self.dz,
            of: !self.of,
            uf: !self.uf,
            nx: !self.nx,
        }
    }
}

impl BitAnd for FpExcept {
    type Output = FpExcept;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            nv: self.nv & rhs.nv,
            dz: self.dz & rhs.dz,
            of: self.of & rhs.of,
            uf: self.uf & rhs.uf,
            nx: self.nx & rhs.nx,
        }
    }
}

/// read from fpu.fprs as floating-point type,
/// does narrower value Nan boxing automatically
/// (fpr, type)
macro_rules! read_fpr_as {
    ($fpr:ident, $t:ty) => {{
        const SZ: usize = size_of::<$t>();
        let mut buf = [0u8; SZ];
        buf.copy_from_slice(&$fpr[..SZ]);
        if $fpr[SZ..].iter().any(|&v| v != 0xff) {
            Self::canonical_nan()
        } else {
            Self::from_le_bytes(buf)
        }
    }};
}

/// write to fpu.fprs from floating-point type,
/// does narrower value Nan boxing automatically
/// (fpr, type, val)
macro_rules! write_fpr_as {
    ($fpr:ident, $t:ty, $val:ident) => {
        const SZ: usize = size_of::<$t>();
        $fpr[..SZ].copy_from_slice(&$val.to_le_bytes());
        $fpr[SZ..].fill(0xff);
    };
}

/// dispatch macro for different precision
/// (pr, func, args, ...)
macro_rules! pr_switch {
    ($pr:ident, $func:ident, $($e:expr), *) => {
        match $pr {
            Precision::S => f32::$func($($e), *),
            #[cfg(feature = "D")]
            Precision::D => f64::$func($($e), *),
        }
    };
}

pub trait FpOp:
    Sized
    + Copy
    + Neg<Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + PartialOrd
{
    fn rd_fpr(fpu: &Fpu, reg: u8) -> Self;
    fn wr_fpr(fpu: &mut Fpu, reg: u8, val: Self);
    /// return canonical Nan defined in risc-v standard
    fn canonical_nan() -> Self;
    fn classify(self) -> FpCategory;
    fn is_nan(self) -> bool;
    /// check if quiet bit defined in risc-v standard is set
    fn is_quiet(self) -> bool;
    fn no_nan_box(self) -> Self {
        if self.is_nan() {
            // TODO: some arch can bypass this
            Self::canonical_nan()
        } else {
            self
        }
    }
    fn ternary_op(fpu: &mut Fpu, rd: u8, rs1: u8, rs2: u8, rs3: u8, op: FpTernaryOp) {
        let rs1 = Self::rd_fpr(fpu, rs1);
        let mut rs2 = Self::rd_fpr(fpu, rs2);
        let mut rs3 = Self::rd_fpr(fpu, rs3);
        // FMADD.S computes (rs1 × rs2) + rs3
        // FMSUB.S computes (rs1 × rs2) - rs3
        // FNMSUB.S computes -(rs1 × rs2) + rs3
        // FNMADD.S computes -(rs1 × rs2) - rs3
        if op == FpTernaryOp::NMSub || op == FpTernaryOp::NMAdd {
            rs2 = -rs2;
        }
        if op == FpTernaryOp::MSub || op == FpTernaryOp::NMAdd {
            rs3 = -rs3;
        }
        let res = fpu
            .pre_fma(rs1, rs2, rs3)
            .unwrap_or_else(|| rs1.mul_add(rs2, rs3))
            .no_nan_box();
        Self::wr_fpr(fpu, rd, res);
    }
    fn binary_op(fpu: &mut Fpu, rd: u8, rs1: u8, rs2: u8, op: FpBinaryOp) {
        let rs1 = Self::rd_fpr(fpu, rs1);
        let rs2 = Self::rd_fpr(fpu, rs2);
        let res = fpu.pre_binary_op(rs1, rs2, op).unwrap_or_else(|| match op {
            FpBinaryOp::Add => rs1 + rs2,
            FpBinaryOp::Sub => rs1 - rs2,
            FpBinaryOp::Mul => rs1 * rs2,
            FpBinaryOp::Div => rs1 / rs2,
            // TODO: check for exception correctness
            // codegen seems fine
            FpBinaryOp::SgnJ => rs1.copysign(rs2),
            FpBinaryOp::SgnJN => rs1.copysign(-rs2),
            FpBinaryOp::SgnJX => {
                if rs2.is_neg() {
                    -rs1
                } else {
                    rs1
                }
            }
            // TODO: check for exception correctness and (+0, -O)
            // codegen seems fine
            FpBinaryOp::Min => rs1.min(rs2),
            FpBinaryOp::Max => rs1.max(rs2),
        });
        let res = match op {
            FpBinaryOp::SgnJ | FpBinaryOp::SgnJN | FpBinaryOp::SgnJX => res,
            _ => res.no_nan_box(),
        };
        Self::wr_fpr(fpu, rd, res);
    }
    fn unary_op(fpu: &mut Fpu, rd: u8, rs1: u8, op: FpUnaryOp) {
        let rs1 = Self::rd_fpr(fpu, rs1);
        let res = fpu
            .pre_unary_op(rs1, op)
            .unwrap_or_else(|| match op {
                // TODO: check for exception correctness and -0.0
                // codegen seems fine
                FpUnaryOp::Sqrt => rs1.sqrt(),
            })
            .no_nan_box();
        Self::wr_fpr(fpu, rd, res);
    }
    fn fp_to_i32(fpu: &Fpu, rs1: u8) -> i32;
    fn fp_to_u32(fpu: &Fpu, rs1: u8) -> u32;
    fn fp_to_i64(fpu: &Fpu, rs1: u8) -> i64;
    fn fp_to_u64(fpu: &Fpu, rs1: u8) -> i32;
    fn i32_to_fp(fpu: &mut Fpu, rd: u8, rs1: i32);
    fn u32_to_fp(fpu: &mut Fpu, rd: u8, rs1: u32);
    fn i64_to_fp(fpu: &mut Fpu, rd: u8, rs1: i64);
    fn u64_to_fp(fpu: &mut Fpu, rd: u8, rs1: u64);
    fn class(fpu: &Fpu, rs1: u8) -> u32 {
        let rs1 = Self::rd_fpr(fpu, rs1);
        // TODO: check exception correctness
        // codegen seems fine
        match (rs1.is_neg(), rs1.classify(), rs1.is_quiet()) {
            (true, FpCategory::Infinite, _) => 1 << 0,
            (true, FpCategory::Normal, _) => 1 << 1,
            (true, FpCategory::Subnormal, _) => 1 << 2,
            (true, FpCategory::Zero, _) => 1 << 3,
            (false, FpCategory::Zero, _) => 1 << 4,
            (false, FpCategory::Subnormal, _) => 1 << 5,
            (false, FpCategory::Normal, _) => 1 << 6,
            (false, FpCategory::Infinite, _) => 1 << 7,
            (_, FpCategory::Nan, false) => 1 << 8,
            (_, FpCategory::Nan, true) => 1 << 9,
        }
    }
    fn cmp(fpu: &Fpu, rs1: u8, rs2: u8, op: FpCmpCond) -> bool {
        let rs1 = Self::rd_fpr(fpu, rs1);
        let rs2 = Self::rd_fpr(fpu, rs2);
        // TODO: check exception correctness
        // likely to be broken on lt, le
        match op {
            FpCmpCond::Eq => rs1 == rs2,
            FpCmpCond::Lt => rs1 < rs2,
            FpCmpCond::Le => rs1 <= rs2,
        }
    }
    // fp use std
    fn mul_add(self, a: Self, b: Self) -> Self;
    fn sqrt(self) -> Self;
    fn copysign(self, rhs: Self) -> Self;
    fn max(self, rhs: Self) -> Self;
    fn min(self, rhs: Self) -> Self;
    fn is_neg(self) -> bool;
}

macro_rules! fp_use_std {
    () => {
        fn mul_add(self, a: Self, b: Self) -> Self {
            self.mul_add(a, b)
        }
        fn sqrt(self) -> Self {
            self.sqrt()
        }
        fn copysign(self, rhs: Self) -> Self {
            self.copysign(rhs)
        }
        fn max(self, rhs: Self) -> Self {
            self.max(rhs)
        }
        fn min(self, rhs: Self) -> Self {
            self.min(rhs)
        }
        fn is_neg(self) -> bool {
            self.is_sign_negative()
        }
    };
}

impl FpOp for f32 {
    fn canonical_nan() -> Self {
        Self::from_bits(0x7fc00000)
    }

    fn rd_fpr(fpu: &Fpu, reg: u8) -> Self {
        let fpr = fpu.fprs[reg as usize];
        read_fpr_as!(fpr, f32)
    }

    fn wr_fpr(fpu: &mut Fpu, reg: u8, val: Self) {
        let fpr = &mut fpu.fprs[reg as usize];
        write_fpr_as!(fpr, f32, val);
    }

    fn classify(self) -> FpCategory {
        // TODO: fix classify exception problem
        self.classify()
    }
    fn is_nan(self) -> bool {
        // TODO: fix nan exception problem
        self.is_nan()
    }

    fn is_quiet(self) -> bool {
        self.to_bits() & 0x400000 != 0
    }

    fp_use_std!();

    fn fp_to_i32(fpu: &Fpu, rs1: u8) -> i32 {
        todo!()
    }

    fn fp_to_u32(fpu: &Fpu, rs1: u8) -> u32 {
        todo!()
    }

    fn fp_to_i64(fpu: &Fpu, rs1: u8) -> i64 {
        todo!()
    }

    fn fp_to_u64(fpu: &Fpu, rs1: u8) -> i32 {
        todo!()
    }

    fn i32_to_fp(fpu: &mut Fpu, rd: u8, rs1: i32) {
        todo!()
    }

    fn u32_to_fp(fpu: &mut Fpu, rd: u8, rs1: u32) {
        todo!()
    }

    fn i64_to_fp(fpu: &mut Fpu, rd: u8, rs1: i64) {
        todo!()
    }

    fn u64_to_fp(fpu: &mut Fpu, rd: u8, rs1: u64) {
        todo!()
    }
}

#[cfg(feature = "D")]
impl FpOp for f64 {
    fn canonical_nan() -> Self {
        f64::from_bits(0x7ff8000000000000)
    }

    fn rd_fpr(fpu: &Fpu, reg: u8) -> Self {
        let fpr = fpu.fprs[reg as usize];
        read_fpr_as!(fpr, f64)
    }

    fn wr_fpr(fpu: &mut Fpu, reg: u8, val: Self) {
        let fpr = &mut fpu.fprs[reg as usize];
        write_fpr_as!(fpr, f64, val);
    }

    fn classify(self) -> FpCategory {
        // TODO: fix classify problem
        self.classify()
    }
    fn is_nan(self) -> bool {
        // TODO: fix nan problem
        self.is_nan()
    }
    fn is_quiet(self) -> bool {
        self.to_bits() & 0x8000000000000 != 0
    }

    fp_use_std!();

    fn fp_to_i32(fpu: &Fpu, rs1: u8) -> i32 {
        todo!()
    }

    fn fp_to_u32(fpu: &Fpu, rs1: u8) -> u32 {
        todo!()
    }

    fn fp_to_i64(fpu: &Fpu, rs1: u8) -> i64 {
        todo!()
    }

    fn fp_to_u64(fpu: &Fpu, rs1: u8) -> i32 {
        todo!()
    }

    fn i32_to_fp(fpu: &mut Fpu, rd: u8, rs1: i32) {
        todo!()
    }

    fn u32_to_fp(fpu: &mut Fpu, rd: u8, rs1: u32) {
        todo!()
    }

    fn i64_to_fp(fpu: &mut Fpu, rd: u8, rs1: i64) {
        todo!()
    }

    fn u64_to_fp(fpu: &mut Fpu, rd: u8, rs1: u64) {
        todo!()
    }
}

impl Fpu {
    pub fn ternary_op(
        &mut self,
        rd: u8,
        rs1: u8,
        rs2: u8,
        rs3: u8,
        pr: Precision,
        op: FpTernaryOp,
    ) {
        pr_switch!(pr, ternary_op, self, rd, rs1, rs2, rs3, op);
    }

    pub fn binary_op(&mut self, rd: u8, rs1: u8, rs2: u8, pr: Precision, op: FpBinaryOp) {
        pr_switch!(pr, binary_op, self, rd, rs1, rs2, op);
    }

    pub fn unary_op(&mut self, rd: u8, rs1: u8, pr: Precision, op: FpUnaryOp) {
        pr_switch!(pr, unary_op, self, rd, rs1, op);
    }

    pub fn fp_cvt_gp<Xlen: XlenT>(&self, rs1: u8, pr: Precision, op: FpGpOp) -> Xlen {
        match op {
            FpGpOp::W => Xlen::from(pr_switch!(pr, fp_to_i32, self, rs1)),
            FpGpOp::WU => Xlen::from(pr_switch!(pr, fp_to_u32, self, rs1)),
            #[cfg(feature = "RV64")]
            FpGpOp::L => Xlen::from(pr_switch!(pr, fp_to_i64, self, rs1)),
            #[cfg(feature = "RV64")]
            FpGpOp::LU => Xlen::from(pr_switch!(pr, fp_to_u64, self, rs1)),
            FpGpOp::MV => match pr {
                Precision::S => Xlen::from(self.f32_mv_u32(rs1)),
                #[cfg(feature = "D")]
                Precision::D => Xlen::from(self.f64_mv_u64(rs1)),
            },
            FpGpOp::Class => Xlen::from(pr_switch!(pr, class, self, rs1)),
        }
    }

    pub fn gp_cvt_fp<Xlen: XlenT>(&mut self, rd: u8, rs1: Xlen, pr: Precision, op: GpFpOp) {
        match op {
            GpFpOp::W => pr_switch!(pr, i32_to_fp, self, rd, rs1.into()),
            GpFpOp::WU => pr_switch!(pr, u32_to_fp, self, rd, rs1.into()),
            #[cfg(feature = "RV64")]
            GpFpOp::L => pr_switch!(pr, i64_to_fp, self, rd, rs1.into()),
            #[cfg(feature = "RV64")]
            GpFpOp::LU => pr_switch!(pr, u64_to_fp, self, rd, rs1.into()),
            GpFpOp::MV => match pr {
                Precision::S => self.u32_mv_f32(rd, rs1.into()),
                #[cfg(feature = "RV64")]
                Precision::D => self.u64_mv_f64(rd, rs1.into()),
            },
        }
    }

    pub fn u32_mv_f32(&mut self, rd: u8, val: u32) {
        let val = f32::from_bits(val);
        f32::wr_fpr(self, rd, val);
    }

    pub fn f32_mv_u32(&self, rs1: u8) -> u32 {
        let val = f32::rd_fpr(self, rs1);
        val.to_bits()
    }

    #[cfg(feature = "D")]
    pub fn u64_mv_f64(&mut self, rd: u8, val: u64) {
        let val = f64::from_bits(val);
        f64::wr_fpr(self, rd, val);
    }

    #[cfg(feature = "D")]
    pub fn f64_mv_u64(&self, rs1: u8) -> u64 {
        let val = f64::rd_fpr(self, rs1);
        val.to_bits()
    }

    pub fn fp_cmp(&self, rs1: u8, rs2: u8, pr: Precision, op: FpCmpCond) -> u32 {
        pr_switch!(pr, cmp, self, rs1, rs2, op) as u32
    }

    #[cfg(feature = "D")]
    pub fn fp_cvt_fp(&mut self, rd: u8, rs1: u8, from: Precision, to: Precision) {
        match (from, to) {
            (Precision::S, Precision::D) => {
                let val = f32::rd_fpr(self, rs1);
                // TODO: check for error
                f64::wr_fpr(self, rd, val as f64);
            }
            (Precision::D, Precision::S) => {
                let val = f64::rd_fpr(self, rs1);
                // TODO: check for error
                f32::wr_fpr(self, rd, val as f32);
            }
            _ => panic!("bad uop"),
        }
    }

    // set round mode for dynamic rounding
    pub fn set_dyn_rm(&mut self, rm: RoundMode) {
        self.dyn_rm = rm;
    }

    pub fn get_fpe(&mut self) -> FpExcept {
        self.sync_fpe();
        self.fpe
    }

    pub fn set_fpe(&mut self, mask: FpExcept) {
        self.fpe = self.fpe | mask;
    }

    pub fn clr_fpe(&mut self, mask: FpExcept) {
        self.sync_fpe();
        arch::clr_fpe();
        self.fpe = self.fpe & !mask;
    }

    pub fn clr_all_fpe(&mut self) {
        arch::clr_fpe();
        self.fpe = Default::default();
    }

    fn sync_fpe(&mut self) {
        self.fpe = self.fpe | arch::get_fpe();
    }

    fn set_rm(&mut self, rm: RoundMode) {
        self.rm = rm;
        arch::set_rm(rm);
    }
}

impl<Xlen: XlenT> Hart<Xlen> {
    /// setup round mode before fp-op
    pub fn set_rt_rm(&mut self, rm: RoundMode) -> Maybe<()> {
        match rm {
            RoundMode::Dyn => {
                // If frm is set to an invalid value
                // subsequent attempt to execute a floating-point
                // operation with a dynamic rounding mode will
                // raise an illegal instruction exception
                if self.fpu.dyn_rm == RoundMode::None {
                    return self.raise(Exception::IllegalInstr);
                } else if self.fpu.dyn_rm != self.fpu.rm {
                    self.fpu.set_rm(self.fpu.dyn_rm);
                }
            }
            // op with no round mode
            RoundMode::None => (),
            // set up round mode for emulator
            _ => self.fpu.set_rm(rm),
        }
        Ok(())
    }
}
