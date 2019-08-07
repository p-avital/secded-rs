use crate::bitwise::Bitwise;
use std::collections::VecDeque;
#[derive(Clone)]
pub struct Bitvec(pub VecDeque<u8>);

#[macro_export]
macro_rules! bitvec {
    [$($e:expr), *] => {
        Bitvec({
            #[allow(unused_mut)]
            let mut v = VecDeque::new();
            $(
                v.push_back($e);
            )*
            v
        })
    };
    ($($e:expr), *) => {bitvec![$($e)*]};
}

impl From<&[u8]> for Bitvec {
    fn from(data: &[u8]) -> Self {
        use std::iter::FromIterator;
        Bitvec(VecDeque::from_iter(
            data[data
                .iter()
                .position(|x| *x != 0)
                .unwrap_or_else(|| data.len())..]
                .iter()
                .copied(),
        ))
    }
}

#[test]
fn from() {
    let b: Bitvec = [1, 2, 3].as_ref().into();
    assert_eq!(bitvec![1, 2, 3], b);
}

//        pub static ref BITVEC_ONE: Bitvec = bitvec!(1);

impl Bitvec {
    pub fn nth_bit_from_right(&self, n: usize) -> u8 {
        let (mut byte_shift, bit_shift) = (n / 8, (n % 8) as u32);
        let mut iter = self.0.iter();
        while let Some(byte) = iter.next_back() {
            if byte_shift == 0 {
                return 1 & (*byte >> bit_shift);
            }
            byte_shift -= 1
        }
        panic!(
            "Requested {}th bit from the right of a {} bytes long Bitvec",
            n,
            self.0.len()
        )
    }

    pub fn mask_not(&mut self, rhs: &Self) {
        for (s, r) in self.0.iter_mut().rev().zip(rhs.0.iter().rev()) {
            *s &= !*r;
        }
    }

    pub fn mask_not_buffer(&self, rhs: &mut [u8]) {
        for (s, r) in rhs.iter_mut().rev().zip(self.0.iter().rev()) {
            *s &= !*r;
        }
    }

    pub fn is_null(&self) -> bool {
        self.0.iter().all(|x| *x == 0)
    }

    #[allow(dead_code)]
    pub(crate) fn to_u64_be(&self) -> u64 {
        let mut result = 0;
        for x in self.0.iter() {
            result = (result << 8) | (*x as u64);
        }
        result
    }

    #[allow(dead_code)]
    fn from_u64_be(mut val: u64) -> Self {
        let mut vec = VecDeque::with_capacity(8);
        while val > 0 {
            vec.push_front(val as u8);
            val >>= 8;
        }
        Bitvec(vec)
    }
}
#[test]
fn mask_not() {
    let mut a = bitvec!(0xff, 0xff);
    a.mask_not(&bitvec!(0x0f));
    assert_eq!(a, bitvec!(0xff, 0xf0));
}

impl Bitwise for Bitvec {
    type Output = usize;

    fn count(&self) -> Self::Output {
        self.0.count()
    }

    fn parity(&self) -> Self::Output {
        self.0.parity()
    }
}

impl std::fmt::Binary for Bitvec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let mut iter = self.0.iter();
        if let Some(byte) = iter.next() {
            write!(f, "{:08b}", byte)?;
        }
        for byte in iter {
            write!(f, "_{:08b}", byte)?
        }
        Ok(())
    }
}
impl std::fmt::Debug for Bitvec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Bitvec({:b})", self)
    }
}

impl std::cmp::PartialEq for Bitvec {
    fn eq(&self, other: &Self) -> bool {
        let (longest, shortest) = if self.0.len() > other.0.len() {
            (&self.0, &other.0)
        } else {
            (&other.0, &self.0)
        };
        let (longest, mut shortest) = (longest.iter().rev(), shortest.iter().rev());
        for l in longest {
            if let Some(s) = shortest.next() {
                if s != l {
                    return false;
                }
            } else if *l != 0 {
                return false;
            }
        }
        true
    }
}
impl std::cmp::Eq for Bitvec {}

use std::hash::Hasher;

impl std::hash::Hash for Bitvec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut in_zero_padding = true;
        for &x in self.0.iter() {
            if x > 0 {
                in_zero_padding = false;
            }
            if !in_zero_padding {
                state.write_u8(x);
            }
        }
    }
}

#[test]
fn hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut vec = bitvec!(1);
    for i in 0..64 {
        let mut hasher = DefaultHasher::new();
        vec.hash(&mut hasher);
        let hash = hasher.finish();
        let mut vec2 = vec.clone() << 1;
        for _ in (i + 1)..64 {
            let mut hasher = DefaultHasher::new();
            vec2.hash(&mut hasher);
            assert_ne!(hash, hasher.finish());
            vec2 <<= 1;
        }
        vec <<= 1;
    }
}

impl std::cmp::PartialOrd for Bitvec {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for Bitvec {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let mut self_it = self.0.iter();
        let mut other_it = other.0.iter();
        if self.0.len() > other.0.len() {
            for _ in 0..(self.0.len() - other.0.len()) {
                if *self_it.next().unwrap() != 0 {
                    return std::cmp::Ordering::Greater;
                }
            }
        } else if other.0.len() > self.0.len() {
            for _ in 0..(other.0.len() - self.0.len()) {
                if *other_it.next().unwrap() != 0 {
                    return std::cmp::Ordering::Less;
                }
            }
        }
        for (s, o) in self_it.zip(other_it) {
            match s.cmp(o) {
                std::cmp::Ordering::Equal => {}
                r => {
                    return r;
                }
            }
        }
        std::cmp::Ordering::Equal
    }
}

impl std::ops::BitXor for &Bitvec {
    type Output = Bitvec;

    fn bitxor(self, other: Self) -> Self::Output {
        let (longest, shortest) = if self.0.len() > other.0.len() {
            (&self.0, &other.0)
        } else {
            (&other.0, &self.0)
        };
        let longest = longest.iter().rev();
        let mut shortest = shortest.iter().rev();
        let iter = longest.map(|l| {
            if let Some(s) = shortest.next() {
                l ^ s
            } else {
                *l
            }
        });
        let collected: Vec<u8> = iter.collect();
        Bitvec(collected.into_iter().rev().collect())
    }
}

impl std::ops::BitXorAssign<&Bitvec> for Bitvec {
    fn bitxor_assign(&mut self, other: &Bitvec) {
        if self.0.len() < other.0.len() {
            *self = &*self ^ other;
        } else {
            for (l, s) in self.0.iter_mut().rev().zip(other.0.iter().rev()) {
                *l ^= s;
            }
        }
    }
}

impl std::ops::AddAssign<&Self> for Bitvec {
    fn add_assign(&mut self, rhs: &Self) {
        while self.0.len() < rhs.0.len() {
            self.0.push_front(0);
        }
        let mut remainder = 0u16;
        let mut r = rhs.0.iter();
        for s in self.0.iter_mut().rev() {
            remainder += *s as u16;
            if let Some(&r) = r.next() {
                remainder += r as u16;
            }
            *s = remainder as u8;
            remainder >>= 8;
        }
        while remainder > 0 {
            self.0.push_front(remainder as u8);
            remainder >>= 8;
        }
    }
}

impl std::ops::AddAssign<u32> for Bitvec {
    #[allow(clippy::cast_lossless)]
    fn add_assign(&mut self, rhs: u32) {
        let mut remainder = rhs;
        for s in self.0.iter_mut().rev() {
            remainder += <u32 as From<u8>>::from(*s);
            *s = remainder as u8;
            remainder >>= 8;
        }
        while remainder > 0 {
            self.0.push_front(remainder as u8);
            remainder = remainder.overflowing_shr(8).0;
        }
    }
}

impl std::ops::Add for &Bitvec {
    type Output = Bitvec;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result += rhs;
        result
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl std::ops::ShrAssign<usize> for Bitvec {
    fn shr_assign(&mut self, rhs: usize) {
        let (byte_shift, bit_shift) = (rhs / 8, (rhs % 8) as u32);
        let bit_lshift = 8 - bit_shift;
        for i in (0..self.0.len()).rev() {
            if bit_shift == 0 {
                if let Some(&value) = self.0.get(i.overflowing_sub(byte_shift).0) {
                    self.0[i] = value;
                } else {
                    self.0[i] = 0;
                }
            } else {
                if let Some(&value) = self.0.get(i.overflowing_sub(byte_shift).0) {
                    self.0[i] = value >> bit_shift;
                } else {
                    self.0[i] = 0;
                }
                if let Some(&value) = self.0.get(i.overflowing_sub(byte_shift + 1).0) {
                    self.0[i] |= value << bit_lshift;
                }
            }
        }
    }
}

impl std::ops::Shr<usize> for Bitvec {
    type Output = Bitvec;

    fn shr(mut self, rhs: usize) -> Self::Output {
        self >>= rhs;
        self
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl std::ops::ShlAssign<usize> for Bitvec {
    fn shl_assign(&mut self, rhs: usize) {
        let (byte_shift, bit_shift) = (rhs / 8, (rhs % 8) as u32);
        if self.0.is_empty() {
            self.0.push_front(0)
        }
        if self.0[0].count() > (self.0[0] << bit_shift).count() {
            self.0.push_front(0)
        }
        for _ in 0..byte_shift {
            self.0.push_front(0);
        }
        let bit_lshift = 8 - bit_shift;
        for i in 0..self.0.len() {
            if bit_shift == 0 {
                if let Some(&value) = self.0.get(i + byte_shift) {
                    self.0[i] = value;
                } else {
                    self.0[i] = 0;
                }
            } else {
                if let Some(&value) = self.0.get(i + byte_shift) {
                    self.0[i] = value << bit_shift;
                } else {
                    self.0[i] = 0;
                }
                if let Some(&value) = self.0.get(i + byte_shift + 1) {
                    self.0[i] |= value >> bit_lshift;
                }
            }
        }
    }
}

impl std::ops::Shl<usize> for Bitvec {
    type Output = Bitvec;

    fn shl(mut self, rhs: usize) -> Self::Output {
        self <<= rhs;
        self
    }
}

impl std::ops::BitOrAssign<&Bitvec> for Bitvec {
    fn bitor_assign(&mut self, rhs: &Bitvec) {
        while self.0.len() < rhs.0.len() {
            self.0.push_front(0);
        }
        let mut rhs = rhs.0.iter().rev();
        for x in self.0.iter_mut().rev() {
            if let Some(&r) = rhs.next() {
                *x |= r;
            }
        }
    }
}

impl std::ops::BitOr for &Bitvec {
    type Output = Bitvec;

    fn bitor(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result |= rhs;
        result
    }
}

impl std::ops::BitOrAssign<u8> for Bitvec {
    fn bitor_assign(&mut self, rhs: u8) {
        if let Some(s) = self.0.iter_mut().last() {
            *s |= rhs;
        }
    }
}

impl std::ops::BitAnd for &Bitvec {
    type Output = Bitvec;

    fn bitand(self, rhs: Self) -> Self::Output {
        let mut s = self.0.iter();
        let mut r = rhs.0.iter();
        if self.0.len() > rhs.0.len() {
            for _ in 0..(self.0.len() - rhs.0.len()) {
                s.next();
            }
        }
        if rhs.0.len() > self.0.len() {
            for _ in 0..(rhs.0.len() - self.0.len()) {
                r.next();
            }
        }
        Bitvec(s.zip(r).map(|(l, r)| l & r).collect())
    }
}

impl std::ops::BitAnd<u8> for &Bitvec {
    type Output = u8;

    fn bitand(self, rhs: u8) -> Self::Output {
        if let Some(s) = self.0.iter().last() {
            rhs & s
        } else {
            0
        }
    }
}

#[test]
fn and_u8() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..100000 {
        let (x, y): (u64, u8) = (rng.gen(), rng.gen_range(0, 64));
        let expected = x as u8 & y;
        let mut x = Bitvec::from_u64_be(x);
        let x = &x & y;
        match expected == x {
            true => {}
            false => {
                dbg!(expected);
                dbg!(x);
                panic!()
            }
        }
    }
}

#[test]
fn and() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..100000 {
        let (x, y): (u64, u64) = (rng.gen(), rng.gen_range(0, 64));
        let expected = x & y;
        let mut x = Bitvec::from_u64_be(x);
        let x = &x & &Bitvec::from_u64_be(y);
        match expected == x.to_u64_be() {
            true => {}
            false => {
                dbg!(expected);
                dbg!(x);
                panic!()
            }
        }
    }
}

#[test]
fn add() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..100000 {
        let (x, y): (u64, u32) = (rng.gen(), rng.gen_range(0, 64));
        let expected = x + y as u64;
        let mut x = Bitvec::from_u64_be(x);
        x += y;
        match expected == x.to_u64_be() {
            true => {}
            false => {
                dbg!(Bitvec::from_u64_be(expected));
                dbg!(x);
                panic!()
            }
        }
    }
}

#[cfg(feature = "bench")]
#[bench]
fn add_bench(b: &mut test::Bencher) {
    let mut a = bitvec!();
    b.iter(|| {
        a += 1;
    })
}

#[cfg(feature = "bench")]
#[bench]
fn add_usize(b: &mut test::Bencher) {
    let mut a = 0;
    b.iter(|| {
        a += 1;
    })
}

#[cfg(feature = "bench")]
#[bench]
fn cmp_bench(bench: &mut test::Bencher) {
    let mut a = bitvec!();
    let b = bitvec!(5, 0);
    let mut result = false;
    bench.iter(|| {
        a += 1;
        result = a < b;
    })
}

#[cfg(feature = "bench")]
#[bench]
fn cmp_usize(bench: &mut test::Bencher) {
    let mut a = 0;
    let b = 0x500;
    let mut result = false;
    bench.iter(|| {
        a += 1;
        result = a < b;
    })
}

#[test]
fn shr() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..100000 {
        let (x, y): (u64, usize) = (rng.gen(), rng.gen_range(0, 64));
        let expected = x >> y;
        let mut x = Bitvec::from_u64_be(x);
        x >>= y;
        match expected == x.to_u64_be() {
            true => {}
            false => {
                dbg!(Bitvec::from_u64_be(expected));
                dbg!(x);
                panic!()
            }
        }
    }
}
#[test]
fn shl() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..100000 {
        let (x, y): (u64, usize) = (rng.gen(), rng.gen_range(0, 64));
        let expected = x << y;
        let mut x = Bitvec::from_u64_be(x);
        x <<= y;
        match expected == x.to_u64_be() {
            true => {}
            false => {
                dbg!(Bitvec::from_u64_be(expected));
                dbg!(x);
                panic!()
            }
        }
    }
}

#[test]
fn eq() {
    assert_eq!(
        Bitvec(vec![0, 3, 2, 0].into()),
        Bitvec(vec![3, 2, 0].into())
    );
    assert_ne!(Bitvec(vec![1, 0].into()), Bitvec(vec![0, 1].into()))
}

#[test]
fn cmp() {
    assert!(Bitvec(vec![3, 2, 0].into()) < Bitvec(vec![4, 3, 2, 0].into()));
    assert!(Bitvec(vec![3, 2, 0].into()) < Bitvec(vec![3, 2, 1].into()));
    assert!(Bitvec(vec![0, 3].into()) < Bitvec(vec![5].into()));
}

#[test]
fn xor() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..100000 {
        let (x, y): (u64, u64) = rng.gen();
        let expected = x ^ y;
        let mut x = Bitvec::from_u64_be(x);
        x ^= &Bitvec::from_u64_be(y);
        assert_eq!(expected, x.to_u64_be());
    }
}
