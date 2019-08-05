use crate::bitwise::Bitwise;
use crate::*;
#[repr(C)]
pub struct SECDED_64 {
    encodable_size: u8,
    m: u8,
    mask: u8,
    encode_matrix: [u64; 7],
    syndromes: [u16; 64],
}

impl SECDED_64 {
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
        let mut encode_matrix = [0; 7];
        for i in 1..(2_u64.pow(m as u32) + 1) {
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
        SECDED_64 {
            encodable_size: encodable_size as u8,
            m: m as u8,
            mask: { 0xffu8 ^ (0..(m + 1)).map(|x| 1u8 << x).sum::<u8>() },
            encode_matrix,
            syndromes,
        }
    }
}

impl SecDedCodec<8> for SECDED_64 {
    fn encodable_size(&self) -> usize {
        self.encodable_size as usize
    }
    fn code_size(&self) -> usize {
        (self.m + 1) as usize
    }

    fn encode(&self, buffer: &mut [u8; 8]) {
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

    fn decode(&self, buffer: &mut [u8; 8]) -> Result<(), ()> {
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
