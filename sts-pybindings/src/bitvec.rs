use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use std::borrow::Cow;
use std::sync::Arc;
use sts_lib::bitvec;

/// A list of bits, tightly packed - used as the data type for all tests.
///
/// Note: `len(self)` returns the count of bits stored in the object.
#[pyclass(frozen)]
pub struct BitVec(pub(crate) Arc<bitvec::BitVec>);

#[pymethods]
impl BitVec {
    /// Creates a new instance.
    ///
    /// ## Arguments
    ///
    /// * `data`: either a list of bytes, a list of bits, or a string.
    ///   If it is a string, '0' will be mapped to bit 0 and '1' will be mapped to bit 1.
    /// * `lossy`: only has an effect if the `data` argument is a string. If `True`, values other
    ///   than '0' or '1' are ignored. If `False`, any character other than '0' or '1' will raise
    ///   an exception. Default value: `False`.
    /// * `max_length`: the maximum count of bits to read from the given data. Not set by default.
    #[new]
    #[pyo3(signature = (data, lossy=false, max_length=None))]
    pub fn new(data: &Bound<'_, PyAny>, lossy: bool, max_length: Option<usize>) -> PyResult<Self> {
        // create the vec dynamically, based on the type given.
        if let Ok(byte_list) = data.extract::<Vec<u8>>() {
            // from byte list, lossy makes no sense in this context.
            let mut bit_vec = bitvec::BitVec::from(byte_list);

            if let Some(max_length) = max_length {
                bit_vec.crop(max_length)
            }

            Ok(Self(Arc::new(bit_vec)))
        } else if let Ok(bit_list) = data.extract::<Vec<bool>>() {
            // from bit list, lossy makes no sense in this context.
            let mut bit_vec = bitvec::BitVec::from(bit_list);

            if let Some(max_length) = max_length {
                bit_vec.crop(max_length)
            }

            Ok(Self(Arc::new(bit_vec)))
        } else if let Ok(ascii) = data.extract::<Cow<'_, str>>() {
            // from string
            if lossy {
                // lossy: builtin option for max length
                match max_length {
                    Some(max_length) => Ok(Self(Arc::new(
                        bitvec::BitVec::from_ascii_str_lossy_with_max_length(
                            ascii.as_ref(),
                            max_length,
                        ),
                    ))),
                    None => Ok(Self(Arc::new(bitvec::BitVec::from_ascii_str_lossy(
                        ascii.as_ref(),
                    )))),
                }
            } else {
                // not lossy: have to check the length manually
                let bit_vec = bitvec::BitVec::from_ascii_str(ascii.as_ref());

                let mut bit_vec = match bit_vec {
                    Some(bit_vec) => bit_vec,
                    None => return Err(PyValueError::new_err("Given string contains an other character than '0' or '1', but lossy=True was not specified"))
                };

                if let Some(max_length) = max_length {
                    bit_vec.crop(max_length)
                }

                Ok(Self(Arc::new(bit_vec)))
            }
        } else {
            // unsupported
            Err(PyTypeError::new_err(
                "Only strings, list of bytes and lists of bits are supported.",
            ))
        }
    }

    /// Returns the length in bit of the BitVec.
    pub fn __len__(&self) -> usize {
        self.0.len_bit()
    }

    /// Returns a new object that contains the same data, but cropped to the given count of bits.
    /// If the given new length is greater than the old one, the data of the new object is
    /// unchanged.
    pub fn crop(&self, new_bit_len: usize) -> Self {
        let mut this = self.0.as_ref().clone();
        this.crop(new_bit_len);
        Self(Arc::new(this))
    }

    // string representation.
    pub fn __str__(&self) -> String {
        format!("BitVec(length={})", self.0.len_bit())
    }
}
