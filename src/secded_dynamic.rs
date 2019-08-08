use crate::bitwise::Bitwise;
use crate::*;

use crate::bitvec::Bitvec;
use std::collections::{HashMap, VecDeque};

pub struct SecDedDynamic {
    encodable_size: usize,
    m: usize,
    #[cfg(not(feature = "no-panics"))]
    max: Bitvec,
    mask: Bitvec,
    encode_matrix: Vec<Bitvec>,
    syndromes: HashMap<Bitvec, Bitvec>,
}
lazy_static::lazy_static! {
    static ref BITVEC_ONE: Bitvec = bitvec![1];
}

impl SecDedDynamic {
    #[inline]
    fn bin_matrix_product_paritied(matrix: &[Bitvec], value: &Bitvec) -> Bitvec {
        let one: &Bitvec = &BITVEC_ONE;
        let mut result = bitvec![];
        for x in matrix.iter() {
            if (x & value).parity() != 0 {
                result |= one;
            }
            result <<= 1;
        }
        if result.parity() != 0 {
            result |= one;
        }
        result
    }

    pub fn new(encodable_size: usize) -> Self {
        let mut m = 1;
        while (1 << m) - m < encodable_size as usize {
            m += 1;
        }
        let mut encode_matrix = vec![Bitvec(vec![].into()); m];
        let mut i = Bitvec(vec![1].into());
        let max = i.clone() << m;
        while i < max {
            if i.count() < 2 {
                i += 1;
                continue;
            }
            for (k, x) in encode_matrix.iter_mut().enumerate() {
                *x <<= 1;
                *x |= i.nth_bit_from_right(m - 1 - k);
            }
            i += 1;
        }
        for (i, x) in encode_matrix.iter_mut().enumerate() {
            *x <<= m + 1;
            if i <= m {
                *x |= &(Bitvec(vec![1].into()) << (m - i));
            }
        }
        let max = bitvec![1] << (encodable_size + m + 1);
        let mut syndromes = HashMap::new();
        let mut error = Bitvec(vec![1].into());
        while error < max {
            let syndrome = Self::bin_matrix_product_paritied(encode_matrix.as_ref(), &error);
            if let Some(other) = syndromes.insert(syndrome, error.clone()) {
                let mut syndrome = None;
                for (syn, err) in syndromes.iter() {
                    if &error == err {
                        syndrome = Some(syn.clone());
                        break;
                    }
                }
                panic!(
                    "{:?} and {:?} have the same syndrome: {:?}",
                    other,
                    error,
                    syndrome.unwrap()
                );
            }
            error <<= 1;
        }
        let mut mask = bitvec!();
        for _ in 0..m {
            mask <<= 1;
            mask |= 1;
        }
        SecDedDynamic {
            m,
            #[cfg(not(feature = "no-panics"))]
            max,
            mask,
            encodable_size,
            encode_matrix,
            syndromes,
        }
    }

    #[cfg(feature = "no-panics")]
    #[inline]
    fn encode_assertions(&self, _buffer: &Bitvec) {}

    #[cfg(not(feature = "no-panics"))]
    #[inline]
    fn encode_assertions(&self, encodable: &Bitvec) {
        if !(encodable & &self.mask).is_null() {
            panic!(
                "{:?} overlaps with the code-correction slot, which is the right-most {} bits ",
                encodable,
                self.code_size(),
            );
        }

        if encodable > &self.max {
            panic!(
                "{:?} is too big to be encoded on {} bits",
                encodable,
                self.encodable_size + self.code_size()
            );
        }
    }
}

#[test]
fn bin_matrix() {
    let dynamic = SecDedDynamic::new(57);
    let fixed = SecDed64::new(57);
    for (i, x) in dynamic.encode_matrix.iter().enumerate() {
        assert_eq!(fixed.encode_matrix[i], x.to_u64_be())
    }
}

fn copy_into(binvec: &Bitvec, buffer: &mut [u8]) {
    let (v_i, b_i) = if binvec.0.len() > buffer.len() {
        (0, binvec.0.len() - buffer.len())
    } else {
        (buffer.len() - binvec.0.len(), 0)
    };
    for (v, b) in binvec.0.iter().skip(v_i).zip(buffer.iter_mut().skip(b_i)) {
        *b = *v;
    }
}

impl SecDedCodec for SecDedDynamic {
    fn encodable_size(&self) -> usize {
        self.encodable_size
    }
    fn code_size(&self) -> usize {
        self.m + 1
    }
    fn encode(&self, data: &mut [u8]) {
        let mut buffer: Bitvec = Bitvec({
            let mut inner = VecDeque::with_capacity(data.len());
            for &x in data.iter() {
                inner.push_back(x);
            }
            inner
        });
        self.encode_assertions(&buffer);
        buffer |= &Self::bin_matrix_product_paritied(self.encode_matrix.as_ref(), &buffer);
        copy_into(&buffer, data);
    }
    fn decode(&self, data: &mut [u8]) -> Result<(), ()> {
        let mut buffer: Bitvec = Bitvec({
            let mut inner = VecDeque::with_capacity(data.len());
            for &x in data.iter() {
                inner.push_back(x);
            }
            inner
        });
        let syndrome = Self::bin_matrix_product_paritied(self.encode_matrix.as_ref(), &buffer);
        if syndrome.is_null() {
            self.mask.mask_not_buffer(data);
            return Ok(());
        }
        if let Some(correction) = self.syndromes.get(&syndrome) {
            buffer ^= correction;
            copy_into(&buffer, data);
            self.mask.mask_not_buffer(data);
            Ok(())
        } else {
            Err(())
        }
    }
}

#[test]
fn codec() {
    let secded = SecDedDynamic::new(57);
    let expected = [0, 0, 0, 0, 5, 0, 0, 0];
    let mut encode_buffer = expected;
    secded.encode(&mut encode_buffer);
    let mut should_panic = false;
    for i in 0..64 {
        let mut local_buffer = encode_buffer;
        local_buffer[i / 8] ^= 1 << (i % 8);
        match secded.decode(&mut local_buffer) {
            Ok(()) => {
                if local_buffer != expected {
                    eprintln!("{:?} != {:?}", local_buffer, expected);
                    should_panic = true;
                }
            }
            Err(()) => {
                eprintln!("{:?} Decode Failed", local_buffer);
                should_panic = true;
            }
        };
    }
    assert!(!should_panic)
}

#[cfg(feature = "bench")]
#[bench]
fn encode(b: &mut test::Bencher) {
    let secded = SecDedDynamic::new(57);
    let expected = [0, 0, 0, 0, 5, 0, 0];
    let mut buffer = [0u8; 8];
    buffer[1..].clone_from_slice(&expected);
    b.iter(|| {
        if buffer[0] > 0 {
            buffer[0] = 0;
            buffer[1..].clone_from_slice(&expected);
        }
        secded.encode(&mut buffer);
    })
}

#[cfg(feature = "bench")]
#[bench]
fn decode(b: &mut test::Bencher) {
    let secded = SecDedDynamic::new(57);
    let expected = [0, 0, 0, 0, 5, 0, 0];
    let mut buffer = [0u8; 8];
    buffer[1..].clone_from_slice(&expected);
    secded.encode(&mut buffer);
    b.iter(|| {
        let mut local_buffer = buffer;
        secded.decode(&mut local_buffer).unwrap();
    })
}

#[cfg(feature = "bench")]
#[bench]
fn decode_1err(b: &mut test::Bencher) {
    let secded = SecDedDynamic::new(57);
    let expected = [0, 0, 0, 0, 5, 0, 0];
    let mut buffer = [0u8; 8];
    buffer[..7].clone_from_slice(&expected);
    secded.encode(&mut buffer);
    let mut i = 3;
    let mut j = 7;
    b.iter(|| {
        let mut local_buffer = buffer;
        j += 1;
        if j > 7 {
            j = 0;
            i -= 1;
            if i < 1 {
                i = 7;
            }
        }
        local_buffer[i] ^= 1 << j;
        secded.decode(&mut local_buffer).unwrap();
    })
}
