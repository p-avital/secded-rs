use crate::{SecDedCodec, Secded128, Secded64};

// }
#[no_mangle]
pub unsafe fn SECDED_64_new(encodable_size: usize) -> Secded64 {
    crate::Secded64::new(encodable_size)
}

#[no_mangle]
pub unsafe fn SECDED_64_encode(secded: *const Secded64, data: *mut [u8; 8]) {
    (*secded).encode(&mut *data);
}

#[no_mangle]
pub unsafe fn SECDED_64_decode(secded: *const Secded64, data: *mut [u8; 8]) -> bool {
    match (*secded).decode(&mut *data) {
        Err(()) => false,
        Ok(()) => true,
    }
}

#[no_mangle]
pub unsafe fn SECDED_128_new(encodable_size: usize) -> Secded128 {
    crate::Secded128::new(encodable_size)
}

#[no_mangle]
pub unsafe fn SECDED_128_encode(secded: *const Secded128, data: *mut [u8; 16]) {
    (*secded).encode(&mut *data);
}

#[no_mangle]
pub unsafe fn SECDED_128_decode(secded: *const Secded128, data: *mut [u8; 16]) -> bool {
    match (*secded).decode(&mut *data) {
        Err(()) => false,
        Ok(()) => true,
    }
}

#[test]
fn hamming_both() {
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

#[cfg(feature = "dyn")]
mod dynamic {
    use crate::SecDedCodec;
    #[repr(C)]
    pub struct SECDED_DYN {}

    #[no_mangle]
    pub unsafe fn SECDED_DYN_new(encodable_size: usize) -> *const SECDED_DYN {
        Box::into_raw(Box::new(crate::SecdedDynamic::new(encodable_size))) as *const SECDED_DYN
    }
    #[no_mangle]
    pub unsafe fn SECDED_DYN_free(secded: *const SECDED_DYN) {
        Box::from_raw(secded as *mut crate::SecdedDynamic);
    }

    #[no_mangle]
    pub unsafe fn SECDED_DYN_encode(secded: *const SECDED_DYN, data: *mut u8, size: usize) {
        let slice = std::slice::from_raw_parts_mut(data, size);
        (*(secded as *const crate::SecdedDynamic)).encode(slice);
    }

    #[no_mangle]
    pub unsafe fn SECDED_DYN_decode(secded: *const SECDED_DYN, data: *mut u8, size: usize) -> bool {
        let slice = std::slice::from_raw_parts_mut(data, size);
        match (*(secded as *const crate::SecdedDynamic)).decode(slice) {
            Err(()) => false,
            Ok(()) => true,
        }
    }
}
