/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(feature = "bench", feature(test))]
extern crate byteorder;
#[cfg(feature = "bench")]
extern crate test;
use byteorder::ByteOrder;

#[cfg(feature = "dyn")]
mod bitvec;
mod bitwise;
pub mod secded_64;
pub use secded_64::SecDed64;
pub mod secded_128;
#[cfg(feature = "dyn")]
use crate::secded_dynamic::SecDedDynamic;
pub use secded_128::SecDed128;

#[cfg(feature = "dyn")]
pub mod secded_dynamic;

fn hamming_size(encodable_size: usize) -> usize {
    let mut m = 1;
    while (1 << m) - m - 1 < encodable_size as usize {
        m += 1;
    }
    m
}

/// Your main interaction point with this crate, it allows you to encode and decode your data slices.
pub trait SecDedCodec {
    /// Returns the number of bits that this SecDedCodec can encode.
    fn encodable_size(&self) -> usize;

    /// Returns the size of the correction code that will be appended to the data.
    fn code_size(&self) -> usize;

    /// Returns `Some(size)` if the implementation would panic if `data.len() != size`
    fn expected_slice_size(&self) -> Option<usize> {
        None
    }

    /// Encodes the data IN-PLACE
    /// # Arguments:
    /// * `data`: The slice of data to encode. The last `secded.code_size()` bits MUST be set to 0.
    /// # Panics:
    /// Depending on the implementation, panics may occur if the size of the slice isn't adapted to the Codec:
    /// * SecDed64 panics if `data.len() != 8`  
    /// * SecDed128 panics if `data.len() != 16`  
    /// * You can use `secded.expected_slice_size()` to find out if a specific size is required for the slice.
    ///
    /// Unless you use the `no-panics` feature, encoding will also panic if the data you try to encode has some
    /// bits set to 1 in the reserved space, or past the `encodable_size() + code_size()` rightmost bits
    fn encode(&self, data: &mut [u8]);

    /// Decodes the data IN-PLACE
    /// # Arguments:
    /// * `data`: The slice of data to decode.  
    /// The last `secded.code_size()` bits will be reset to 0, a single error will be corrected implicitly.
    /// # Returns:
    /// `Ok(())` if the data slice's correctness has been checked: 0 error found or 1 found and corrected.
    /// `Err(())` if 2 errors were detected.
    /// # Panics:
    /// Depending on the implementation, panics may occur if the size of the slice isn't adapted to the Codec:
    /// * SecDed64 panics if `data.len() != 8`
    /// * SecDed128 panics if `data.len() != 16`
    /// * You can use `secded.expected_slice_size()` to find out if a specific size is required for the slice.
    fn decode(&self, data: &mut [u8]) -> Result<(), ()>;
}

pub enum SECDED {
    U64(SecDed64),
    U128(SecDed128),
    #[cfg(feature = "dyn")]
    DYNAMIC(SecDedDynamic),
}

impl SECDED {
    pub fn new(encodable_size: usize) -> Self {
        match encodable_size {
            0..=57 => SECDED::U64(SecDed64::new(encodable_size)),
            58..=120 => SECDED::U128(SecDed128::new(encodable_size)),
            #[cfg(feature = "dyn")]
            _ => SECDED::DYNAMIC(SecDedDynamic::new(encodable_size)),
            #[cfg(not(feature = "dyn"))]
            _ => panic!("{} bits not handled by this version of the crate, try on a platform that has u128 or \
            using features std and dyn", encodable_size)
        }
    }
}

#[cfg(feature = "ffi")]
#[allow(non_snake_case)]
mod ffi;
