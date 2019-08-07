pub trait Bitwise {
    type Output;
    fn count(&self) -> Self::Output;
    fn parity(&self) -> Self::Output;
}

impl Bitwise for u128 {
    type Output = u128;
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

impl Bitwise for u64 {
    type Output = u64;
    fn count(&self) -> u64 {
        let mut count = 0;
        let mut copy = *self;
        while copy != 0 {
            count += copy & 1;
            copy >>= 1;
        }
        count
    }

    fn parity(&self) -> u64 {
        let mut result = *self;
        result ^= result >> 32;
        result ^= result >> 16;
        result ^= result >> 8;
        result ^= result >> 4;
        result ^= result >> 2;
        result ^= result >> 1;
        (result & 1)
    }
}

impl Bitwise for u8 {
    type Output = u8;
    fn count(&self) -> u8 {
        let mut count = 0;
        let mut copy = *self;
        while copy != 0 {
            count += copy & 1;
            copy >>= 1;
        }
        count
    }

    fn parity(&self) -> u8 {
        let mut result = *self;
        result ^= result >> 4;
        result ^= result >> 2;
        result ^= result >> 1;
        (result & 1)
    }
}

impl<I> Bitwise for [I]
where
    I: Bitwise,
    <I as Bitwise>::Output: Into<usize>,
{
    type Output = usize;
    fn count(&self) -> usize {
        self.iter().fold(0, |val, el| {
            val + <<I as Bitwise>::Output as Into<usize>>::into(el.count())
        })
    }
    fn parity(&self) -> usize {
        self.iter().fold(0, |val, el| {
            val ^ <<I as Bitwise>::Output as Into<usize>>::into(el.parity())
        })
    }
}

#[cfg(feature = "dyn")]
impl<I> Bitwise for std::collections::VecDeque<I>
where
    I: Bitwise,
    <I as Bitwise>::Output: Into<usize>,
{
    type Output = usize;
    fn count(&self) -> usize {
        self.iter().fold(0, |val, el| {
            val + <<I as Bitwise>::Output as Into<usize>>::into(el.count())
        })
    }
    fn parity(&self) -> usize {
        self.iter().fold(0, |val, el| {
            val ^ <<I as Bitwise>::Output as Into<usize>>::into(el.parity())
        })
    }
}

#[test]
fn test_count() {
    assert_eq!(1u128.count(), 1);
    assert_eq!(2u128.count(), 1);
    assert_eq!(3u128.count(), 2);
    assert_eq!([2u8, 2, 5].count(), 4);
    assert_eq!([2u8, 2, 8].count(), 3);
}

#[test]
fn test_parity() {
    assert_eq!(1u128.parity(), 1);
    assert_eq!(2u128.parity(), 1);
    assert_eq!(3u128.parity(), 0);
    assert_eq!([2u8, 2, 5].parity(), 0);
    assert_eq!([2u8, 2, 8].parity(), 1);
}

#[cfg(feature = "nightly")]
#[bench]
fn parity_u64(b: &mut test::Bencher) {
    let mut guard = 1;
    let v = 0x1203_0245_2124_9151u64;
    b.iter(|| {
        for _ in 0..1_000_000usize {
            guard = v.parity()
        }
    });
}

#[cfg(feature = "nightly")]
#[bench]
fn parity_u8_8(b: &mut test::Bencher) {
    let mut guard = 1;
    let v = [0x12u8, 0x03, 0x02, 0x45, 0x21, 0x24, 0x91, 0x51];
    b.iter(|| {
        for _ in 0..1_000_000usize {
            guard = v.parity()
        }
    });
}

#[test]
fn test_count_parity() {
    for x in (0..100).map(|x| x * (core::u128::MAX / 100)) {
        assert_eq!(x.parity(), x.count() % 2);
    }
}
