use crate::bitwise::Bitwise;
use crate::*;
#[repr(C)]
pub struct SecDed128 {
    encodable_size: u8,
    m: u8,
    mask: u8,
    encode_matrix: [u128; 7],
    syndromes: [u16; 128],
}

impl SecDed128 {
    #[inline]
    fn bin_matrix_product_paritied(matrix: &[u128], value: u128) -> u128 {
        let mut result = 0;
        for x in matrix.iter() {
            result ^= (*x & value).parity();
            result <<= 1;
        }
        result |= result.parity();
        result
    }

    pub fn new(encodable_size: usize) -> Self {
        if encodable_size > 120 {
            panic!("This implementation is based on u64, and can thus only encode payloads of at most 57 bits");
        }
        let mut m = 1;
        while 2_usize.pow(m as u32) - m < encodable_size as usize {
            m += 1;
        }
        let mut encode_matrix = [0; 7];
        for i in 1..(2_u128.pow(m as u32) + 1) {
            if i.count() < 2 {
                continue;
            }
            for k in 0..(m as usize) {
                encode_matrix[k] <<= 1;
                encode_matrix[k] |= i >> (m - 1 - k) & 1;
            }
        }
        for i in 0..7 {
            encode_matrix[i] = encode_matrix[i] << (m + 1);
            if i <= m {
                encode_matrix[i] |= 1 << (m - i);
            }
        }
        let mut syndromes = [0; 128];
        for error_bit in 0..=(encodable_size + m) {
            let error: u128 = 1u128 << error_bit;
            syndromes[error_bit] =
                Self::bin_matrix_product_paritied(&encode_matrix[0..m], error) as u16;
        }
        for (i, x) in syndromes[..=(encodable_size + m)].iter().enumerate() {
            for y in syndromes[i + 1..=(encodable_size + m)].iter() {
                assert_ne!(x, y);
            }
        }
        SecDed128 {
            encodable_size: encodable_size as u8,
            m: m as u8,
            mask: { (0..=m).map(|x| 1u8 << x).sum::<u8>() },
            encode_matrix,
            syndromes,
        }
    }

    #[cfg(feature = "no-panics")]
    #[inline]
    fn encode_assertions(&self, _encodable: u128) {}

    #[cfg(not(feature = "no-panics"))]
    #[inline]
    fn encode_assertions(&self, encodable: u128) {
        match encodable & self.mask as u128 {
            0 => {}
            _ => {
                let mut buffer: [u8; 16] = unsafe { std::mem::uninitialized() };
                byteorder::BigEndian::write_u128(&mut buffer[..], encodable);
                panic!(
                    "{:?} overlaps with the code-correction slot, which is the right-most {} bits ",
                    buffer.as_ref(),
                    self.code_size(),
                );
            }
        }
        match 1u128.overflowing_shl((self.encodable_size + self.m + 1) as u32) {
            (value, false) if encodable > value => {
                let mut buffer: [u8; 16] = unsafe { std::mem::uninitialized() };
                byteorder::BigEndian::write_u128(&mut buffer[..], encodable);
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

impl SecDedCodec for SecDed128 {
    fn encodable_size(&self) -> usize {
        self.encodable_size as usize
    }
    fn code_size(&self) -> usize {
        (self.m + 1) as usize
    }
    fn expected_slice_size(&self) -> Option<usize> {
        Some(16)
    }

    /// Encodes the data IN-PLACE
    /// # Arguments:
    /// * `data`: The slice of data to encode. The last `secded.code_size()` bits MUST be set to 0.
    /// # Panics:
    /// Panics if `data.len() != 16`
    ///
    /// Unless you use the `no-panics` feature, encoding will also panic if the data you try to encode has some
    /// bits set to 1 in the reserved space, or past the `encodable_size() + code_size()` rightmost bits
    fn encode(&self, buffer: &mut [u8]) {
        let mut encodable = byteorder::BigEndian::read_u128(buffer);
        self.encode_assertions(encodable);
        let code =
            Self::bin_matrix_product_paritied(&self.encode_matrix[..self.m as usize], encodable);
        encodable |= code;
        byteorder::BigEndian::write_u128(&mut buffer[..], encodable);
    }

    /// Decodes the data IN-PLACE
    /// # Arguments:
    /// * `data`: The slice of data to decode.  
    /// The last `secded.code_size()` bits will be reset to 0, a single error will be corrected implicitly.
    /// # Returns:
    /// `Ok(())` if the data slice's correctness has been checked: 0 error found or 1 found and corrected.
    /// `Err(())` if 2 errors were detected.
    /// # Panics:
    /// Panics if `data.len() != 16`
    fn decode(&self, buffer: &mut [u8]) -> Result<(), ()> {
        let mut decodable = byteorder::BigEndian::read_u128(buffer);
        let syndrome =
            Self::bin_matrix_product_paritied(&self.encode_matrix[..self.m as usize], decodable)
                as u16;
        if syndrome == 0 {
            buffer[15] &= !self.mask;
            return Ok(());
        }
        for (i, s) in self.syndromes.iter().enumerate() {
            if *s == syndrome {
                decodable ^= 1 << i;
                byteorder::BigEndian::write_u128(buffer, decodable);
                buffer[15] &= !self.mask;
                return Ok(());
            }
        }
        Err(())
    }
}

#[cfg(feature = "bench")]
#[bench]
fn encode(b: &mut test::Bencher) {
    let secded = SecDed128::new(57);
    let mut buffer = [0u8; 16];
    buffer[13] = 5;
    b.iter(|| {
        if buffer[0] > 0 {
            buffer[0] = 0;
            buffer[15] = 0;
        }
        secded.encode(&mut buffer);
    })
}

#[cfg(feature = "bench")]
#[bench]
fn decode(b: &mut test::Bencher) {
    let secded = SecDed128::new(57);
    let mut buffer = [0u8; 16];
    buffer[13] = 5;
    secded.encode(&mut buffer);
    b.iter(|| {
        let mut local_buffer = buffer;
        secded.decode(&mut local_buffer).unwrap();
    })
}

#[cfg(feature = "bench")]
#[bench]
fn decode_1err(b: &mut test::Bencher) {
    let secded = SecDed128::new(57);
    let mut buffer = [0u8; 16];
    buffer[13] = 5;
    secded.encode(&mut buffer);
    let mut i = 15;
    let mut j = 0;
    b.iter(|| {
        let mut local_buffer = buffer;
        j += 1;
        if j > 7 {
            j = 0;
            i -= 1;
            if i < 9 {
                i = 15;
            }
        }
        local_buffer[i] ^= 1 << j;
        secded.decode(&mut local_buffer).unwrap();
    })
}

#[test]
fn codec() {
    let hamming = SecDed128::new(120);
    //    assert_eq!(hamming.code_size(), 7);
    let mut test_value = [0; 16];
    test_value[12] = 5;
    let mut buffer = test_value;
    hamming.encode(&mut buffer);
    buffer[2] ^= 1;
    hamming.decode(&mut buffer).unwrap();
    assert_eq!(&test_value[..15], &buffer[..15])
}
