//! Special iterators that allow iteration over chunks of values of a given type

use crate::bitvec::iter::{elements_per_usize, shared_split_impl};
use crate::bitvec::BitVec;
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use std::{array, mem};
use tinyvec::ArrayVec;

/// Implementation for an iterator yielding chunks of the given type, using a usize slice as base.
/// A generic implementation is not possible because Rust does not support const generics affecting
/// the struct size as of now.
macro_rules! impl_chunks {
    ($name: ident<$primitive: ty> => |$u_name: ident: usize| $split_usize: block) => {
        pub(crate) struct $name<'a, const N: usize> {
            start: ArrayVec<[$primitive; const { elements_per_usize::<$primitive>() - 1 }]>,
            data: &'a [usize],
            end: ArrayVec<[$primitive; const { elements_per_usize::<$primitive>() - 1 }]>,
        }

        impl<'a, const N: usize> $name<'a, N> {
            pub(super) fn new(data: &'a [usize], end: ArrayVec<[$primitive; const { elements_per_usize::<$primitive>() - 1 }]>) -> Self {
                Self {
                    start: ArrayVec::new(),
                    data,
                    end,
                }
            }

            /// count the available elements, not chunks!
            fn count_elements(&self) -> usize {
                let start_len = self.start.len();
                let data_len = self.data.len() * elements_per_usize::<$primitive>();
                let end_len = self.end.len();
                start_len + data_len + end_len
            }

            /// split into 2 iterators, the first produces `len` items, the second produces the rest
            fn split(self, len: usize) -> (Self, Self) {
                // how many elements (not chunks) are needed
                let len = len * N;

                // start is always taken in full because it is smaller than N
                shared_split_impl!(self, len, $primitive)
            }

            /// Split a usize into an array of the type
            pub(super) fn split_usize($u_name: usize) -> [$primitive; elements_per_usize::<$primitive>()] {
                $split_usize
            }
        }

        impl<'a, const N: usize> Iterator for $name<'a, N> {
            type Item = [$primitive; N];

            fn next(&mut self) -> Option<Self::Item> {
                if self.count_elements() < N {
                    return None;
                }

                // allocate next item
                let mut next: [$primitive; N] = [0; N];

                // how many items are needed after start
                let mut current_idx = self.start.len();
                let mut remaining_len = N - self.start.len();

                // how many usize values is that
                let remaining_usize_len = {
                    let base = remaining_len * size_of::<$primitive>();
                    let len = base / size_of::<usize>();

                    // if a usize will be split
                    let len = if base % size_of::<usize>() == 0 {
                        len
                    } else {
                        len + 1
                    };

                    // if some elements come from end
                    if len > self.data.len() {
                        self.data.len()
                    } else {
                        len
                    }
                };

                // take all items out of start
                next.iter_mut()
                    .zip(self.start.drain(0..self.start.len()))
                    .for_each(|(dst, src)| *dst = src);

                // take from data
                let (data, rest) = self.data.split_at(remaining_usize_len);
                self.data = rest;

                for value in data {
                    let mut values = ArrayVec::from(Self::split_usize(*value));

                    // how many items to add to the array
                    let length = if remaining_len > values.len() {
                        values.len()
                    } else {
                        remaining_len
                    };

                    for value in values.drain(0..length) {
                        next[current_idx] = value;
                        current_idx += 1;
                        remaining_len -= 1;
                    }

                    // if values still contains items, this is the last value - items should be stored into start
                    if !values.is_empty() {
                        self.start.extend(values);
                    }
                }

                // add the last elements
                if remaining_len != 0 {
                    next[current_idx..]
                        .iter_mut()
                        .zip(self.end.drain(0..self.end.len()))
                        .for_each(|(dst, src)| *dst = src);
                }

                Some(next)
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (ExactSizeIterator::len(self), Some(ExactSizeIterator::len(self)))
            }

            fn count(self) -> usize
            where
                Self: Sized,
            {
                ExactSizeIterator::len(&self)
            }
        }

        impl<'a, const N: usize> ExactSizeIterator for $name<'a, N> {
            fn len(&self) -> usize {
                self.count_elements() / N
            }
        }

        impl<'a, const N: usize> DoubleEndedIterator for $name<'a, N> {
            fn next_back(&mut self) -> Option<Self::Item> {
                let mut temp = Self {
                    start: Default::default(),
                    data: &[],
                    end: Default::default(),
                };

                mem::swap(self, &mut temp);

                let mut part2;
                (*self, part2) = temp.split(self.len() - 1);
                part2.next()
            }
        }
    };

    ($name: ident<$primitive: ty>) => {
        impl_chunks!($name<$primitive> => |value: usize| {
            let bytes = value.to_be_bytes();

            let array: [$primitive; elements_per_usize::<$primitive>()] = array::from_fn(|i| {
                let bytes = bytes
                [i * size_of::<$primitive>()..(i + 1) * size_of::<$primitive>()]
                .try_into()
                .unwrap();
                <$primitive>::from_be_bytes(bytes)
            });

            array
        });
    }
}

/// Implements a parallel iterator that wraps the given serial iterator, yielding array chunks.
macro_rules! impl_chunks_par {
    ($outer_name: ident($inner_ty: ident)) => {
        pub(crate) struct $outer_name<'a, const N: usize>($inner_ty<'a, N>);

        impl<'a, const N: usize> $outer_name<'a, N> {
            pub(super) fn new(inner: $inner_ty<'a, N>) -> Self {
                Self(inner)
            }
        }

        impl<'a, const N: usize> IndexedParallelIterator for $outer_name<'a, N> {
            fn len(&self) -> usize {
                self.0.len()
            }

            fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
                bridge(self, consumer)
            }

            fn with_producer<CB: ProducerCallback<Self::Item>>(self, callback: CB) -> CB::Output {
                struct CurrentProducer<'b, const M: usize>($inner_ty<'b, M>);

                impl<'b, const M: usize> Producer for CurrentProducer<'b, M> {
                    type Item = <$inner_ty<'b, M> as Iterator>::Item;

                    type IntoIter = $inner_ty<'b, M>;

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

        impl<'a, const N: usize> ParallelIterator for $outer_name<'a, N> {
            type Item = <$inner_ty<'a, N> as Iterator>::Item;

            fn drive_unindexed<C>(self, consumer: C) -> C::Result
            where
                C: UnindexedConsumer<Self::Item>,
            {
                bridge(self, consumer)
            }
        }
    };
}

impl_chunks!(BitVecChunksU8<u8> => |value: usize| {
    value.to_be_bytes()
});
impl_chunks_par!(ParBitVecChunksU8(BitVecChunksU8));
impl_chunks!(BitVecChunksU16<u16>);
impl_chunks_par!(ParBitVecChunksU16(BitVecChunksU16));
#[cfg(not(target_pointer_width = "32"))]
impl_chunks!(BitVecChunksU32<u32>);
// on 32-bit systems, usize and u32 are the same
#[cfg(target_pointer_width = "32")]
impl_chunks!(BitVecChunksU32<u32> => |value: usize| {
    [value as u32]
});
impl_chunks_par!(ParBitVecChunksU32(BitVecChunksU32));
impl_chunks!(BitVecChunksUsize<usize> => |value: usize| {
    [value]
});
impl_chunks_par!(ParBitVecChunksUsize(BitVecChunksUsize));

/// Trait for generic access to typed array chunks - meaning chunk iterators that yield their
/// chunks as arrays.
pub(crate) trait BitVecChunks<T>
where
    T: Copy + Clone + Send + Sync,
{
    /// The type of the sequential iterator
    type Iterator<'a, const N: usize>: Iterator<Item = [T; N]>
        + ExactSizeIterator
        + DoubleEndedIterator
        + 'a
    where
        Self: 'a;

    /// The type of the parallel iterator
    type ParIterator<'a, const N: usize>: IndexedParallelIterator<Item = [T; N]> + 'a
    where
        Self: 'a;

    /// Iterate over chunks sequentially
    #[allow(clippy::needless_lifetimes)]
    fn chunks<'a, const N: usize>(&'a self) -> Self::Iterator<'a, N>;

    /// Iterator over chunks in parallel
    #[allow(clippy::needless_lifetimes)]
    fn par_chunks<'a, const N: usize>(&'a self) -> Self::ParIterator<'a, N>;
}

impl BitVecChunks<u8> for BitVec {
    type Iterator<'a, const N: usize> = BitVecChunksU8<'a, N>;
    type ParIterator<'a, const N: usize> = ParBitVecChunksU8<'a, N>;

    #[allow(clippy::needless_lifetimes)]
    fn chunks<'a, const N: usize>(&'a self) -> Self::Iterator<'a, N> {
        let (slice, value) = self.as_full_slice();

        let mut rest = ArrayVec::new();
        if let Some(value) = value {
            let values = value.to_be_bytes();

            for value in values
                .into_iter()
                .take((self.bit_count_last_word as usize) / (u8::BITS as usize))
            {
                rest.push(value)
            }
        }

        BitVecChunksU8::new(slice, rest)
    }

    #[allow(clippy::needless_lifetimes)]
    fn par_chunks<'a, const N: usize>(&'a self) -> Self::ParIterator<'a, N> {
        ParBitVecChunksU8::new(BitVecChunks::<u8>::chunks::<N>(self))
    }
}

impl BitVecChunks<u16> for BitVec {
    type Iterator<'a, const N: usize> = BitVecChunksU16<'a, N>;

    type ParIterator<'a, const N: usize> = ParBitVecChunksU16<'a, N>;

    #[allow(clippy::needless_lifetimes)]
    fn chunks<'a, const N: usize>(&'a self) -> Self::Iterator<'a, N> {
        let (slice, value) = self.as_full_slice();

        let mut rest = ArrayVec::new();
        if let Some(value) = value {
            let values = BitVecChunksU16::<N>::split_usize(value);

            for value in values
                .into_iter()
                .take((self.bit_count_last_word as usize) / (u16::BITS as usize))
            {
                rest.push(value)
            }
        }

        BitVecChunksU16::new(slice, rest)
    }

    #[allow(clippy::needless_lifetimes)]
    fn par_chunks<'a, const N: usize>(&'a self) -> Self::ParIterator<'a, N> {
        ParBitVecChunksU16::new(BitVecChunks::<u16>::chunks::<N>(self))
    }
}

impl BitVecChunks<u32> for BitVec {
    type Iterator<'a, const N: usize> = BitVecChunksU32<'a, N>;
    type ParIterator<'a, const N: usize> = ParBitVecChunksU32<'a, N>;

    #[allow(clippy::needless_lifetimes)]
    fn chunks<'a, const N: usize>(&'a self) -> Self::Iterator<'a, N> {
        let (slice, value) = self.as_full_slice();

        #[allow(unused_mut)]
        let mut rest = ArrayVec::new();
        #[cfg(not(target_pointer_width = "32"))]
        if let Some(value) = value {
            let values = BitVecChunksU32::<N>::split_usize(value);

            for value in values
                .into_iter()
                .take((self.bit_count_last_word as usize) / (u32::BITS as usize))
            {
                rest.push(value)
            }
        }

        BitVecChunksU32::new(slice, rest)
    }

    #[allow(clippy::needless_lifetimes)]
    fn par_chunks<'a, const N: usize>(&'a self) -> Self::ParIterator<'a, N> {
        ParBitVecChunksU32::new(BitVecChunks::<u32>::chunks::<N>(self))
    }
}

impl BitVecChunks<usize> for BitVec {
    type Iterator<'a, const N: usize> = BitVecChunksUsize<'a, N>;
    type ParIterator<'a, const N: usize> = ParBitVecChunksUsize<'a, N>;

    #[allow(clippy::needless_lifetimes)]
    fn chunks<'a, const N: usize>(&'a self) -> Self::Iterator<'a, N> {
        let (slice, _) = self.as_full_slice();

        BitVecChunksUsize::new(slice, ArrayVec::new())
    }

    #[allow(clippy::needless_lifetimes)]
    fn par_chunks<'a, const N: usize>(&'a self) -> Self::ParIterator<'a, N> {
        ParBitVecChunksUsize::new(BitVecChunks::<usize>::chunks::<N>(self))
    }
}
