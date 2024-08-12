use ethnum::U256;

use crate::Error;

pub fn sqrt(n: &U256) -> U256 {
    if *n == U256::ZERO {
        return U256::ZERO;
    }
    let shift: u32 = (255 - n.leading_zeros()) & !1;
    let mut bit = U256::ONE << shift;

    let mut n = *n;
    let mut result = U256::ZERO;
    for _ in (0..shift + 1).step_by(2) {
        let res_bit = result + bit;
        result >>= 1;
        if n >= res_bit {
            n -= res_bit;
            result += bit;
        }
        bit >>= 2;
    }
    result
}

pub fn cbrt(n: &U256) -> Result<u128, Error> {
    let leading_zeros = n.leading_zeros();
    if leading_zeros < 128 {
        let lo = cbrt(&(n >> 3u8))? << 1;
        let hi = lo + 1;

        if cube(U256::new(hi))? <= *n {
            Ok(hi)
        } else {
            Ok(lo)
        }
    } else {
        Ok(cbrt128(n.as_u128()))
    }
}

pub fn cbrt128(n: u128) -> u128 {
    if n <= u64::MAX as u128 {
        cbrt64(n as u64) as u128
    } else {
        let lo = cbrt128(n >> 3) << 1;
        let hi = lo + 1;
        if hi * hi * hi <= n {
            hi
        } else {
            lo
        }
    }
}

pub fn cbrt64(mut n: u64) -> u64 {
    let mut x = 0u64;
    let mut shift = 63;
    while shift >= 0 {
        x <<= 1;
        let z = x * 3 * (x + 1) + 1;
        if (n >> shift) >= z {
            n -= z << shift;
            x += 1;
        }

        shift -= 3;
    }
    x
}

#[inline]
pub fn cube(v: U256) -> Result<U256, Error> {
    v.checked_mul(v)
        .and_then(|v2| v2.checked_mul(v))
        .ok_or(Error::U256Overflow)
}

#[inline]
pub fn square(v: U256) -> Result<U256, Error> {
    v.checked_mul(v).ok_or(Error::U256Overflow)
}
