// abstracts xlen out of emulator logic

pub trait Cast<T>: Sized {
    fn from(value: T) -> Self;
    fn into(self) -> T;
}

macro_rules! cast_types {
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

macro_rules! impl_cast {
    ($($t:ty),*) => {
        $(
            cast_types!($t, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
        )*
    };
}

pub trait XlenOp: Sized {
    /// high parts for signed * signed
    fn mulh(self, rhs: Self) -> Self;
    /// high parts for unsigned * unsigned
    fn mulhu(self, rhs: Self) -> Self;
    /// high parts for signed * unsigned
    fn mulhsu(self, rhs: Self) -> Self;
}

fn u64_mul_h32(lhs: u64, rhs: u64) -> u32 {
    let res = lhs.wrapping_mul(rhs);
    (res >> 32) as u32
}

impl XlenOp for u32 {
    fn mulhu(self, rhs: Self) -> Self {
        u64_mul_h32(self as u64, rhs as u64)
    }
    fn mulh(self, rhs: Self) -> Self {
        u64_mul_h32(self as i32 as u64, rhs as i32 as u64)
    }
    fn mulhsu(self, rhs: Self) -> Self {
        u64_mul_h32(self as i32 as u64, rhs as u64)
    }
}

fn u128_mul_h64(lhs: u128, rhs: u128) -> u64 {
    let res = lhs.wrapping_mul(rhs);
    (res >> 64) as u64
}

impl XlenOp for u64 {
    fn mulhu(self, rhs: Self) -> Self {
        u128_mul_h64(self as u128, rhs as u128)
    }
    fn mulh(self, rhs: Self) -> Self {
        u128_mul_h64(self as i64 as u128, rhs as i64 as u128)
    }
    fn mulhsu(self, rhs: Self) -> Self {
        u128_mul_h64(self as i64 as u128, rhs as u128)
    }
}

fn u256_mul(al: u128, ah: u128, bl: u128, bh: u128) -> (u128, u128) {
    let res0 = al.wrapping_mul(bl);
    let res1 = ah
        .wrapping_mul(bl)
        .wrapping_add(al.wrapping_mul(bh))
        .wrapping_add(al.mulhu(bl));
    (res0, res1)
}

impl XlenOp for u128 {
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
    fn mulh(self, rhs: Self) -> Self {
        u256_mul(self, self.sign_blast(), rhs, rhs.sign_blast()).1
    }
    fn mulhsu(self, rhs: Self) -> Self {
        u256_mul(self, self.sign_blast(), rhs, 0).1
    }
}

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
    + Cast<u8>
    + Cast<u16>
    + Cast<u32>
    + Cast<u64>
    + Cast<u128>
    + Cast<i8>
    + Cast<i16>
    + Cast<i32>
    + Cast<i64>
    + Cast<i128>
    + Cast<Self>
    + XlenOp
{
    fn xlen() -> u32;
    fn add<T>(self, rhs: T) -> Self
    where
        Self: Cast<T>;
    fn sub<T>(self, rhs: T) -> Self
    where
        Self: Cast<T>;
    /// truncate to low 32 bit
    fn trunc32(self) -> Self;
    /// sign-extend from low 32 bit
    fn sext32(self) -> Self;
    /// truncate to low 64 bit
    fn trunc64(self) -> Self;
    /// sign-extend from low 64 bit
    fn sext64(self) -> Self;
    /// signed compare
    fn scmp(self, rhs: Self) -> std::cmp::Ordering;
    /// shift right arithmetic
    fn sra(self, shamt: u32) -> Self;
    /// signed-self >= 0 ? 0 : -1;
    fn sign_blast(self) -> Self;
    /// carrying_add in nightly feature
    fn adc(self, rhs: Self, carry: bool) -> (Self, bool);
    fn mul(self, rhs: Self) -> Self;
    fn div(self, rhs: Self) -> Self;
    fn rem(self, rhs: Self) -> Self;
    fn divu(self, rhs: Self) -> Self;
    fn remu(self, rhs: Self) -> Self;
}

macro_rules! impl_xlen_t {
    ($($t:ty, $s:ty), *) => {
        $(
            impl XlenT for $t {
                fn xlen() -> u32 {
                    <$t>::BITS
                }
                fn add<T>(self, rhs: T) -> Self
                where
                    Self: Cast<T>
                {
                    self.wrapping_add(<Self as Cast<T>>::from(rhs))
                }
                fn sub<T>(self, rhs: T) -> Self
                where
                    Self: Cast<T>
                {
                    self.wrapping_sub(<Self as Cast<T>>::from(rhs))
                }
                fn trunc32(self) -> Self {
                    self as u32 as Self
                }
                fn sext32(self) -> Self {
                    self as i32 as Self
                }
                fn trunc64(self) -> Self {
                    self as u64 as Self
                }
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
                fn adc(self, rhs: Self, carry: bool) -> (Self, bool) {
                    let (res, c0) = self.overflowing_add(rhs);
                    let (res, c1) = res.overflowing_add(carry as $t);
                    (res, c0 || c1)
                }
                fn sign_blast(self) -> Self {
                    self.sra(Self::xlen() - 1)
                }
                fn mul(self, rhs: Self) -> Self {
                    self.wrapping_mul(rhs)
                }
                fn div(self, rhs: Self) -> Self {
                    let lhs = self as $s;
                    let rhs = rhs as $s;
                    if rhs == 0 {
                        !0
                    } else {
                        lhs.wrapping_div(rhs) as Self
                    }
                }
                fn rem(self, rhs: Self) -> Self {
                    let lhs = self as $s;
                    let rhs = rhs as $s;
                    if rhs == 0 {
                        lhs as Self
                    } else {
                        lhs.wrapping_rem(rhs) as Self
                    }
                }
                fn divu(self, rhs: Self) -> Self {
                    if rhs == 0 {
                        !0
                    } else {
                        self.wrapping_div(rhs)
                    }
                }
                fn remu(self, rhs: Self) -> Self {
                    if rhs == 0 {
                        self
                    } else {
                        self.wrapping_rem(rhs)
                    }
                }
            }
        )*
    };
}

impl_cast!(u32, u64, u128);
impl_xlen_t!(u32, i32, u64, i64, u128, i128);

#[cfg(test)]
mod xlenop_tests {
    use super::*;

    #[test]
    fn u32_xlenop() {
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

    #[test]
    fn u128_xlenop() {
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
