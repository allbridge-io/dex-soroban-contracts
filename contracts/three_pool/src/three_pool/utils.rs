use crate::storage::sized_array::SizedU128Array;

pub fn get_triple_tuple_from_sized_u128_array(array: SizedU128Array) -> (u128, u128, u128) {
    (array.get(0usize), array.get(1usize), array.get(2usize))
}
