/// same as using `as` keyword
pub trait Cast<T>: Sized {
    fn from(value: T) -> Self;
    fn into(self) -> T;
}

pub trait CastPrimitive:
    Cast<u8>
    + Cast<u16>
    + Cast<u32>
    + Cast<u64>
    + Cast<u128>
    + Cast<i8>
    + Cast<i16>
    + Cast<i32>
    + Cast<i64>
    + Cast<i128>
{
}

macro_rules! impl_cast {
    ($t:ty, $($u:ty),*) => {
        $(
            impl Cast<$u> for $t {
                fn from(value: $u) -> Self {
                    value as Self
                }
                fn into(self) -> $u {
                    self as $u
                }
            }
        )*
    };
}

macro_rules! impl_cast_primitive {
    ($t:ty) => {
        impl_cast!($t, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
        impl CastPrimitive for $t {}
    };
}

/// trait for xlen operations that differ between xlens
pub trait XlenOp: Sized {
    /// high parts for signed * signed
    #[cfg(feature = "M")]
    fn mulh(self, rhs: Self) -> Self;
    /// high parts for unsigned * unsigned
    #[cfg(feature = "M")]
    fn mulhu(self, rhs: Self) -> Self;
    /// high parts for signed * unsigned
    #[cfg(feature = "M")]
    fn mulhsu(self, rhs: Self) -> Self;
}

#[cfg(feature = "M")]
fn u64_mul_h32(lhs: u64, rhs: u64) -> u32 {
    let res = lhs.wrapping_mul(rhs);
    (res >> 32) as u32
}

impl XlenOp for u32 {
    #[cfg(feature = "M")]
    fn mulhu(self, rhs: Self) -> Self {
        u64_mul_h32(self as u64, rhs as u64)
    }
    #[cfg(feature = "M")]
    fn mulh(self, rhs: Self) -> Self {
        u64_mul_h32(self as i32 as u64, rhs as i32 as u64)
    }
    #[cfg(feature = "M")]
    fn mulhsu(self, rhs: Self) -> Self {
        u64_mul_h32(self as i32 as u64, rhs as u64)
    }
}

#[cfg(all(feature = "M", feature = "RV64"))]
fn u128_mul_h64(lhs: u128, rhs: u128) -> u64 {
    let res = lhs.wrapping_mul(rhs);
    (res >> 64) as u64
}

#[cfg(feature = "RV64")]
impl XlenOp for u64 {
    #[cfg(feature = "M")]
    fn mulhu(self, rhs: Self) -> Self {
        u128_mul_h64(self as u128, rhs as u128)
    }
    #[cfg(feature = "M")]
    fn mulh(self, rhs: Self) -> Self {
        u128_mul_h64(self as i64 as u128, rhs as i64 as u128)
    }
    #[cfg(feature = "M")]
    fn mulhsu(self, rhs: Self) -> Self {
        u128_mul_h64(self as i64 as u128, rhs as u128)
    }
}

/// -> (low, high)
#[cfg(all(feature = "M", feature = "RV128"))]
fn u256_mul(lhs_low: u128, lhs_high: u128, rhs_low: u128, rhs_high: u128) -> (u128, u128) {
    let res0 = lhs_low.wrapping_mul(rhs_low);
    let res1 = lhs_high
        .wrapping_mul(rhs_low)
        .wrapping_add(lhs_low.wrapping_mul(rhs_high))
        .wrapping_add(lhs_low.mulhu(rhs_low));
    (res0, res1)
}

#[cfg(feature = "RV128")]
impl XlenOp for u128 {
    #[cfg(feature = "M")]
    fn mulhu(self, rhs: Self) -> Self {
        let al = self as u64;
        let ah = (self >> 64) as u64;
        let bl = rhs as u64;
        let bh = (rhs >> 64) as u64;
        // |  res3  |  res2  |  res1  |  res0  |
        // |        |        | albl_h | albl_l | al * bl
        // |        | albh_h | albh_l |        | al * bh
        // |        | ahbl_h | ahbl_l |        | ah * bl
        // | ahbh_h | ahbh_l |        |        | ah * bh
        let albl_h = al.mulhu(bl);
        let ahbl_l = ah.mul(bl);
        let ahbl_h = ah.mulhu(bl);
        let albh_l = al.mul(bh);
        let albh_h = al.mulhu(bh);
        let ahbh_l = ah.mul(bh);
        let ahbh_h = ah.mulhu(bh);
        let (res1, c0) = albl_h.overflowing_add(albh_l);
        let c1 = res1.overflowing_add(ahbl_l).1;
        let (res2, c2) = albh_h.adc(ahbl_h, c0);
        let (res2, c3) = res2.adc(ahbh_l, c1);
        let res3 = ahbh_h.wrapping_add(c2 as u64 + c3 as u64);
        (res2 as u128) | ((res3 as u128) << 64)
    }
    #[cfg(feature = "M")]
    fn mulh(self, rhs: Self) -> Self {
        u256_mul(self, self.sign_blast(), rhs, rhs.sign_blast()).1
    }
    #[cfg(feature = "M")]
    fn mulhsu(self, rhs: Self) -> Self {
        u256_mul(self, self.sign_blast(), rhs, 0).1
    }
}

/// abstracts xlen (rv32, rv64, rv128) out of emulator logic\
/// so emulator runs on XlenT trait instead of concrete types\
/// implements all ALU operations required by riscv spec
pub trait XlenT:
    Sized
    + Copy
    + Default
    + Ord
    + std::fmt::Display
    + std::fmt::Debug
    + std::ops::BitAnd<Output = Self>
    + std::ops::BitOr<Output = Self>
    + std::ops::BitXor<Output = Self>
    + std::ops::Shl<u32, Output = Self>
    + std::ops::Shr<u32, Output = Self>
    + CastPrimitive
    + Cast<Self>
    + XlenOp
{
    const XLEN: u32;
    fn add<T>(self, rhs: T) -> Self
    where
        Self: Cast<T>;
    fn sub<T>(self, rhs: T) -> Self
    where
        Self: Cast<T>;
    /// truncate to low 32 bit
    #[cfg(feature = "RV64")]
    fn trunc32(self) -> Self;
    /// sign-extend from low 32 bit
    #[cfg(feature = "RV64")]
    fn sext32(self) -> Self;
    /// truncate to low 64 bit
    #[cfg(feature = "RV128")]
    fn trunc64(self) -> Self;
    /// sign-extend from low 64 bit
    #[cfg(feature = "RV128")]
    fn sext64(self) -> Self;
    /// signed compare
    fn scmp(self, rhs: Self) -> std::cmp::Ordering;
    /// shift right arithmetic
    fn sra(self, shamt: u32) -> Self;
    /// signed-self >= 0 ? 0 : !0;
    #[cfg(feature = "RV128")]
    fn sign_blast(self) -> Self;
    /// carrying_add in nightly feature
    #[cfg(feature = "RV128")]
    fn adc(self, rhs: Self, carry: bool) -> (Self, bool);
    #[cfg(feature = "M")]
    fn mul(self, rhs: Self) -> Self;
    #[cfg(feature = "M")]
    fn div(self, rhs: Self) -> Self;
    #[cfg(feature = "M")]
    fn rem(self, rhs: Self) -> Self;
    #[cfg(feature = "M")]
    fn divu(self, rhs: Self) -> Self;
    #[cfg(feature = "M")]
    fn remu(self, rhs: Self) -> Self;
}

/// implements XlenT trait for xlen\
/// implements operations that have same logic for all xlens
macro_rules! impl_xlen_t {
    ($t:ty, $s:ty) => {
        impl XlenT for $t {
            const XLEN: u32 = <$t>::BITS;
            fn add<T>(self, rhs: T) -> Self
            where
                Self: Cast<T>,
            {
                self.wrapping_add(<Self as Cast<T>>::from(rhs))
            }
            fn sub<T>(self, rhs: T) -> Self
            where
                Self: Cast<T>,
            {
                self.wrapping_sub(<Self as Cast<T>>::from(rhs))
            }
            #[cfg(feature = "RV64")]
            fn trunc32(self) -> Self {
                self as u32 as Self
            }
            #[cfg(feature = "RV64")]
            fn sext32(self) -> Self {
                self as i32 as Self
            }
            #[cfg(feature = "RV128")]
            fn trunc64(self) -> Self {
                self as u64 as Self
            }
            #[cfg(feature = "RV128")]
            fn sext64(self) -> Self {
                self as i64 as Self
            }
            fn scmp(self, rhs: Self) -> std::cmp::Ordering {
                let lhs = self as $s;
                let rhs = rhs as $s;
                lhs.cmp(&rhs)
            }
            fn sra(self, shamt: u32) -> Self {
                let lhs = self as $s;
                (lhs >> shamt) as $t
            }
            #[cfg(feature = "RV128")]
            fn sign_blast(self) -> Self {
                self.sra(Self::XLEN - 1)
            }
            #[cfg(feature = "RV128")]
            fn adc(self, rhs: Self, carry: bool) -> (Self, bool) {
                let (res, c0) = self.overflowing_add(rhs);
                let (res, c1) = res.overflowing_add(carry as $t);
                (res, c0 || c1)
            }
            #[cfg(feature = "M")]
            fn mul(self, rhs: Self) -> Self {
                self.wrapping_mul(rhs)
            }
            #[cfg(feature = "M")]
            fn div(self, rhs: Self) -> Self {
                let lhs = self as $s;
                let rhs = rhs as $s;
                if rhs == 0 {
                    !0
                } else {
                    lhs.wrapping_div(rhs) as Self
                }
            }
            #[cfg(feature = "M")]
            fn rem(self, rhs: Self) -> Self {
                let lhs = self as $s;
                let rhs = rhs as $s;
                if rhs == 0 {
                    lhs as Self
                } else {
                    lhs.wrapping_rem(rhs) as Self
                }
            }
            #[cfg(feature = "M")]
            fn divu(self, rhs: Self) -> Self {
                if rhs == 0 {
                    !0
                } else {
                    self.wrapping_div(rhs)
                }
            }
            #[cfg(feature = "M")]
            fn remu(self, rhs: Self) -> Self {
                if rhs == 0 {
                    self
                } else {
                    self.wrapping_rem(rhs)
                }
            }
        }
    };
}

impl_cast_primitive!(u32);
#[cfg(feature = "RV64")]
impl_cast_primitive!(u64);
#[cfg(feature = "RV128")]
impl_cast_primitive!(u128);

impl_xlen_t!(u32, i32);
#[cfg(feature = "RV64")]
impl_xlen_t!(u64, i64);
#[cfg(feature = "RV128")]
impl_xlen_t!(u128, i128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        #[cfg(feature = "M")]
        {
            let val1 = 0x80000000u32; // -2147483648
            let val2 = 0xffffffffu32; // -1
            let val3 = 0x7fffffffu32; // 2147483647
            let val4 = 0xffff0000u32; // -65536
            assert_eq!(val1.mulhu(val1), 0x40000000);
            assert_eq!(val1.mulh(val1), 0x40000000);
            assert_eq!(val2.mulh(val2), 0x0);
            assert_eq!(val2.mulh(val1), 0x0);
            assert_eq!(val1.mulh(val3), 0xc0000000);
            assert_eq!(val4.mulh(val4), 0x1);
            assert_eq!(val1.mulhsu(val1), 0xc0000000);
            assert_eq!(val2.mulhsu(val2), 0xffffffff);
            assert_eq!(val1.mulhsu(val2), 0x80000000);
            assert_eq!(val3.mulhsu(val2), 0x7ffffffe);
            assert_eq!(val4.mulhsu(val4), 0xffff0001);
        }

        // u64 has same logic as u32

        #[cfg(all(feature = "M", feature = "RV128"))]
        {
            let val1 = 0x80000000000000000000000000000000u128;
            let val2 = 0xffffffffffffffffffffffffffffffffu128;
            let val3 = 0x7fffffffffffffffffffffffffffffffu128;
            let val4 = 0xffffffffffffffff0000000000000000u128;
            assert_eq!(val1.mulhu(val1), 0x40000000000000000000000000000000);
            assert_eq!(val1.mulh(val1), 0x40000000000000000000000000000000);
            assert_eq!(val2.mulh(val2), 0x0);
            assert_eq!(val2.mulh(val1), 0x0);
            assert_eq!(val1.mulh(val3), 0xc0000000000000000000000000000000);
            assert_eq!(val4.mulh(val4), 0x1);
            assert_eq!(val1.mulhsu(val1), 0xc0000000000000000000000000000000);
            assert_eq!(val2.mulhsu(val2), 0xffffffffffffffffffffffffffffffff);
            assert_eq!(val1.mulhsu(val2), 0x80000000000000000000000000000000);
            assert_eq!(val3.mulhsu(val2), 0x7ffffffffffffffffffffffffffffffe);
            assert_eq!(val4.mulhsu(val4), 0xffffffffffffffff0000000000000001);
        }
    }
}
