/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{SecDed128, SecDed64, SecDedCodec};

// }
#[no_mangle]
pub unsafe fn SECDED_64_new(encodable_size: usize) -> SecDed64 {
    crate::SecDed64::new(encodable_size)
}

#[no_mangle]
pub unsafe fn SECDED_64_encode(secded: *const SecDed64, data: *mut [u8; 8]) {
    (*secded).encode(&mut *data);
}

#[no_mangle]
pub unsafe fn SECDED_64_decode(secded: *const SecDed64, data: *mut [u8; 8]) -> bool {
    match (*secded).decode(&mut *data) {
        Err(()) => false,
        Ok(()) => true,
    }
}

#[no_mangle]
pub unsafe fn SECDED_128_new(encodable_size: usize) -> SecDed128 {
    crate::SecDed128::new(encodable_size)
}

#[no_mangle]
pub unsafe fn SECDED_128_encode(secded: *const SecDed128, data: *mut [u8; 16]) {
    (*secded).encode(&mut *data);
}

#[no_mangle]
pub unsafe fn SECDED_128_decode(secded: *const SecDed128, data: *mut [u8; 16]) -> bool {
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
        Box::into_raw(Box::new(crate::SecDedDynamic::new(encodable_size))) as *const SECDED_DYN
    }
    #[no_mangle]
    pub unsafe fn SECDED_DYN_free(secded: *const SECDED_DYN) {
        Box::from_raw(secded as *mut crate::SecDedDynamic);
    }

    #[no_mangle]
    pub unsafe fn SECDED_DYN_encode(secded: *const SECDED_DYN, data: *mut u8, size: usize) {
        let slice = std::slice::from_raw_parts_mut(data, size);
        (*(secded as *const crate::SecDedDynamic)).encode(slice);
    }

    #[no_mangle]
    pub unsafe fn SECDED_DYN_decode(secded: *const SECDED_DYN, data: *mut u8, size: usize) -> bool {
        let slice = std::slice::from_raw_parts_mut(data, size);
        match (*(secded as *const crate::SecDedDynamic)).decode(slice) {
            Err(()) => false,
            Ok(()) => true,
        }
    }
}
