use crate::{hart::Hart, uop::*, utils::Maybe, xlen::XlenT};
use std::mem::size_of;

#[cfg(feature = "D")]
const FPLEN: usize = 8;
#[cfg(not(feature = "D"))]
const FPLEN: usize = 4;

#[repr(align(8))]
#[derive(Debug, Clone)]
pub struct Fpu {
    fprs: [[u8; FPLEN]; 32],
}

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

macro_rules! write_fpr_as {
    ($fpr:ident, $t:ty, $val:expr) => {
        const SZ: usize = size_of::<$t>();
        $fpr[..SZ].copy_from_slice(&$val);
        $fpr[SZ..].fill(0xff);
    };
}

macro_rules! pr_switch {
    ($pr:ident, $func:ident, $($e:expr), *) => {
        match $pr {
            Precision::S => f32::$func($($e), *),
            #[cfg(feature = "D")]
            Precision::D => f64::$func($($e), *),
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FpClass {
    NegInf,
    NegNorm,
    NegSubNorm,
    NegZero,
    Zero,
    SubNorm,
    Norm,
    Inf,
    SNan,
    QNan,
}

trait FpOp:
    Sized
    + std::ops::Neg<Output = Self>
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + Copy
{
    fn canonical_nan() -> Self;
    fn rd_fpr(fpu: &Fpu, reg: u8) -> Self;
    fn wr_fpr(fpu: &mut Fpu, reg: u8, val: Self);
    fn mul_add(self, a: Self, b: Self) -> Self;
    fn sqrt(self) -> Self;
    // fn to_
    fn classify(self) -> FpClass;
    fn is_nan(self) -> bool {
        let class = self.classify();
        matches!(class, FpClass::SNan | FpClass::QNan)
    }
    fn ternary_op(fpu: &mut Fpu, rd: u8, rs1: u8, rs2: u8, rs3: u8, op: FpTernaryOp) {
        let rs1 = Self::rd_fpr(fpu, rs1);
        let mut rs2 = Self::rd_fpr(fpu, rs2);
        let mut rs3 = Self::rd_fpr(fpu, rs3);
        if op == FpTernaryOp::NMSub || op == FpTernaryOp::NMAdd {
            rs2 = -rs2;
        }
        if op == FpTernaryOp::MSub || op == FpTernaryOp::NMAdd {
            rs3 = -rs3;
        }
        let res = rs1.mul_add(rs2, rs3);
        if res.is_nan() {
            // fix boxing nan
            // fix fma inexact should be raise even when 0 * inf + qnan
            todo!()
        }
        Self::wr_fpr(fpu, rd, res);
    }
    fn binary_op(fpu: &mut Fpu, rd: u8, rs1: u8, rs2: u8, op: FpBinaryOp) {
        let rs1 = Self::rd_fpr(fpu, rs1);
        let rs2 = Self::rd_fpr(fpu, rs2);
        let res = match op {
            FpBinaryOp::Add => rs1 + rs2,
            FpBinaryOp::Sub => rs1 - rs2,
            FpBinaryOp::Mul => rs1 * rs2,
            FpBinaryOp::Div => rs1 / rs2,
            _ => todo!(),
        };
        if res.is_nan() {
            // fix boxing nan
            todo!()
        }
        Self::wr_fpr(fpu, rd, res);
    }
    fn unary_op(fpu: &mut Fpu, rd: u8, rs1: u8, op: FpUnaryOp) {
        let rs1 = Self::rd_fpr(fpu, rs1);
        let res = match op {
            FpUnaryOp::Sqrt => rs1.sqrt(),
        };
        if res.is_nan() {
            // fix boxing nan
            todo!()
        }
        Self::wr_fpr(fpu, rd, res);
    }
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
        write_fpr_as!(fpr, f32, val.to_le_bytes());
    }
    fn mul_add(self, a: Self, b: Self) -> Self {
        self.mul_add(a, b)
    }
    fn classify(self) -> FpClass {
        todo!()
    }
    fn sqrt(self) -> Self {
        let n = 0.0f32 as i32;
        self.sqrt()
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
        write_fpr_as!(fpr, f64, val.to_le_bytes());
    }
    fn mul_add(self, a: Self, b: Self) -> Self {
        self.mul_add(a, b)
    }
    fn classify(self) -> FpClass {
        todo!()
    }
    fn sqrt(self) -> Self {
        self.sqrt()
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
        todo!()
    }
    pub fn gp_cvt_fp<Xlen: XlenT>(&mut self, rd: u8, rs1: Xlen, pr: Precision, op: GpFpOp) {
        todo!()
    }
    pub fn s_from_u32(&mut self, rd: u8, val: u32) {
        todo!()
    }
    pub fn s_to_u32(&self, rs2: u8) -> u32 {
        todo!()
    }
    pub fn d_from_u64(&mut self, rd: u8, val: u64) {
        todo!()
    }
    pub fn d_to_u64(&self, rs2: u8) -> u64 {
        todo!()
    }
    pub fn fp_cmp(&self, rs1: u8, rs2: u8, pr: Precision, op: FpCmpCond) -> u32 {
        todo!()
    }
    pub fn fp_cvt_fp(&mut self, rd: u8, rs1: u8, from: Precision, to: Precision) {
        todo!()
    }
}

impl<Xlen: XlenT> Hart<Xlen> {
    pub fn set_rm(&mut self, rm: RoundMode) -> Maybe<()> {
        todo!()
    }
}
