use crate::bitwise::Bitwise;
use std::cmp::max;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Bitvec(pub VecDeque<u8>);

impl Bitwise for Bitvec {
    type Output = usize;

    fn count(&self) -> Self::Output {
        self.0.count()
    }

    fn parity(&self) -> Self::Output {
        self.0.parity()
    }
}

impl std::cmp::PartialEq for Bitvec {
    fn eq(&self, other: &Self) -> bool {
        let (longest, shortest) = if self.0.len() > other.0.len() {
            (&self.0, &other.0)
        } else {
            (&other.0, &self.0)
        };
        let (mut longest, mut shortest) = (longest.iter().rev(), shortest.iter().rev());
        for (i, l) in longest.enumerate() {
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

impl std::cmp::PartialOrd for Bitvec {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let (longest, shortest, on_longest_superior) = if self.0.len() > other.0.len() {
            (&self.0, &other.0, std::cmp::Ordering::Greater)
        } else {
            (&other.0, &self.0, std::cmp::Ordering::Less)
        };
        for (i, l) in longest.iter().enumerate().rev() {
            if let Some(s) = shortest.get(i) {
                if l > s {
                    return Some(on_longest_superior);
                } else if l < s {
                    return Some(match on_longest_superior {
                        std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
                        std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
                        _ => unreachable!(),
                    });
                }
            } else if *l != 0 {
                return Some(on_longest_superior);
            }
        }
        Some(std::cmp::Ordering::Equal)
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
        let mut longest = longest.iter().rev();
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
        let mut remainder = 0;
        let mut i = self.0.len();
        let mut j = rhs.0.len() as isize;
        while i > 0 {
            i -= 1;
            j -= 1;
            remainder += self.0[i];
            if j >= 0 {
                remainder += rhs.0[j as usize];
            }
            self.0[i] = remainder as u8;
            remainder = remainder.overflowing_shr(8).0;
        }
        while remainder > 0 {
            self.0.push_front(remainder as u8);
            remainder = remainder.overflowing_shr(8).0;
        }
    }
}

impl std::ops::AddAssign<u32> for Bitvec {
    fn add_assign(&mut self, rhs: u32) {
        let mut remainder = rhs;
        let mut i = self.0.len();
        while i > 0 {
            i -= 1;
            remainder += self.0[i] as u32;
            self.0[i] = remainder as u8;
            remainder = remainder.overflowing_shr(8).0;
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

impl std::ops::ShrAssign<usize> for Bitvec {
    fn shr_assign(&mut self, rhs: usize) {
        let (mut byte_shift, bit_shift) = (rhs / 8, (rhs % 8) as u32);
        let bit_lshift = 8 - bit_shift;
        for i in (0..self.0.len()).rev() {
            if dbg!(bit_shift) == 0 {
                if let Some(&value) = self.0.get(i.overflowing_sub(byte_shift).0) {
                    self.0[i] = dbg!(value);
                } else {
                    self.0[i] = dbg!(0);
                }
            } else {
                if let Some(&value) = self.0.get(i.overflowing_sub(byte_shift).0) {
                    self.0[i] = dbg!(value >> bit_shift);
                }
                if let Some(&value) = self.0.get(i.overflowing_sub(byte_shift + 1).0) {
                    self.0[i] |= dbg!(value << bit_lshift);
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

impl std::ops::ShlAssign<usize> for Bitvec {
    fn shl_assign(&mut self, rhs: usize) {
        let (mut byte_shift, bit_shift) = (rhs / 8, (rhs % 8) as u32);
        let mut extension: VecDeque<u8> = vec![0; byte_shift + if bit_shift > 0 {1} else {0}].into();
        extension.extend(&self.0);
        self.0 = extension;
        let bit_lshift = 8 - bit_shift;
        for i in (0..self.0.len()) {
            dbg!(&self);
            if dbg!(bit_shift) == 0 {
                if let Some(&value) = self.0.get(dbg!(i + byte_shift)) {
                    self.0[i] = dbg!(value);
                } else {
                    self.0[i] = dbg!(0);
                }
            } else {
                if let Some(&value) = self.0.get(i + byte_shift) {
                    self.0[i] = dbg!(value << bit_shift);
                }
                if let Some(&value) = self.0.get(i + byte_shift + 1) {
                    self.0[i] |= dbg!(value >> bit_lshift);
                }
            }
        }
        dbg!(&self);
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
            if let Some(&r) = rhs.next(){
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

impl std::ops::BitAnd for &Bitvec {
    type Output = Bitvec;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitvec(self.0.iter().rev().zip(rhs.0.iter()).map(|(l, r)| l & r).collect())
    }
}

impl std::ops::BitAnd<u8> for &Bitvec {
    type Output = Bitvec;

    fn bitand(self, rhs: u8) -> Self::Output {
        Bitvec(vec![self.0[self.0.len() - 1] & rhs].into())
    }
}

#[test]
fn add() {
    let mut a = Bitvec(vec![0x3, 0x42, 0].into());
    a += 1;
    assert_eq!(a, Bitvec(vec![0x3, 0x42, 1].into()))
}

#[test]
fn shr() {
    let mut a = Bitvec(vec![0x3, 0x42, 0].into());
    let mut b = a.clone();
    a >>= 4;
    b >>= 8;
    assert_eq!(b, Bitvec(vec![3, 0x42].into()));
    assert_eq!(a, Bitvec(vec![0x34, 0x20].into()));
}
#[test]
fn shl() {
    let mut a = Bitvec(vec![0x3, 0x42, 0].into());
    let mut b = a.clone();
    b <<= 8;
    a <<= 4;
    assert_eq!(b, Bitvec(vec![3, 0x42, 0, 0].into()));
//    assert_eq!(a, Bitvec(vec![0x34, 0x20, 0].into()));
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
}

#[test]
fn xor() {
    let a = Bitvec(vec![3, 2, 0].into());
    let b = Bitvec(vec![1, 2, 0].into());
    let mut c = a.clone();
    let d = Bitvec(vec![2, 0].into());
    let mut e = d.clone();
    let mut f = a.clone();
    c ^= &a;
    e ^= &a;
    f ^= &d;
    assert_eq!(c, Bitvec(vec![].into()));
    assert_eq!(&a ^ &b, Bitvec(vec![2, 0, 0].into()));
    assert_eq!(f, Bitvec(vec![3, 0, 0].into()));
    assert_eq!(&b ^ &d, Bitvec(vec![1, 0, 0].into()));
    assert_eq!(e, Bitvec(vec![3, 0, 0].into()));
}
