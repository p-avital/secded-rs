#![cfg_attr(feature = "bench", feature(test))]
extern crate byteorder;
#[cfg(feature = "bench")]
extern crate test;
use byteorder::ByteOrder;

#[cfg(feature = "dyn")]
mod bitvec;
mod bitwise;
pub mod secded_64;
pub use secded_64::Secded64;
pub mod secded_128;
#[cfg(feature = "dyn")]
use crate::secded_dynamic::SecdedDynamic;
pub use secded_128::Secded128;

#[cfg(feature = "dyn")]
pub mod secded_dynamic;

pub trait SecDedCodec {
    fn encodable_size(&self) -> usize;
    fn code_size(&self) -> usize;
    fn encode(&self, data: &mut [u8]);
    fn decode(&self, data: &mut [u8]) -> Result<(), ()>;
}

pub enum SECDED {
    U64(Secded64),
    U128(Secded128),
    #[cfg(feature = "dyn")]
    DYNAMIC(SecdedDynamic),
}

impl SECDED {
    pub fn new(encodable_size: usize) -> Self {
        match encodable_size {
            0..=57 => SECDED::U64(Secded64::new(encodable_size)),
            58..=120 => SECDED::U128(Secded128::new(encodable_size)),
            #[cfg(feature = "dyn")]
            _ => SECDED::DYNAMIC(SecdedDynamic::new(encodable_size)),
            #[cfg(not(feature = "dyn"))]
            _ => panic!("{} bits not handled by this version of the crate, try on a platform that has u128 or \
            using features std and dyn", encodable_size)
        }
    }
}

#[cfg(feature = "ffi")]
#[allow(non_snake_case)]
mod ffi;
