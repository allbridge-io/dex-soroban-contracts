use ethnum::U256;

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