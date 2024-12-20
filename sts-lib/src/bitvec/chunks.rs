//! Chunk iterators - chunks are counted in bytes. The chunk type is not optimized for usage,
//! but rather for performance when using it in tests, see [Chunk].

use crate::bitvec::BitVec;
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::prelude::*;
use std::mem;
use tinyvec::ArrayVec;

/// Length of the start / end ArrayVecs
pub const PART_ARRAY_LEN: usize = BYTES_PER_USIZE - 1;

/// How many bytes fit into 1 usize value
const BYTES_PER_USIZE: usize = size_of::<usize>() / size_of::<u8>();

/// A generic chunk: contains full bytes, but only split as necessary.
/// This allows for a large part of each chunk to be given as a reference.
pub struct Chunk<'a> {
    /// Start: not a complete usize value
    pub start: ArrayVec<[u8; PART_ARRAY_LEN]>,
    /// Middle: complete usize values
    pub middle: &'a [usize],
    /// End: not a complete usize value
    pub end: ArrayVec<[u8; PART_ARRAY_LEN]>,
}

impl<'a> Chunk<'a> {
    /// The length of the chunk, measured in bytes.
    pub fn len_byte(&self) -> usize {
        self.start.len() + self.middle.len() * BYTES_PER_USIZE + self.end.len()
    }

    /// create a new instance
    #[inline]
    fn new(
        start: ArrayVec<[u8; PART_ARRAY_LEN]>,
        middle: &'a [usize],
        end: ArrayVec<[u8; PART_ARRAY_LEN]>,
    ) -> Self {
        Self { start, middle, end }
    }

    /// create a new, empty instance
    #[inline]
    fn new_empty() -> Self {
        Self {
            start: ArrayVec::new(),
            middle: &[],
            end: ArrayVec::new(),
        }
    }
}

/// Chunk Iterator
pub struct ChunksExact<'a> {
    // data is stored as 1 chunk, but in fact contains multiple chunks
    data: Chunk<'a>,
    chunk_len: usize,
}

impl ChunksExact<'_> {
    /// Split the iterator into 2, with the first one having the specified length (in bytes).
    ///
    /// Panics if the length is greater than the iterator length.
    fn split(mut self, len: usize) -> (Self, Self) {
        // calculate length in bytes
        let len = len * self.chunk_len;

        if len < self.data.start.len() {
            // only self.0.start needs to be touched
            let start1 = self.data.start.drain(0..len).collect();

            let p1 = Self {
                data: Chunk::new(start1, &[], ArrayVec::new()),
                chunk_len: self.chunk_len,
            };

            return (p1, self);
        }

        // always have to take the full self.0.start()
        let len = len - self.data.start.len();

        let (p1, p2) = if len < self.data.middle.len() * BYTES_PER_USIZE {
            // need to split self.middle
            let split_idx = len / BYTES_PER_USIZE;
            let split_byte_idx = len % BYTES_PER_USIZE;

            if split_byte_idx == 0 {
                // clean split is possible
                let (part1, part2) = self.data.middle.split_at(split_idx);

                (
                    Chunk::new(self.data.start, part1, ArrayVec::new()),
                    Chunk::new(ArrayVec::new(), part2, self.data.end),
                )
            } else {
                // self.data cannot be split into full usize values, need to split a value
                let (part1, middle, part2) = {
                    let (part1, temp) = self.data.middle.split_at(split_idx);
                    let (&middle, part2) = temp.split_first().unwrap();
                    (part1, middle, part2)
                };

                // split the middle usize
                let middle = middle.to_be_bytes();
                let end = ArrayVec::from_iter(middle[0..split_byte_idx].iter().copied());
                let start = ArrayVec::from_iter(middle[split_byte_idx..].iter().copied());

                (
                    Chunk::new(self.data.start, part1, end),
                    Chunk::new(start, part2, self.data.end),
                )
            }
        } else {
            // have to take the full self.0.middle(), maybe some part of self.end()
            let len = len - self.data.middle.len() * BYTES_PER_USIZE;

            let Chunk {
                start,
                middle,
                mut end,
            } = self.data;

            let end1 = end.drain(0..len).collect();
            let start2 = end;

            (
                Chunk::new(start, middle, end1),
                Chunk::new(start2, &[], ArrayVec::new()),
            )
        };

        let p1 = Self {
            data: p1,
            chunk_len: self.chunk_len,
        };

        let p2 = Self {
            data: p2,
            chunk_len: self.chunk_len,
        };
        (p1, p2)
    }
}

impl<'a> Iterator for ChunksExact<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len() > 0 {
            // construct a default instance for mem::swap
            let mut this = Self {
                data: Chunk::new_empty(),
                chunk_len: 0,
            };

            mem::swap(&mut this, self);

            // this now contains self - split this and save the rest into self
            let data;
            (Self { data, .. }, *self) = this.split(1);
            debug_assert_eq!(data.len_byte(), self.chunk_len);
            Some(data)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }

    fn count(self) -> usize {
        self.len()
    }
}

impl ExactSizeIterator for ChunksExact<'_> {
    fn len(&self) -> usize {
        self.data.len_byte() / self.chunk_len
    }
}

impl DoubleEndedIterator for ChunksExact<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len() > 0 {
            // construct a default instance for mem::swap
            let mut this = Self {
                data: Chunk::new_empty(),
                chunk_len: 0,
            };

            mem::swap(&mut this, self);

            // this now contains self - split this and save the first part into self
            let data;
            (*self, Self { data, .. }) = this.split(self.len() - 1);
            debug_assert_eq!(data.len_byte(), self.chunk_len);
            Some(data)
        } else {
            None
        }
    }
}

/// Parallel Chunks Iterator
pub struct ParChunksExact<'a>(ChunksExact<'a>);

impl IndexedParallelIterator for ParChunksExact<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        bridge(self, consumer)
    }

    fn with_producer<CB: ProducerCallback<Self::Item>>(self, callback: CB) -> CB::Output {
        struct CurrentProducer<'b>(ChunksExact<'b>);

        impl<'b> Producer for CurrentProducer<'b> {
            type Item = Chunk<'b>;

            type IntoIter = ChunksExact<'b>;

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

impl<'a> ParallelIterator for ParChunksExact<'a> {
    type Item = Chunk<'a>;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }
}

// constructors on BitVec
impl BitVec {
    /// Returns an iterator that yields chunks of size_in_bytes bytes at a time.
    /// The chunk datatype is [Chunk].
    pub fn chunks_exact(&self, size_in_bytes: usize) -> ChunksExact {
        let (data, _) = self.as_full_slice();

        ChunksExact {
            data: Chunk {
                start: ArrayVec::new(),
                middle: data,
                end: ArrayVec::new(),
            },
            chunk_len: size_in_bytes,
        }
    }

    /// Returns a parallel iterator that yields chunks of size_in_bytes bytes at a time.
    /// The chunk datatype is [Chunk].
    pub fn par_chunks_exact(&self, size_in_bytes: usize) -> ParChunksExact {
        ParChunksExact(self.chunks_exact(size_in_bytes))
    }
}
