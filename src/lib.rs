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
    for x in (0..100).map(|x| x * (std::u128::MAX / 100)) {
        assert_eq!(x.parity(), x.count() % 2);
    }
}


trait BinPrintable {
    fn bin_print(&self, min_chars: isize);
    fn bin_println(&self, min_chars: isize) {
        self.bin_print(min_chars);
        println!(); 
    }
}
impl BinPrintable for u128 {
    fn bin_print(&self, min_chars: isize){
        let mut do_print = false;
        for i in (0..128).rev() {
            let bit = (self >> i) as u32 & 1;
            if !do_print {
                if bit == 1 || i < min_chars {
                    do_print = true;
                }
            }
            if do_print {
                print!("{}", bit);
            }
        }
    }
}
impl BinPrintable for &[u8] {
    fn bin_print(&self, mut min_chars: isize){
        let mut do_print = false;
        for byte in self.iter() {
            for i in (0..8).rev() {
                let bit = (byte >> i) as u32 & 1;
                if !do_print {
                    if bit == 1 || min_chars <= 0 {
                        do_print = true;
                    }
                }
                if do_print {
                    print!("{}", bit);
                }
                min_chars -= 1;
            }
        }
    }
}

pub struct SECDED {
    encodable_size: u8,
    code_size: u8,
    encode_matrix: Vec<u128>,
    decode_matrix: Vec<u128>,
    syndromes: Vec<u16>,
}

impl SECDED {
    pub fn code_size(&self) -> usize {self.code_size as usize}
    fn bin_matrix_product_paritied(matrix: &Vec<u128>, value: u128) -> u128 {
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
        let mut encode_matrix = vec![];
        for _ in 0..m {encode_matrix.push(0);}
        for i in 1..(2_u128.pow(m as u32) + 1) {
            if i.count() < 2 {continue;}
            for k in 0..(m as usize) {
                encode_matrix[k] <<= 1;
                encode_matrix[k] |= i >> (m - 1 - k) & 1;
            }
        }
        let decode_matrix = encode_matrix.iter().enumerate().map(|(i, x)| (x<<m + 1) | (1 << m - i)).collect::<Vec<_>>();
        let mut syndromes = vec![];
        for error_bit in 0..(encodable_size + m) {
            let error: u128 = 1u128 << error_bit;
            syndromes.push(Self::bin_matrix_product_paritied(&decode_matrix, error) as u16);
        }
        // for (i, x) in syndromes.iter().enumerate() {
            // println!("SYNDROME[{}] = {}", i, x);
        // }
        for (i, x) in syndromes.iter().enumerate() {
            // println!("SYNDROME[{}] = {}", i, x);
            for y in syndromes[i+1..].iter() {
                assert_ne!(x, y);
            }
        }
        SECDED {
            encodable_size: encodable_size as u8,
            code_size: m as u8 + 1,
            encode_matrix,
            decode_matrix,
            syndromes,
        }
    }

    fn u128_to_le(&self, value: u128) -> Vec<u8> {
        let max_size = self.code_size + self.encodable_size;
        let mut result = Vec::with_capacity(max_size as usize);
        let mut cursor: i32 = 120;
        while cursor >= 0 {
            if cursor < max_size as i32 {
                result.push(value.overflowing_shr(cursor as u32).0 as u8);
            }
            cursor -= 8;
        }
        result
    }

    fn le_to_u128<T: AsRef<[u8]>>(&self, data: &T) -> u128 {
        let mut result = 0;
        for (i, &byte) in data.as_ref().iter().rev().enumerate() {
            result |= (byte as u128) << (8 * i); 
        }
        result
    }

    pub fn encode<T: AsRef<[u8]>>(&self, data: &T) -> Vec<u8> {
        // data.as_ref().bin_println(8);
        let mut encodable = self.le_to_u128(data);
        // encodable.bin_println(8);
        if encodable > 1u128 << self.encodable_size {
            panic!("{:?} is too big to be encoded on {} bits", data.as_ref(), self.encodable_size);
        }
        let code = Self::bin_matrix_product_paritied(&self.encode_matrix, encodable);
        encodable = encodable << self.code_size() | code;
        // encodable.bin_println(8);
        let result = self.u128_to_le(encodable);
        result
    }

    pub fn decode<T: AsRef<[u8]>>(&self, data: &T) -> Option<Vec<u8>> {
        let mut decodable = self.le_to_u128(data);
        // decodable.bin_println(8);
        let syndrome = Self::bin_matrix_product_paritied(&self.decode_matrix, decodable) as u16;
        if dbg!(syndrome) == 0 {
            decodable >>= self.code_size();
            return Some(self.u128_to_le(decodable));
        }
        for (i, s) in self.syndromes.iter().enumerate() {
            if *s == syndrome {
                return Some(self.u128_to_le((decodable ^ (1 << dbg!(i)))>>self.code_size()));
            }
        }
        None
    }
}

#[test]
fn u128_encoding() {
    let hamming = SECDED::new(57);
    // dbg!(hamming.le_to_u128([1, 2]));
    // dbg!(hamming.u128_to_le(258));
    for i in (0..1000000).step_by(100) {
        assert_eq!(hamming.le_to_u128(&hamming.u128_to_le(i)), i)
    }
}

#[test]
fn hamming_size() {
    assert_eq!(SECDED::new(57).code_size(), 7);
    assert_eq!(SECDED::new(64).code_size(), 8);
    assert_eq!(SECDED::new(120).code_size(), 8);
}
#[test]
#[should_panic]
fn hamming_oversize() {
    SECDED::new(121);
}

#[test]
fn hamming_both() {
    let hamming = SECDED::new(57);
    assert_eq!(hamming.code_size(), 7);
    let test_value = vec![0,0,0,0,0,0,0,5];
    let mut encoded = hamming.encode(&test_value);
    encoded[2] = 1;
    let decoded = hamming.decode(&encoded).unwrap();
    <Vec<u8> as AsRef<[u8]>>::as_ref(&decoded).bin_println(8);
    assert_eq!(test_value, decoded)
}

#[cfg(feature = "ffi")]
#[allow(non_snake_case)]
mod ffi {
    // mod ffi {
        #[repr(C)]
        pub struct SECDED { _private: [u8; 0] }
    // }
    #[no_mangle]
    pub unsafe fn SECDED_new(encodable_size: usize) -> *mut SECDED {
        let safe = std::boxed::Box::new(crate::SECDED::new(encodable_size));
        Box::into_raw(safe) as *mut SECDED
    }

    #[no_mangle]
    pub unsafe fn SECDED_free(secded: *mut SECDED) {
        Box::from_raw(secded as *mut crate::SECDED);
    }

    #[no_mangle]
    pub unsafe fn SECDED_encode(secded: *mut SECDED, data: *const u8, size: usize, out_buffer: *mut u8) {
        let secded = &*(secded as *mut crate::SECDED);
        let result = secded.encode(&std::slice::from_raw_parts(data, size));
        std::ptr::copy_nonoverlapping(result.as_ptr(), out_buffer, result.len());
    }

    #[no_mangle]
    pub unsafe fn SECDED_decode(secded: *mut SECDED, data: *const u8, size: usize, out_buffer: *mut u8) -> bool{
        let secded = &*(secded as *mut crate::SECDED);
        match secded.decode(&std::slice::from_raw_parts(data, size)) {
            None => false,
            Some(result) => {
                std::ptr::copy_nonoverlapping(result.as_ptr(), out_buffer, result.len());
                true
            }
        }
    }

    #[test]
    fn ffi_hamming_both() {
        unsafe {
            let secded = SECDED_new(57);
            let expected = [0,0,0,0,0,0,0,5];
            let mut buffer = [0u8; 8];
            SECDED_encode(secded, expected.as_ptr(), 8, buffer.as_mut_ptr());
            assert!(SECDED_decode(secded, buffer.as_ptr(), 8, buffer.as_mut_ptr()));
            assert_eq!(expected, buffer);
            SECDED_free(secded);
        }
    }
}
