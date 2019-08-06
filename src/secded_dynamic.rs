use crate::bitwise::Bitwise;
use crate::*;

use crate::bitvec::Bitvec;
use std::collections::{HashMap, VecDeque};

pub struct SecdedDynamic {
    encodable_size: usize,
    m: usize,
    max: Bitvec,
    encode_matrix: Vec<Bitvec>,
    syndromes: HashMap<Bitvec, Bitvec>,
}

impl SecdedDynamic {
    #[inline]
    fn bin_matrix_product_paritied(matrix: &[Bitvec], value: &Bitvec) -> Bitvec {
        let one = Bitvec(vec![1].into());
        let mut result = bitvec![];
        for x in matrix.iter() {
            if (x & value).parity() != 0 {
                result |= &one;
            }
            result <<= 1;
        }
        if result.parity() != 0 {
            result |= &one;
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
            let syndrome = Self::bin_matrix_product_paritied(&encode_matrix, &error);
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
        SecdedDynamic {
            m,
            max,
            encodable_size,
            encode_matrix,
            syndromes,
        }
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

impl SecDedCodec for SecdedDynamic {
    fn encodable_size(&self) -> usize {
        self.encodable_size
    }
    fn code_size(&self) -> usize {
        self.m + 1
    }
    fn encode(&self, data: &mut [u8]) {
        let buffer: Bitvec = data.as_ref().into();
        if buffer > self.max {
            panic!(
                "data: {:?} is too big to encode with {} encodable bits",
                data.as_ref(),
                self.encodable_size
            )
        }
        let buffer = Self::bin_matrix_product_paritied(self.encode_matrix.as_ref(), &buffer);
        copy_into(&buffer, data);
    }
    fn decode(&self, data: &mut [u8]) -> Result<(), ()> {
        let mut buffer: Bitvec = data.as_ref().into();
        if buffer > self.max {
            panic!(
                "data: {:?} is too big to encode with {} encodable bits",
                data.as_ref(),
                self.encodable_size
            )
        }
        let syndrome = Self::bin_matrix_product_paritied(self.encode_matrix.as_ref(), &buffer);
        if syndrome == bitvec![] {
            return Ok(());
        }
        if let Some(correction) = self.syndromes.get(&syndrome) {
            buffer ^= correction;
            copy_into(&buffer, data);
            Ok(())
        } else {
            Err(())
        }
    }
}

#[test]
fn codec() {
    let secded = SecdedDynamic::new(57);
    let expected = [0, 0, 0, 0, 0, 5, 0, 0];
    let mut buffer = expected;
    secded.encode(&mut buffer);
    secded.decode(&mut buffer).unwrap();
    assert_eq!(buffer[..7], expected[..7]);
}

#[cfg(feature = "bench")]
#[bench]
fn encode_bench(b: &mut test::Bencher) {
    let secded = SecdedDynamic::new(57);
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
fn decode_bench(b: &mut test::Bencher) {
    let secded = SecdedDynamic::new(57);
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
fn decode_1err_bench(b: &mut test::Bencher) {
    let secded = SecdedDynamic::new(57);
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
