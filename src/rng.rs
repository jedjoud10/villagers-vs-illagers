use std::ops::{Bound, RangeBounds};

// Our custom Rng forked from fastrand to NOT use a thread local Cell<T>
// It was taking way too much space in memory so let us not do that lol
pub struct Rng(u64);

// Directly copied from their implementation lol
macro_rules! rng_integer {
    ($t:tt, $unsigned_t:tt, $gen:tt, $mod:tt, $doc:tt) => {
        #[doc = $doc]
        ///
        /// Panics if the range is empty.
        #[inline]
        pub fn $t(&mut self, range: impl RangeBounds<$t>) -> $t {
            let low = match range.start_bound() {
                Bound::Unbounded => core::$t::MIN,
                Bound::Included(&x) => x,
                Bound::Excluded(&x) => x.overflowing_add(1).0,
            };

            let high = match range.end_bound() {
                Bound::Unbounded => core::$t::MAX,
                Bound::Included(&x) => x,
                Bound::Excluded(&x) => x.overflowing_sub(1).0,
            };

            if low == core::$t::MIN && high == core::$t::MAX {
                self.$gen() as $t
            } else {
                let len = high.wrapping_sub(low).wrapping_add(1);
                low.wrapping_add(self.$mod(len as $unsigned_t as _) as $t)
            }
        }
    };
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn gen_u64(&mut self) -> u64 {
        let s = self.0.wrapping_add(0xA0761D6478BD642F);
        self.0 = s;
        let t = (s as u128) * ((s ^ 0xE7037ED1A0B428DB) as u128);
        (t as u64) ^ (t >> 64) as u64
    }

    #[inline(always)]
    fn gen_u32(&mut self) -> u32 {
        (self.gen_u64() & (u32::MAX as u64)) as u32
    }

    
    /// Generates a random `u128`.
    #[inline]
    fn gen_u128(&mut self) -> u128 {
        (u128::from(self.gen_u64()) << 64) | u128::from(self.gen_u64())
    }

    /// Generates a random `u32` in `0..n`.
    #[inline]
    fn gen_mod_u32(&mut self, n: u32) -> u32 {
        // Adapted from: https://lemire.me/blog/2016/06/30/fast-random-shuffling/
        let mut r = self.gen_u32();
        let mut hi = mul_high_u32(r, n);
        let mut lo = r.wrapping_mul(n);
        if lo < n {
            let t = n.wrapping_neg() % n;
            while lo < t {
                r = self.gen_u32();
                hi = mul_high_u32(r, n);
                lo = r.wrapping_mul(n);
            }
        }
        hi
    }

    /// Generates a random `u64` in `0..n`.
    #[inline]
    fn gen_mod_u64(&mut self, n: u64) -> u64 {
        // Adapted from: https://lemire.me/blog/2016/06/30/fast-random-shuffling/
        let mut r = self.gen_u64();
        let mut hi = mul_high_u64(r, n);
        let mut lo = r.wrapping_mul(n);
        if lo < n {
            let t = n.wrapping_neg() % n;
            while lo < t {
                r = self.gen_u64();
                hi = mul_high_u64(r, n);
                lo = r.wrapping_mul(n);
            }
        }
        hi
    }

    /// Generates a random `u128` in `0..n`.
    #[inline]
    fn gen_mod_u128(&mut self, n: u128) -> u128 {
        // Adapted from: https://lemire.me/blog/2016/06/30/fast-random-shuffling/
        let mut r = self.gen_u128();
        let mut hi = mul_high_u128(r, n);
        let mut lo = r.wrapping_mul(n);
        if lo < n {
            let t = n.wrapping_neg() % n;
            while lo < t {
                r = self.gen_u128();
                hi = mul_high_u128(r, n);
                lo = r.wrapping_mul(n);
            }
        }
        hi
    }

    
    rng_integer!(
        i8,
        u8,
        gen_u32,
        gen_mod_u32,
        "Generates a random `i8` in the given range."
    );

    rng_integer!(
        i16,
        u16,
        gen_u32,
        gen_mod_u32,
        "Generates a random `i16` in the given range."
    );

    rng_integer!(
        i32,
        u32,
        gen_u32,
        gen_mod_u32,
        "Generates a random `i32` in the given range."
    );

    rng_integer!(
        i64,
        u64,
        gen_u64,
        gen_mod_u64,
        "Generates a random `i64` in the given range."
    );

    rng_integer!(
        i128,
        u128,
        gen_u128,
        gen_mod_u128,
        "Generates a random `i128` in the given range."
    );

    #[cfg(target_pointer_width = "16")]
    rng_integer!(
        isize,
        usize,
        gen_u32,
        gen_mod_u32,
        "Generates a random `isize` in the given range."
    );
    #[cfg(target_pointer_width = "32")]
    rng_integer!(
        isize,
        usize,
        gen_u32,
        gen_mod_u32,
        "Generates a random `isize` in the given range."
    );
    #[cfg(target_pointer_width = "64")]
    rng_integer!(
        isize,
        usize,
        gen_u64,
        gen_mod_u64,
        "Generates a random `isize` in the given range."
    );

    
    rng_integer!(
        u8,
        u8,
        gen_u32,
        gen_mod_u32,
        "Generates a random `u8` in the given range."
    );

    rng_integer!(
        u16,
        u16,
        gen_u32,
        gen_mod_u32,
        "Generates a random `u16` in the given range."
    );

    rng_integer!(
        u32,
        u32,
        gen_u32,
        gen_mod_u32,
        "Generates a random `u32` in the given range."
    );

    rng_integer!(
        u64,
        u64,
        gen_u64,
        gen_mod_u64,
        "Generates a random `u64` in the given range."
    );

    rng_integer!(
        u128,
        u128,
        gen_u128,
        gen_mod_u128,
        "Generates a random `u128` in the given range."
    );

    #[cfg(target_pointer_width = "16")]
    rng_integer!(
        usize,
        usize,
        gen_u32,
        gen_mod_u32,
        "Generates a random `usize` in the given range."
    );
    #[cfg(target_pointer_width = "32")]
    rng_integer!(
        usize,
        usize,
        gen_u32,
        gen_mod_u32,
        "Generates a random `usize` in the given range."
    );
    #[cfg(target_pointer_width = "64")]
    rng_integer!(
        usize,
        usize,
        gen_u64,
        gen_mod_u64,
        "Generates a random `usize` in the given range."
    );
    #[cfg(target_pointer_width = "128")]
    rng_integer!(
        usize,
        usize,
        gen_u128,
        gen_mod_u128,
        "Generates a random `usize` in the given range."
    );

}

/// Computes `(a * b) >> 32`.
#[inline]
fn mul_high_u32(a: u32, b: u32) -> u32 {
    (((a as u64) * (b as u64)) >> 32) as u32
}

/// Computes `(a * b) >> 64`.
#[inline]
fn mul_high_u64(a: u64, b: u64) -> u64 {
    (((a as u128) * (b as u128)) >> 64) as u64
}

/// Computes `(a * b) >> 128`.
#[inline]
fn mul_high_u128(a: u128, b: u128) -> u128 {
    // Adapted from: https://stackoverflow.com/a/28904636
    let a_lo = a as u64 as u128;
    let a_hi = (a >> 64) as u64 as u128;
    let b_lo = b as u64 as u128;
    let b_hi = (b >> 64) as u64 as u128;
    let carry = (a_lo * b_lo) >> 64;
    let carry = ((a_hi * b_lo) as u64 as u128 + (a_lo * b_hi) as u64 as u128 + carry) >> 64;
    a_hi * b_hi + ((a_hi * b_lo) >> 64) + ((a_lo * b_hi) >> 64) + carry
}