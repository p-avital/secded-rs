use crate::bitwise::Bitwise;
use crate::*;

use std::collections::{HashMap, VecDeque};
use crate::bitvec::Bitvec;

pub struct SECDED_DYNAMIC {
    encodable_size: usize,
    m: usize,
    encode_matrix: Vec<Bitvec>,
    syndromes: HashMap<Bitvec, Bitvec>,
}

impl SECDED_DYNAMIC {
    #[inline]
    fn increment(vec: &mut Vec<u8>) {
        for x in vec.iter_mut().rev() {
            match x.overflowing_add(1) {
                (_, true) => *x = 0,
                (value, false) => {
                    *x = value;
                    return;
                }
            }
        }
    }
    #[inline]
    fn bin_matrix_product_paritied(matrix: &Vec<Bitvec>, value: &Bitvec) -> Bitvec {
        let ONE = Bitvec(vec![1].into());
        let mut result = Bitvec(VecDeque::new());
        for x in matrix.iter() {
            if (x & value).parity() != 0 {
                result |= &ONE;
            }
            result <<= 1;
        }
        if result.parity() != 0 {
            result |= &ONE;
        }
        result
    }

    pub fn new(encodable_size: usize) -> Self {
        let mut m = 1;
        while 2_usize.pow(m as u32) - m < encodable_size as usize {
            m += 1;
        }
        let m = m;
        let mut encode_matrix = vec![Bitvec(vec![].into()); m];
        let mut i = Bitvec(vec![1].into());
        let max = i.clone() << m;
        while &i <= &max {
            if i.count() < 2 {
                continue;
            }
            for k in 0..(m as usize) {
                encode_matrix[k] <<= 1;
                encode_matrix[k] |= &(&(i.clone() >> (m - 1 - k)) & 1);
            }
            i += 1;
        }
        for i in 0..m {
            encode_matrix[i] <<= (m + 1);
            if i <= m {
                encode_matrix[i] |= &(Bitvec(vec![1].into()) << (m - i));
            }
        }
        let mut syndromes = HashMap::new();
        let mut error = Bitvec(vec![1].into());
        while &error <= &max {
            syndromes.insert(Self::bin_matrix_product_paritied(&encode_matrix, &error), error.clone()).expect("Several vectors with same error");
            error <<= 1;
        }
        SECDED_DYNAMIC {
            m,
            encodable_size,
            encode_matrix,
            syndromes
        }
    }
}
