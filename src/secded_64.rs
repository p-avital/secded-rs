/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::bitwise::Bitwise;
use crate::*;
#[repr(C)]
pub struct SecDed64 {
    encodable_size: u8,
    m: u8,
    mask: u8,
    pub(crate) encode_matrix: [u64; 6],
    syndromes: [u16; 64],
}

impl SecDed64 {
    #[inline]
    fn bin_matrix_product_paritied(matrix: &[u64], value: u64) -> u64 {
        let mut result = 0u64;
        for x in matrix.iter() {
            result ^= (*x & value).parity();
            result <<= 1;
        }
        result |= result.parity();
        result
    }

    pub fn new(encodable_size: usize) -> Self {
        if encodable_size > 57 {
            panic!("This implementation is based on u64, and can thus only encode payloads of at most 57 bits");
        }
        let m = hamming_size(encodable_size);
        let mut encode_matrix = [0; 6];
        for i in 1..=(2_u64.pow(m as u32)) {
            if i.count() < 2 {
                continue;
            }
            for (k, x) in encode_matrix.iter_mut().enumerate().take(m) {
                *x <<= 1;
                *x |= i >> (m - 1 - k) & 1;
            }
        }
        for (i, x) in encode_matrix.iter_mut().enumerate().take(m) {
            *x <<= m + 1;
            if i <= m {
                *x |= 1 << (m - i);
            }
        }
        let mut syndromes = [0; 64];
        for error_bit in 0..=(encodable_size + m) {
            let error: u64 = 1u64 << error_bit;
            syndromes[error_bit] =
                Self::bin_matrix_product_paritied(&encode_matrix[0..m], error) as u16;
        }
        for (i, x) in syndromes[..=(encodable_size + m)].iter().enumerate() {
            for y in syndromes[i + 1..=(encodable_size + m)].iter() {
                assert_ne!(x, y);
            }
        }
        SecDed64 {
            encodable_size: encodable_size as u8,
            m: m as u8,
            mask: { (0..=m).map(|x| 1u8 << x).sum::<u8>() },
            encode_matrix,
            syndromes,
        }
    }

    #[cfg(feature = "no-panics")]
    #[inline]
    fn encode_assertions(&self, _buffer: u64) {}

    #[cfg(not(feature = "no-panics"))]
    #[inline]
    fn encode_assertions(&self, encodable: u64) {
        match encodable & (self.mask as u64) {
            0 => {}
            _ => {
                let mut buffer: [u8; 8] = unsafe { core::mem::uninitialized() };
                byteorder::BigEndian::write_u64(&mut buffer[..], encodable);
                panic!(
                    "{:?} overlaps with the code-correction slot, which is the right-most {} bits ",
                    buffer.as_ref(),
                    self.code_size(),
                );
            }
        }
        match 1u64.overflowing_shl((self.encodable_size + self.m + 1) as u32) {
            (value, false) if encodable > value => {
                let mut buffer: [u8; 8] = unsafe { core::mem::uninitialized() };
                byteorder::BigEndian::write_u64(&mut buffer[..], encodable);
                panic!(
                    "{:?} is too big to be encoded on {} bits",
                    buffer.as_ref(),
                    self.encodable_size as usize + self.code_size()
                );
            }
            _ => {}
        };
    }
}

impl SecDedCodec for SecDed64 {
    fn encodable_size(&self) -> usize {
        self.encodable_size as usize
    }
    fn code_size(&self) -> usize {
        (self.m + 1) as usize
    }
    fn expected_slice_size(&self) -> Option<usize> {
        Some(8)
    }

    /// Encodes the data IN-PLACE
    /// # Arguments:
    /// * `data`: The slice of data to encode. The last `secded.code_size()` bits MUST be set to 0.
    /// # Panics:
    /// Panics if `data.len() != 8`
    ///
    /// Unless you use the `no-panics` feature, encoding will also panic if the data you try to encode has some
    /// bits set to 1 in the reserved space, or past the `encodable_size() + code_size()` rightmost bits
    fn encode(&self, buffer: &mut [u8]) {
        let mut encodable = byteorder::BigEndian::read_u64(&buffer[..]);
        self.encode_assertions(encodable);
        let code =
            Self::bin_matrix_product_paritied(&self.encode_matrix[..self.m as usize], encodable);
        encodable |= code;
        byteorder::BigEndian::write_u64(&mut buffer[..], encodable);
    }

    /// Decodes the data IN-PLACE
    /// # Arguments:
    /// * `data`: The slice of data to decode.  
    /// The last `secded.code_size()` bits will be reset to 0, a single error will be corrected implicitly.
    /// # Returns:
    /// `Ok(())` if the data slice's correctness has been checked: 0 error found or 1 found and corrected.
    /// `Err(())` if 2 errors were detected.
    /// # Panics:
    /// Panics if `data.len() != 8`
    fn decode(&self, buffer: &mut [u8]) -> Result<(), ()> {
        let mut decodable = byteorder::BigEndian::read_u64(&buffer);
        let syndrome =
            Self::bin_matrix_product_paritied(&self.encode_matrix[..self.m as usize], decodable)
                as u16;
        if syndrome == 0 {
            buffer[7] &= !self.mask;
            return Ok(());
        }
        for (i, s) in self.syndromes.iter().enumerate() {
            if *s == syndrome {
                decodable ^= 1 << i;
                byteorder::BigEndian::write_u64(buffer, decodable);
                buffer[7] &= !self.mask;
                return Ok(());
            }
        }
        Err(())
    }
}

#[cfg(feature = "bench")]
#[bench]
fn encode(b: &mut test::Bencher) {
    let secded = SecDed64::new(57);
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
    let secded = SecDed64::new(57);
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
    let secded = SecDed64::new(57);
    let expected = [0, 0, 0, 0, 0, 5, 0, 0];
    let mut buffer = expected;
    secded.encode(&mut buffer);
    let mut i = 3;
    let mut j = 7;
    let mut should_panic = false;
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
    });
    assert!(!should_panic)
}

#[test]
fn codec() {
    let secded = SecDed64::new(57);
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
