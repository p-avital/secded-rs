#![feature(test)]
#![feature(const_generics)]
extern crate byteorder;
extern crate test;
use byteorder::ByteOrder;

mod bitvec;
mod bitwise;
mod secded_64;
pub use secded_64::SECDED_64;
mod secded_128;
pub use secded_128::SECDED_128;
mod secded_dynamic;

pub trait SecDedCodec<const buffer_size: usize> {
    fn encodable_size(&self) -> usize;
    fn code_size(&self) -> usize;
    fn encode(&self, data: &mut [u8; buffer_size]);
    fn decode(&self, data: &mut [u8; buffer_size]) -> Result<(), ()>;
}

pub enum SECDED {
    U64(SECDED_64),
    U128(SECDED_128),
}

impl SECDED {
    pub fn new(encodable_size: usize) -> Self {
        match encodable_size {
            0..=57 => SECDED::U64(SECDED_64::new(encodable_size)),
            58..=120 => SECDED::U128(SECDED_128::new(encodable_size)),
            _ => panic!(
                "No implementation available yet for encodable_size {}",
                encodable_size
            ),
        }
    }
}

impl SecDedCodec<8> for SECDED {
    fn encodable_size(&self) -> usize {
        match self {
            SECDED::U64(s) => s.encodable_size(),
            SECDED::U128(s) => panic!(),
        }
    }
    fn code_size(&self) -> usize {
        match self {
            SECDED::U64(s) => s.code_size(),
            SECDED::U128(s) => panic!(),
        }
    }
    fn encode(&self, data: &mut [u8; 8]) {
        match self {
            SECDED::U64(s) => s.encode(data),
            SECDED::U128(s) => panic!(),
        }
    }
    fn decode(&self, data: &mut [u8; 8]) -> Result<(), ()> {
        match self {
            SECDED::U64(s) => s.decode(data),
            SECDED::U128(s) => panic!(),
        }
    }
}

impl SecDedCodec<16> for SECDED {
    fn encodable_size(&self) -> usize {
        match self {
            SECDED::U64(s) => panic!(),
            SECDED::U128(s) => s.encodable_size(),
        }
    }
    fn code_size(&self) -> usize {
        match self {
            SECDED::U64(s) => panic!(),
            SECDED::U128(s) => s.code_size(),
        }
    }
    fn encode(&self, data: &mut [u8; 16]) {
        match self {
            SECDED::U64(s) => panic!(),
            SECDED::U128(s) => s.encode(data),
        }
    }
    fn decode(&self, data: &mut [u8; 16]) -> Result<(), ()> {
        match self {
            SECDED::U64(s) => panic!(),
            SECDED::U128(s) => s.decode(data),
        }
    }
}

#[test]
fn hamming_size() {
    //    assert_eq!(<SECDED as SecDedCodec<8>>::code_size(SECDED::new(57)), 7);
    //    assert_eq!(<SECDED as SecDedCodec<16>>::code_size(SECDED::new(64)), 8);
    //    assert_eq!(<SECDED as SecDedCodec<16>>::code_size(SECDED::new(120)), 8);
}

#[test]
fn hamming_both() {
    let hamming = SECDED::new(57);
    //    assert_eq!(hamming.code_size(), 7);
    let test_value = [0, 0, 0, 0, 5, 0, 0, 0];
    let mut buffer = test_value;
    hamming.encode(&mut buffer);
    buffer[2] ^= 1;
    hamming.decode(dbg!(&mut buffer)).unwrap();
    assert_eq!(&test_value[..7], dbg!(&buffer[..7]))
}

#[cfg(feature = "ffi")]
#[allow(non_snake_case)]
mod ffi {
    use crate::{SecDedCodec, SECDED_128, SECDED_64};
    // }
    #[no_mangle]
    pub unsafe fn SECDED_64_new(encodable_size: usize) -> SECDED_64 {
        crate::SECDED_64::new(encodable_size)
    }

    #[no_mangle]
    pub unsafe fn SECDED_64_encode(secded: *const SECDED_64, data: *mut [u8; 8]) {
        (*secded).encode(&mut *data);
    }

    #[no_mangle]
    pub unsafe fn SECDED_64_decode(secded: *const SECDED_64, data: *mut [u8; 8]) -> bool {
        match (*secded).decode(&mut *data) {
            Err(()) => false,
            Ok(()) => true,
        }
    }

    #[no_mangle]
    pub unsafe fn SECDED_128_new(encodable_size: usize) -> SECDED_128 {
        crate::SECDED_128::new(encodable_size)
    }

    #[no_mangle]
    pub unsafe fn SECDED_128_encode(secded: *const SECDED_128, data: *mut [u8; 16]) {
        (*secded).encode(&mut *data);
    }

    #[no_mangle]
    pub unsafe fn SECDED_128_decode(secded: *const SECDED_128, data: *mut [u8; 16]) -> bool {
        match (*secded).decode(&mut *data) {
            Err(()) => false,
            Ok(()) => true,
        }
    }

    #[test]
    fn ffi_hamming_both() {
        unsafe {
            let secded = SECDED_64_new(57);
            let expected = [0, 0, 0, 0, 5, 0, 0];
            let mut buffer = [0u8; 8];
            buffer[..7].clone_from_slice(&expected);
            SECDED_64_encode(&secded, buffer.as_mut_ptr() as *mut [u8; 8]);
            assert!(SECDED_64_decode(
                &secded,
                buffer.as_mut_ptr() as *mut [u8; 8]
            ));
            assert_eq!(expected, buffer[..7]);
        }
    }
}
#[bench]
fn secded_64_encode_bench(b: &mut test::Bencher) {
    let secded = SECDED_64::new(57);
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
#[bench]
fn secded_64_decode_bench(b: &mut test::Bencher) {
    let secded = SECDED_64::new(57);
    let expected = [0, 0, 0, 0, 5, 0, 0];
    let mut buffer = [0u8; 8];
    buffer[1..].clone_from_slice(&expected);
    secded.encode(&mut buffer);
    b.iter(|| {
        let mut local_buffer = buffer;
        secded.decode(&mut local_buffer).unwrap();
    })
}
#[bench]
fn secded_64_decode_1err_bench(b: &mut test::Bencher) {
    let secded = SECDED_64::new(57);
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

#[bench]
fn secded_128_encode_bench(b: &mut test::Bencher) {
    let secded = SECDED_128::new(57);
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

#[bench]
fn secded_128_decode_bench(b: &mut test::Bencher) {
    let secded = SECDED_128::new(57);
    let mut buffer = [0u8; 16];
    buffer[13] = 5;
    secded.encode(&mut buffer);
    b.iter(|| {
        let mut local_buffer = buffer;
        secded.decode(&mut local_buffer).unwrap();
    })
}

#[bench]
fn secded_128_decode_1err_bench(b: &mut test::Bencher) {
    let secded = SECDED_128::new(57);
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
