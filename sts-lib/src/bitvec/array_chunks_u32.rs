//! Iterator over a BitVec, always returning an even count of u32 values.
// This implementation is specifically for the binary matrix rank test.
// Since the test needs an even count of values, the code can be optimized to be quite simple on both
// 32 and 64-bit platforms.

use crate::bitvec::BitVec;
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::prelude::*;

/// Supports iteration over N u32 at a time. N must be even.
pub struct BitVecU32Chunks<'a, const N: usize>(&'a [usize]);

impl<const N: usize> BitVecU32Chunks<'_, N> {
    /// Split the iterator into 2, with the first one having the specified length.
    ///
    /// Panics if the length is greater than the iterator length.
    //noinspection RsAssertEqual
    fn split(self, len: usize) -> (Self, Self) {
        const { assert!(N % 2 == 0, "N must be even") };

        // will always be even
        let len = len * N;
        let len = {
            // on a 64-bit platform, each usize contains 2 u32
            #[cfg(target_pointer_width = "64")]
            {
                len / 2
            }
            #[cfg(target_pointer_width = "32")]
            {
                len
            }
        };

        let part1 = &self.0[0..len];
        let part2 = &self.0[len..];

        let part1 = Self(part1);
        let part2 = Self(part2);
        (part1, part2)
    }
}

impl<const N: usize> Iterator for BitVecU32Chunks<'_, N> {
    type Item = [u32; N];

    //noinspection RsAssertEqual
    #[cfg(target_pointer_width = "64")]
    fn next(&mut self) -> Option<Self::Item> {
        use std::array;
        const { assert!(N % 2 == 0, "N must be even") };

        let count_usize = N / 2;
        let (data, last) = self.0.split_at_checked(count_usize)?;
        self.0 = last;

        let result: [u32; N] = array::from_fn(|i| {
            let element = data[i / 2];
            if i % 2 == 0 {
                (element >> u32::BITS) as u32
            } else {
                element as u32
            }
        });

        Some(result)
    }

    #[cfg(target_pointer_width = "32")]
    fn next(&mut self) -> Option<Self::Item> {
        use std::mem;

        let (data, last) = self.0.split_first_chunk::<N>()?;
        self.0 = last;
        // Safety: this transmute is safe, because on platforms with target_pointer_width = 32,
        // usize and u32 are the same type. Thus, both arrays are guaranteed to have the same size and
        // alignment. The referenced value is then copied - no problem.
        Some(*unsafe { mem::transmute::<&[usize; N], &[u32; N]>(data) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }

    fn count(self) -> usize {
        self.len()
    }
}

impl<const N: usize> ExactSizeIterator for BitVecU32Chunks<'_, N> {
    fn len(&self) -> usize {
        let len = {
            #[cfg(target_pointer_width = "64")]
            {
                self.0.len() * 2
            }
            #[cfg(target_pointer_width = "32")]
            {
                self.data.len()
            }
        };

        len / N
    }
}

impl<const N: usize> DoubleEndedIterator for BitVecU32Chunks<'_, N> {
    #[cfg(target_pointer_width = "64")]
    fn next_back(&mut self) -> Option<Self::Item> {
        use std::array;

        let count_usize = N / 2;
        let (first, data) = self.0.split_at_checked(self.0.len() - count_usize)?;
        self.0 = first;

        let result: [u32; N] = array::from_fn(|i| {
            let element = data[i / 2];
            if i % 2 == 0 {
                (element >> u32::BITS) as u32
            } else {
                element as u32
            }
        });

        Some(result)
    }

    #[cfg(target_pointer_width = "32")]
    fn next_back(&mut self) -> Option<Self::Item> {
        use std::mem;

        let (first, data) = self.0.split_last_chunk::<N>()?;
        self.0 = first;
        // Safety: this transmute is safe, because on platforms with target_pointer_width = 32,
        // usize and u32 are the same type. Thus, both arrays are guaranteed to have the same size and
        // alignment. The referenced value is then copied - no problem.
        Some(*unsafe { mem::transmute::<&[usize; N], &[u32; N]>(data) })
    }
}

/// Supports iteration over N u32 at a time. N must be even. Parallel.
pub struct BitVecU32ParChunks<'a, const N: usize>(BitVecU32Chunks<'a, N>);

impl<const N: usize> IndexedParallelIterator for BitVecU32ParChunks<'_, N> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        bridge(self, consumer)
    }

    fn with_producer<CB: ProducerCallback<Self::Item>>(self, callback: CB) -> CB::Output {
        struct CurrentProducer<'b, const M: usize>(BitVecU32Chunks<'b, M>);

        impl<'b, const M: usize> Producer for CurrentProducer<'b, M> {
            type Item = <BitVecU32Chunks<'b, M> as Iterator>::Item;

            type IntoIter = BitVecU32Chunks<'b, M>;

            fn into_iter(self) -> Self::IntoIter {
                self.0
            }

            fn split_at(self, index: usize) -> (Self, Self) {
                let (i1, i2) = self.0.split(index);
                (Self(i1), Self(i2))
            }
        }

        callback.callback(CurrentProducer(self.0))
    }
}

impl<'a, const N: usize> ParallelIterator for BitVecU32ParChunks<'a, N> {
    type Item = <BitVecU32Chunks<'a, N> as Iterator>::Item;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }
}

// constructors on BitVec
impl BitVec {
    /// Returns an iterator that yields N u32 values at a time. N must be even.
    // const context does not support assert_eq!()
    //noinspection RsAssertEqual
    pub fn array_chunks_u32<const N: usize>(&self) -> BitVecU32Chunks<N> {
        const { assert!(N % 2 == 0, "N must be even") };

        let (data, _) = self.as_full_slice();
        BitVecU32Chunks(data)
    }

    /// Returns a parallel iterator that yields N u32 values at a time. N must be even.
    //noinspection RsAssertEqual
    pub fn par_array_chunks_u32<const N: usize>(&self) -> BitVecU32ParChunks<N> {
        const { assert!(N % 2 == 0, "N must be even") };
        BitVecU32ParChunks(self.array_chunks_u32())
    }
}
