extern crate byteorder;
use byteorder::ByteOrder;

trait Bitwise {
    fn count(&self) -> Self;
    fn parity(&self) -> Self;
}

impl Bitwise for u128 {
    fn count(&self) -> u128 {
        let mut count = 0;
        let mut copy = *self;
        while copy != 0 {
            count += copy & 1;
            copy >>= 1;
        }
        count
    }

    fn parity(&self) -> u128 {
        let mut result = *self;
        result ^= result >> 64;
        result ^= result >> 32;
        result ^= result >> 16;
        result ^= result >> 8;
        result ^= result >> 4;
        result ^= result >> 2;
        result ^= result >> 1;
        (result & 1)
    }
}

#[test]
fn test_count() {
    assert_eq!(1u128.count(), 1);
    assert_eq!(2u128.count(), 1);
    assert_eq!(3u128.count(), 2);
}

#[test]
fn test_parity() {
    assert_eq!(1u128.parity(), 1);
    assert_eq!(2u128.parity(), 1);
    assert_eq!(3u128.parity(), 0);
}

#[test]
fn test_count_parity() {
    for x in (0..100).map(|x| x * (core::u128::MAX / 100)) {
        assert_eq!(x.parity(), x.count() % 2);
    }
}


#[repr(C)]
pub struct SECDED {
    encodable_size: u8,
    m: u8,
    encode_matrix: [u128; 8],
    decode_matrix: [u128; 8],
    syndromes: [u16; 128],
}

impl SECDED {
    pub fn code_size(&self) -> usize {(self.m + 1) as usize}
    fn bin_matrix_product_paritied(matrix: &[u128], value: u128) -> u128 {
        let mut result = 0u128;
        for x in matrix.iter() {
            result ^= (*x & value).parity();
            result <<= 1;
        }
        result |= result.parity();
        result
    }

    pub fn new(encodable_size: usize) -> Self {
        if encodable_size > 120 {
            panic!("This implementation is based on u128, and can thus only encode payloads of at most 120 bits");
        }
        let mut m = 1;
        while 2_usize.pow(m as u32) - m < encodable_size as usize{
            m+=1;
        }
        let mut encode_matrix = [0; 8];
        for i in 1..(2_u128.pow(m as u32) + 1) {
            if i.count() < 2 {continue;}
            for k in 0..(m as usize) {
                encode_matrix[k] <<= 1;
                encode_matrix[k] |= i >> (m - 1 - k) & 1;
            }
        }
        let mut decode_matrix = [0; 8];
        for i in 0..8 {
            decode_matrix[i] = encode_matrix[i] << (m + 1);
            if i <= m {
                decode_matrix[i] |= 1 << (m - i);
            }
        }
        let mut syndromes = [0; 128];
        for error_bit in 0..(encodable_size + m) {
            let error: u128 = 1u128 << error_bit;
            syndromes[error_bit] = Self::bin_matrix_product_paritied(&decode_matrix[0..m], error) as u16;
        }
        for (i, x) in syndromes[..(encodable_size + m)].iter().enumerate() {
            for y in syndromes[i+1..(encodable_size + m)].iter() {
                assert_ne!(x, y);
            }
        }
        SECDED {
            encodable_size: encodable_size as u8,
            m: m as u8,
            encode_matrix,
            decode_matrix,
            syndromes,
        }
    }

    pub fn encode(&self, data: &mut [u8]) {
        let mut buffer = [0; 16];
        let start = 16-data.len();
        buffer[start..].clone_from_slice(data);
        let mut encodable = byteorder::BigEndian::read_u128(&buffer);
        // encodable.bin_println(8);
        if encodable > (1u128 << self.encodable_size) {
            panic!("{:?} is too big to be encoded on {} bits", buffer.as_ref(), self.encodable_size);
        }
        let code = Self::bin_matrix_product_paritied(&self.encode_matrix[..self.m as usize], encodable);
        encodable = encodable << self.code_size() | code;
        // encodable.bin_println(8);
        byteorder::BigEndian::write_u128(&mut buffer, encodable);
        data.clone_from_slice(&buffer[start..]);
    }

    pub fn decode(&self, data: &mut [u8]) -> Result<(),()> {
        let mut buffer = [0; 16];
        let start = 16-data.len();
        buffer[start..].clone_from_slice(data);
        let mut decodable = byteorder::BigEndian::read_u128(&buffer);
        let syndrome = Self::bin_matrix_product_paritied(&self.decode_matrix[..self.m as usize], decodable) as u16;
        if syndrome == 0 {
            decodable >>= self.code_size();
            byteorder::BigEndian::write_u128(&mut buffer, decodable);
            data.clone_from_slice(&buffer[start..]);
            return Ok(());
        }
        for (i, s) in self.syndromes.iter().enumerate() {
            if *s == syndrome {
                decodable = (decodable ^ (1 << i))>>self.code_size();
                byteorder::BigEndian::write_u128(&mut buffer, decodable);
                data.clone_from_slice(&buffer[start..]);
                return Ok(())
            }
        }
        Err(())
    }
}

#[test]
fn hamming_size() {
    assert_eq!(SECDED::new(57).code_size(), 7);
    assert_eq!(SECDED::new(64).code_size(), 8);
    assert_eq!(SECDED::new(120).code_size(), 8);
}

#[test]
fn hamming_both() {
    let hamming = SECDED::new(57);
    assert_eq!(hamming.code_size(), 7);
    let test_value = [0,0,0,0,0,0,0,5];
    let mut buffer = test_value;
    hamming.encode(&mut buffer);
    buffer[2] = 1;
    hamming.decode(&mut buffer).unwrap();
    assert_eq!(test_value, buffer)
}

#[cfg(feature = "ffi")]
#[allow(non_snake_case)]
mod ffi {
    use crate::SECDED;
    // }
    #[no_mangle]
    pub unsafe fn SECDED_new(encodable_size: usize) -> SECDED {
        crate::SECDED::new(encodable_size)
    }

    #[no_mangle]
    pub unsafe fn SECDED_encode(secded: *const SECDED, data: *mut u8, size: usize) {
        (*secded).encode(core::slice::from_raw_parts_mut(data, size));
    }

    #[no_mangle]
    pub unsafe fn SECDED_decode(secded: *const SECDED, data: *mut u8, size: usize) -> bool{
        match (*secded).decode(core::slice::from_raw_parts_mut(data, size)) {
            Err(()) => false,
            Ok(()) => {true}
        }
    }

    #[test]
    fn ffi_hamming_both() {
        unsafe {
            let secded = SECDED_new(57);
            let expected = [0,0,0,0,0,0,5];
            let mut buffer = [0u8; 8];
            buffer[1..].clone_from_slice(&expected);
            SECDED_encode(&secded, buffer.as_mut_ptr(), 8);
            assert!(SECDED_decode(&secded, buffer.as_mut_ptr(), 8));
            assert_eq!(expected, buffer[1..]);
        }
    }
}
