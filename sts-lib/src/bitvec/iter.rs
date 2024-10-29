//! Iterators over BitVec, multiple possible types

use crate::bitvec::BitVec;
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use std::{array, mem};
use tinyvec::ArrayVec;

/// how many of the given T fit into one usize
pub(super) const fn elements_per_usize<T: Sized>() -> usize {
    size_of::<usize>() / size_of::<T>()
}

// part of the implementation of Iterator::split() is shared between this module and array_chunks
macro_rules! shared_split_impl {
    ($self: ident, $len: ident, $primitive: ty) => {
        if $len == 0 {
            let part1 = Self {
                start: ArrayVec::new(),
                data: &[],
                end: ArrayVec::new(),
            };

            (part1, $self)
        } else if $len < $self.start.len() {
            let Self {
                mut start,
                data,
                end,
            } = $self;

            let part1 = Self {
                start: ArrayVec::from_iter(start.drain(0..$len)),
                data: &[],
                end: ArrayVec::new(),
            };

            let part2 = Self {
                start,
                data,
                end,
            };

            (part1, part2)
        } else if $len == $self.start.len() {
            let part1 = Self {
                start: $self.start,
                data: &[],
                end: ArrayVec::new(),
            };

            let part2 = Self {
                start: ArrayVec::new(),
                data: $self.data,
                end: $self.end,
            };

            (part1, part2)
        } else if $len - $self.start.len()
            >= $self.data.len() * elements_per_usize::<$primitive>()
        {
            // we can take the whole self.data
            let rem = $len
                - $self.start.len()
                - ($self.data.len() * elements_per_usize::<$primitive>());

            if rem == 0 {
                let Self { start, data, end } = $self;

                let part1 = Self {
                    start,
                    data,
                    end: ArrayVec::new(),
                };

                let part2 = Self {
                    start: ArrayVec::new(),
                    data: &[],
                    end,
                };

                (part1, part2)
            } else if rem < $self.end.len() {
                let Self {
                    start,
                    data,
                    mut end,
                } = $self;

                let part1 = Self {
                    start,
                    data,
                    end: ArrayVec::from_iter(end.drain(0..rem)),
                };

                let part2 = Self {
                    start: ArrayVec::new(),
                    data: &[],
                    end,
                };

                (part1, part2)
            } else {
                // rem == self.end.len()
                let empty = Self {
                    start: ArrayVec::new(),
                    data: &[],
                    end: ArrayVec::new(),
                };

                ($self, empty)
            }
        } else {
            // need to split self.data
            let rem = $len - $self.start.len();

            if rem % elements_per_usize::<$primitive>() == 0 {
                // clean split is possible
                let split_idx = rem / elements_per_usize::<$primitive>();
                let (part1, part2) = $self.data.split_at(split_idx);

                let part1 = Self {
                    start: $self.start,
                    data: part1,
                    end: ArrayVec::new(),
                };

                let part2 = Self {
                    start: ArrayVec::new(),
                    data: part2,
                    end: $self.end,
                };
                (part1, part2)
            } else {
                // self.data cannot be splitted into pure usize values, need to split a usize
                let (part1, middle, part2) = {
                    let split_idx = rem / elements_per_usize::<$primitive>();
                    let (part1, temp) = $self.data.split_at(split_idx);
                    let (&middle, part2) = temp.split_first().unwrap();
                    (part1, middle, part2)
                };

                // split the middle usize
                let (end, start) = {
                    let values = Self::split_usize(middle);
                    let split_idx = rem % elements_per_usize::<$primitive>();

                    let mut end = ArrayVec::new();
                    for &v in &values[0..split_idx] {
                        end.push(v);
                    }

                    let mut start = ArrayVec::new();
                    for &v in &values[split_idx..] {
                        start.push(v);
                    }

                    (end, start)
                };

                let part1 = Self {
                    start: $self.start,
                    data: part1,
                    end,
                };
                let part2 = Self {
                    start,
                    data: part2,
                    end: $self.end,
                };
                (part1, part2)
            }
        }
    }
}

pub(super) use shared_split_impl;

macro_rules! iter {
    ($name: ident<$primitive: ty> => |$u_name: ident: usize| $split_usize: block) => {
        pub struct $name<'a> {
            start: ArrayVec<[$primitive; const { elements_per_usize::<$primitive>() - 1 }]>,
            data: &'a [usize],
            end: ArrayVec<[$primitive; const { elements_per_usize::<$primitive>() - 1 }]>,
        }

        impl<'a> $name<'a> {
            pub(super) fn new(data: &'a [usize], end: ArrayVec<[$primitive; const { elements_per_usize::<$primitive>() - 1 }]>) -> Self {
                Self {
                    start: ArrayVec::new(),
                    data,
                    end,
                }
            }

            /// count the available elements
            fn count_elements(&self) -> usize {
                let start_len = self.start.len();
                let data_len = self.data.len() * elements_per_usize::<$primitive>();
                let end_len = self.end.len();
                start_len + data_len + end_len
            }

            /// split into 2 iterators, the first produces `len` items, the second produces the rest
            fn split(self, len: usize) -> (Self, Self) {
                shared_split_impl!(self, len, $primitive)
            }

            /// Split a usize into an array of the type
            pub(super) fn split_usize($u_name: usize) -> [$primitive; elements_per_usize::<$primitive>()] {
                $split_usize
            }
        }

        impl<'a> Iterator for $name<'a> {
            type Item = $primitive;

            fn next(&mut self) -> Option<Self::Item> {
                if !self.start.is_empty() {
                    Some(self.start.remove(0))
                } else if let Some((&value, rest)) = self.data.split_first() {
                    // need to take from self.data
                    self.data = rest;
                    let mut values = Self::split_usize(value).into_iter();
                    // guaranteed to contain > 1 element
                    let value = values.next();
                    // fill up start with the unused values
                    self.start.extend(values);

                    value
                } else if !self.end.is_empty(){
                    // need to take from self.end
                    Some(self.end.remove(0))
                } else {
                    None
                }
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

        impl<'a> ExactSizeIterator for $name<'a> {
            fn len(&self) -> usize {
                self.count_elements()
            }
        }

        impl<'a> DoubleEndedIterator for $name<'a> {
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
        iter!($name<$primitive> => |value: usize| {
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

macro_rules! par_iter {
    ($outer_name: ident($inner_ty: ident)) => {
        pub struct $outer_name<'a>($inner_ty<'a>);

        impl<'a> $outer_name<'a> {
            pub(super) fn new(inner: $inner_ty<'a>) -> Self {
                Self(inner)
            }
        }

        impl<'a> IndexedParallelIterator for $outer_name<'a> {
            fn len(&self) -> usize {
                self.0.len()
            }

            fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
                bridge(self, consumer)
            }

            fn with_producer<CB: ProducerCallback<Self::Item>>(self, callback: CB) -> CB::Output {
                struct CurrentProducer<'b>($inner_ty<'b>);

                impl<'b> Producer for CurrentProducer<'b> {
                    type Item = <$inner_ty<'b> as Iterator>::Item;

                    type IntoIter = $inner_ty<'b>;

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

        impl<'a> ParallelIterator for $outer_name<'a> {
            type Item = <$inner_ty<'a> as Iterator>::Item;

            fn drive_unindexed<C>(self, consumer: C) -> C::Result
            where
                C: UnindexedConsumer<Self::Item>,
            {
                bridge(self, consumer)
            }
        }
    };
}

iter!(BitvecIterU8<u8> => |value: usize| {
    value.to_be_bytes()
});
par_iter!(ParBitvecIterU8(BitvecIterU8));
iter!(BitVecIterU16<u16>);
par_iter!(ParBitVecIterU16(BitVecIterU16));
#[cfg(not(target_pointer_width = "32"))]
iter!(BitvecIterU32<u32>);
// on 32-bit systems, usize and u32 are the same
#[cfg(target_pointer_width = "32")]
iter!(BitvecIterU32<u32> => |value: usize| {
    [value as u32]
});
par_iter!(ParBitvecIterU32(BitvecIterU32));
iter!(BitvecIterUsize<usize> => |value: usize| {
    [value]
});
par_iter!(ParBitvecIterUsize(BitvecIterUsize));

/// Trait for generic iteration
pub trait BitVecIntoIter<T>
where
    T: Copy + Clone + Send + Sync,
{
    /// The type of the sequential iterator
    type Iterator<'a>: Iterator<Item = T> + ExactSizeIterator + DoubleEndedIterator + 'a
    where
        Self: 'a;

    /// The type of the parallel iterator
    type ParIterator<'a>: IndexedParallelIterator<Item = T> + 'a
    where
        Self: 'a;

    /// Iterate sequentially
    #[allow(clippy::needless_lifetimes)]
    fn iter<'a>(&'a self) -> Self::Iterator<'a>;

    /// Iterate in parallel
    #[allow(clippy::needless_lifetimes)]
    fn par_iter<'a>(&'a self) -> Self::ParIterator<'a>;
}

impl BitVecIntoIter<u8> for BitVec {
    type Iterator<'a> = BitvecIterU8<'a>;
    type ParIterator<'a> = ParBitvecIterU8<'a>;

    #[allow(clippy::needless_lifetimes)]
    fn iter<'a>(&'a self) -> Self::Iterator<'a> {
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

        BitvecIterU8::new(slice, rest)
    }

    #[allow(clippy::needless_lifetimes)]
    fn par_iter<'a>(&'a self) -> Self::ParIterator<'a> {
        ParBitvecIterU8::new(BitVecIntoIter::<u8>::iter(self))
    }
}

impl BitVecIntoIter<u16> for BitVec {
    type Iterator<'a> = BitVecIterU16<'a>;

    type ParIterator<'a> = ParBitVecIterU16<'a>;

    #[allow(clippy::needless_lifetimes)]
    fn iter<'a>(&'a self) -> Self::Iterator<'a> {
        let (slice, value) = self.as_full_slice();

        let mut rest = ArrayVec::new();
        if let Some(value) = value {
            let values = BitVecIterU16::split_usize(value);

            for value in values
                .into_iter()
                .take((self.bit_count_last_word as usize) / (u16::BITS as usize))
            {
                rest.push(value)
            }
        }

        BitVecIterU16::new(slice, rest)
    }

    #[allow(clippy::needless_lifetimes)]
    fn par_iter<'a>(&'a self) -> Self::ParIterator<'a> {
        ParBitVecIterU16::new(BitVecIntoIter::<u16>::iter(self))
    }
}

impl BitVecIntoIter<u32> for BitVec {
    type Iterator<'a> = BitvecIterU32<'a>;
    type ParIterator<'a> = ParBitvecIterU32<'a>;

    #[allow(clippy::needless_lifetimes)]
    fn iter<'a>(&'a self) -> Self::Iterator<'a> {
        let (slice, value) = self.as_full_slice();

        #[allow(unused_mut)]
        let mut rest = ArrayVec::new();
        #[cfg(not(target_pointer_width = "32"))]
        if let Some(value) = value {
            let values = BitvecIterU32::split_usize(value);

            for value in values
                .into_iter()
                .take((self.bit_count_last_word as usize) / (u32::BITS as usize))
            {
                rest.push(value)
            }
        }

        BitvecIterU32::new(slice, rest)
    }

    #[allow(clippy::needless_lifetimes)]
    fn par_iter<'a>(&'a self) -> Self::ParIterator<'a> {
        ParBitvecIterU32::new(BitVecIntoIter::<u32>::iter(self))
    }
}

impl BitVecIntoIter<usize> for BitVec {
    type Iterator<'a> = BitvecIterUsize<'a>;
    type ParIterator<'a> = ParBitvecIterUsize<'a>;

    #[allow(clippy::needless_lifetimes)]
    fn iter<'a>(&'a self) -> Self::Iterator<'a> {
        let (slice, _) = self.as_full_slice();

        BitvecIterUsize::new(slice, ArrayVec::new())
    }

    #[allow(clippy::needless_lifetimes)]
    fn par_iter<'a>(&'a self) -> Self::ParIterator<'a> {
        ParBitvecIterUsize::new(BitVecIntoIter::<usize>::iter(self))
    }
}
