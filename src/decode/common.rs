// common decoding functions

use std::{
    cmp::Eq,
    default::Default,
    ops::{BitAnd, Shl, Shr, Sub},
};

use crate::xlen::XlenT;

pub const GP_ZERO: u8 = 0;
pub const GP_RA: u8 = 1;
pub const GP_SP: u8 = 2;

/// enabled isa extension flags
#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct Isa<Xlen: XlenT> {
    /// Atomic
    #[cfg(feature = "A")]
    pub A: bool,
    /// Compressed
    #[cfg(feature = "C")]
    pub C: bool,
    /// Double-precision floating-point
    #[cfg(feature = "D")]
    pub D: bool,
    /// Single-precision floating-point
    #[cfg(feature = "F")]
    pub F: bool,
    /// Integer Multiply/Divide
    #[cfg(feature = "M")]
    pub M: bool,
    /// CSR instructions
    #[cfg(feature = "Zicsr")]
    pub Zicsr: bool,
    /// Instruction-Fetch Fence
    #[cfg(feature = "Zifencei")]
    pub Zifencei: bool,

    xlen: std::marker::PhantomData<Xlen>,
}

impl<Xlen: XlenT> Default for Isa<Xlen> {
    fn default() -> Self {
        Self {
            #[cfg(feature = "A")]
            A: true,
            #[cfg(feature = "C")]
            C: true,
            #[cfg(feature = "D")]
            D: true,
            #[cfg(feature = "F")]
            F: true,
            #[cfg(feature = "M")]
            M: true,
            #[cfg(feature = "Zicsr")]
            Zicsr: true,
            #[cfg(feature = "Zifencei")]
            Zifencei: true,
            xlen: Default::default(),
        }
    }
}

/// sign-extend any length imm to i32
pub fn sext(imm: u32, sign_bit: u32) -> i32 {
    let len = 31 - sign_bit;
    let imm = (imm << len) as i32;
    imm >> len
}

macro_rules! if_ge_rv64 {
    ($e_true:expr, $e_false:expr) => {{
        #[cfg(feature = "RV64")]
        if Xlen::XLEN >= 64 {
            $e_true
        } else {
            $e_false
        }
        #[cfg(not(feature = "RV64"))]
        {
            $e_false
        }
    }};
    ($e:expr) => {
        if_ge_rv64!(Ok($e), Err(()))
    };
}

macro_rules! if_rv128 {
    ($e_true:expr, $e_false:expr) => {{
        #[cfg(feature = "RV128")]
        if Xlen::XLEN == 128 {
            $e_true
        } else {
            $e_false
        }
        #[cfg(not(feature = "RV128"))]
        {
            $e_false
        }
    }};
    ($e:expr) => {
        if_rv128!(Ok($e), Err(()))
    };
}

macro_rules! if_ext {
    ($id:ident, $lit:literal, $isa:expr, $e_true:expr, $e_false:expr) => {{
        #[cfg(feature = $lit)]
        if $isa.$id {
            $e_true
        } else {
            $e_false
        }
        #[cfg(not(feature = $lit))]
        {
            $e_false
        }
    }};
    ($id:ident, $lit:literal, $isa:expr, $e:expr) => {
        if_ext!($id, $lit, $isa, Ok($e), Err(()))
    };
}

macro_rules! if_ext_a {
    ($isa:expr, $($e:expr), *) => {
        if_ext!(A, "A", $isa, $($e), *)
    };
}

macro_rules! if_ext_c {
    ($isa:expr, $($e:expr), *) => {
        if_ext!(C, "C", $isa, $($e), *)
    };
}

macro_rules! if_ext_d {
    ($isa:expr, $($e:expr), *) => {
        if_ext!(D, "D", $isa, $($e), *)
    };
}

macro_rules! if_ext_f {
    ($isa:expr, $($e:expr), *) => {
        if_ext!(F, "F", $isa, $($e), *)
    };
}

macro_rules! if_ext_m {
    ($isa:expr, $($e:expr), *) => {
        if_ext!(M, "M", $isa, $($e), *)
    };
}

macro_rules! if_ext_zicsr {
    ($isa:expr, $($e:expr), *) => {
        if_ext!(Zicsr, "Zicsr", $isa, $($e), *)
    };
}

macro_rules! if_ext_zifencei {
    ($isa:expr, $($e:expr), *) => {
        if_ext!(Zifencei, "Zifencei", $isa, $($e), *)
    };
}

/// `result` = `val`\[`high`:`low`\]
pub fn select_bits<T>(val: T, high: u8, low: u8) -> T
where
    T: Copy + From<u8> + Shr<Output = T> + Shl<Output = T> + BitAnd<Output = T> + Sub<Output = T>,
{
    let one = T::from(1);
    let len = T::from(high - low + 1);
    let mask = (one << len) - one;
    (val >> T::from(low)) & mask
}

/// test if `bit` is set in `val`
pub fn test_bit<T>(val: T, bit: u8) -> bool
where
    T: Copy + From<u8> + Shl<Output = T> + BitAnd<Output = T> + Eq,
{
    let mask = T::from(1) << T::from(bit);
    val & mask == mask
}

/// group_x = `val`\[`high_x`:`low_x`\]\
/// result = \[`...` : `group_1` : `group_0` : `shift`\]
///
/// # Arguments
///
/// * `val` - value to be shuffled
/// * `shift` - shift amount
/// * `(high, low)+` - bit groups to be selected
///
macro_rules! shuffle_bits {
    ($val:expr, $shift:literal, $($high:literal, $low:literal), *) => {
        {
            let val = $val as u32;
            let mut res = 0;
            let mut _pos = $shift;
            $(
                res |= select_bits(val, $high, $low) << _pos;
                _pos += $high - $low + 1;
            )*
            res
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        assert_eq!(sext(5, 2), -3);
        assert_eq!(select_bits(0b0011_1000, 6, 2), 0b01110);
        assert_eq!(select_bits(0b0011_1000, 5, 5), 0b1);
        assert!(!test_bit(0b0011_1000, 6));
        assert!(test_bit(0b0011_1000, 5));
        assert_eq!(shuffle_bits!(0b1010_0101, 2, 6, 4, 2, 2), 0b10_1000);
        assert_eq!(
            shuffle_bits!(0b0011_1100, 3, 4, 4, 7, 5, 3, 1),
            0b11_0001_1000
        );
    }
}
