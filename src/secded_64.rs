use crate::bitwise::Bitwise;
use crate::*;
#[repr(C)]
pub struct Secded64 {
    encodable_size: u8,
    m: u8,
    mask: u8,
    encode_matrix: [u64; 6],
    syndromes: [u16; 64],
}

impl Secded64 {
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
        let mut m = 1;
        while 2_usize.pow(m as u32) - m < encodable_size as usize {
            m += 1;
        }
        let mut encode_matrix = [0; 6];
        for i in 1..=(2_u64.pow(m as u32)) {
            if i.count() < 2 {
                continue;
            }
            for (k, x) in encode_matrix.iter_mut().enumerate() {
                *x <<= 1;
                *x |= i >> (m - 1 - k) & 1;
            }
        }
        for (i, x) in encode_matrix.iter_mut().enumerate() {
            *x <<= m + 1;
            if i <= m {
                *x |= 1 << (m - i);
            }
        }
        let mut syndromes = [0; 64];
        for error_bit in 0..(encodable_size + m) {
            let error: u64 = 1u64 << error_bit;
            syndromes[error_bit] =
                Self::bin_matrix_product_paritied(&encode_matrix[0..m], error) as u16;
        }
        for (i, x) in syndromes[..(encodable_size + m)].iter().enumerate() {
            for y in syndromes[i + 1..(encodable_size + m)].iter() {
                assert_ne!(x, y);
            }
        }
        Secded64 {
            encodable_size: encodable_size as u8,
            m: m as u8,
            mask: { 0xffu8 ^ (0..(m + 1)).map(|x| 1u8 << x).sum::<u8>() },
            encode_matrix,
            syndromes,
        }
    }
}

impl SecDedCodec for Secded64 {
    fn encodable_size(&self) -> usize {
        self.encodable_size as usize
    }
    fn code_size(&self) -> usize {
        (self.m + 1) as usize
    }

    fn encode(&self, buffer: &mut [u8]) {
        let mut encodable = byteorder::BigEndian::read_u64(&buffer[..]);
        match 1u64.overflowing_shl((self.encodable_size + self.m + 1) as u32) {
            (value, false) if encodable > value => {
                panic!(
                    "{:?} is too big to be encoded on {} bits",
                    buffer.as_ref(),
                    self.encodable_size
                );
            }
            _ => {}
        };
        let code =
            Self::bin_matrix_product_paritied(&self.encode_matrix[..self.m as usize], encodable);
        encodable |= code;
        byteorder::BigEndian::write_u64(&mut buffer[..], encodable);
    }

    fn decode(&self, buffer: &mut [u8]) -> Result<(), ()> {
        let mut decodable = byteorder::BigEndian::read_u64(&buffer[..]);
        let syndrome =
            Self::bin_matrix_product_paritied(&self.encode_matrix[..self.m as usize], decodable)
                as u16;
        if syndrome == 0 {
            return Ok(());
        }
        for (i, s) in self.syndromes.iter().enumerate() {
            if *s == syndrome {
                decodable ^= 1 << i;
                byteorder::BigEndian::write_u64(&mut buffer[..], decodable);
                return Ok(());
            }
        }
        Err(())
    }
}

#[cfg(feature = "bench")]
#[bench]
fn encode_bench(b: &mut test::Bencher) {
    let secded = Secded64::new(57);
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
    let secded = Secded64::new(57);
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
    let secded = Secded64::new(57);
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


#[test]
fn codec() {
    let hamming = Secded64::new(57);
    //    assert_eq!(hamming.code_size(), 7);
    let test_value = [0, 0, 0, 0, 5, 0, 0, 0];
    let mut buffer = test_value;
    hamming.encode(&mut buffer);
    buffer[2] ^= 1;
    hamming.decode(&mut buffer).unwrap();
    assert_eq!(&test_value[..7], &buffer[..7])
}