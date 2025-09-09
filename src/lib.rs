use bytes::{Bytes, BytesMut};

use pyo3::{exceptions::PyValueError, prelude::*, types::{PyBytes, PyType}};
use splinter_rs::{Cut, Encodable, Merge, Optimizable, PartitionRead, PartitionWrite, Splinter, SplinterRef};

#[derive(Clone)]
pub enum SplinterType {
    Splinter,
    SplinterRef,
    CowSplinter,
}

/// A wrapper for higher-order functionality over the Splinter 
/// crate
#[pyclass(name="Splinter")]
#[derive(Clone)] // 
pub struct SplinterWrapper {
    pub splinter_type: SplinterType,
    pub splinter: Splinter,
}

#[pymethods]
impl SplinterWrapper {
    #[new]
    pub fn __new__() -> Self {
        let splinter = Splinter::from_iter(std::iter::empty::<u32>());

        Self {
            splinter_type: SplinterType::Splinter,
            splinter,
        }
    }
    #[staticmethod]
    pub fn from_list(data: Vec<u32>) -> Self {
        // `pyo3` automatically converts the Python list into a `Vec<u32>`.
        // `Splinter::from_iter` can then consume the vector directly via `into_iter`.
        let splinter = Splinter::from_iter(data);

        Self {
            splinter_type: SplinterType::Splinter,
            splinter,
        }
    }
    pub fn to_bytes(&mut self, py: Python) -> PyResult<Py<PyBytes>> {
        // optimize before serializing
        self.splinter.optimize();
        let bytes = self.splinter.encode_to_splinter_ref().into_inner();
        let py_bytes = PyBytes::new(py, &bytes);
        Ok( py_bytes.into() )
    }
    pub fn __len__(&self) -> usize {
        self.splinter.cardinality()
    }
    pub fn __repr__(&self) -> String {
        let s = format!("SplinterWrapper(len = {}, compressed_byte_size = {})", self.splinter.cardinality(), self.splinter.encoded_size());
        s
    }
    // not going to implement this at present, as it's a feature that can potentially lead to
    // errors and we don't want to make the wrong thing too easy
    #[classmethod]
    pub fn from_bytes(
        _cls: &Bound<'_, PyType>,
        data: &[u8],
    ) -> PyResult<Self> {

        let splinter_ref = SplinterRef::from_bytes(data).map_err(|e| {
            PyValueError::new_err(format!("Splinter could not be constructed from bytes: {e}"))
        })?;

        let splinter = splinter_ref.decode_to_splinter();

        Ok(Self {
            splinter_type: SplinterType::Splinter,
            splinter,
        })
    }

    /// Checks if the bitmap contains a single value or multiple values.
    ///
    /// This method is overloaded. It can accept either a single integer or an
    /// iterable of integers.
    ///
    /// Args:
    ///     value (int | list[int]): The value or values to check for.
    ///
    /// Returns:
    ///     bool | list[bool]: A single boolean if the input was a single integer,
    ///                         or a list of booleans if the input was a list.
    pub fn contains(&self, value: &Bound<PyAny>) -> PyResult<BoolOrVec> {
        if let Ok(single_val) = value.extract::<u32>() {
            let result = self.splinter.contains(single_val);
            Ok(BoolOrVec::Bool(result))
        } else if let Ok(vals) = value.extract::<Vec<u32>>() {
            let results: Vec<bool> = vals.iter().map(|val| {
                self.splinter.contains(*val)
            }).collect();

            Ok(BoolOrVec::Vec(results))
        } else { 
            Err(pyo3::exceptions::PyTypeError::new_err(
                format!(
                    "contains() argument must be an integer or a list of integers, but received an object of type {:#?}", 
                    value.get_type().name()?
                )
            ))
        }
    }
    /// Implements the Python 'in' operator for checking a single value.
    ///
    /// This allows for pythonic checks like `if 5 in splinter:`.
    ///
    /// Args:
    ///     value (int): The value to check for.
    ///
    /// Returns:
    ///     bool: True if the value is present, False otherwise.
    fn __contains__(&self, value: u32) -> PyResult<bool> {
        Ok(self.splinter.contains(value))
    }
    
    // mimicking python's syntax for sets, instead of lists

    pub fn add(&mut self, values: &Bound<PyAny>) -> PyResult<()> {

        if let Ok(val) = values.extract::<u32>() {
            self.splinter.insert(val);
            Ok(())
        } else if let Ok(vals) = values.extract::<Vec<u32>>() {
            vals.iter().for_each(|val| {
                self.splinter.insert(*val);
            });
            Ok(())
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                format!(
                    "discard() argument must be an integer or a list of integers, but received an object of type {:#?}", 
                    values.get_type().name()?
                )
            ))
        }
    }

    pub fn remove(&mut self, value: &Bound<PyAny>) -> PyResult<()> {
        if let Ok(single_val) = value.extract::<u32>() {
            if !self.splinter.remove(single_val) {
                Err(pyo3::exceptions::PyKeyError::new_err(
                    format!(
                        "remove() could not find the key {single_val} in the splinter. For a fault-tolerant alternative to remove(), consider discard()"
                    )
                ))
            } else {
                // if we get to this point, the operation completed successfully, and we optimize
                self.splinter.optimize();
                Ok(())
            }
        } else if let Ok(vals) = value.extract::<Vec<u32>>() {
            for val in &vals {
                // check to see that all values are actually present: don't mutate anything unless
                // we know the entire transaction would be successful
                // for a version of this operation which is fault tolerant, discard is the choice
                if !self.splinter.contains(*val) {
                    return Err(pyo3::exceptions::PyKeyError::new_err(
                        format!(
                            "remove() could not find the key {val} in the splinter.\nFor a fault-tolerant alternative to remove(), consider discard()"
                        )
                    ));
                }
            }
            // actually remove them
            vals.iter().for_each(|val| {
                self.splinter.remove(*val);
            });
            // if we ge to this point, the operation completed successfully
            self.splinter.optimize();
            Ok(())
        } else { 
            Err(pyo3::exceptions::PyTypeError::new_err(
                format!(
                    "discard() argument must be an integer or a list of integers, but received an object of type {:#?}", 
                    value.get_type().name()?
                )
            ))
        }
    }


    //for discard, we don' return a bool or raise an error on incorrect removal

    pub fn discard(&mut self, value: &Bound<PyAny>) -> PyResult<()> {
        if let Ok(single_val) = value.extract::<u32>() {
            self.splinter.remove(single_val);
            self.splinter.optimize();
            Ok(())
        } else if let Ok(vals) = value.extract::<Vec<u32>>() {
            vals.iter().for_each(|val| {
                self.splinter.remove(*val);
            });
            self.splinter.optimize();
            Ok(())
        } else { 
            Err(pyo3::exceptions::PyTypeError::new_err(
                format!(
                    "discard() argument must be an integer or a list of integers, but received an object of type {:#?}", 
                    value.get_type().name()?
                )
            ))
        }
    }


    pub fn merge(&mut self, splinters: &Bound<PyAny>) -> PyResult<()> {
        if let Ok(rhs) = splinters.extract::<SplinterWrapper>() {
            self.splinter.merge(&rhs.splinter);
            Ok(())
        } else if let Ok(splinter_list) = splinters.extract::<Vec<SplinterWrapper>>() {
            // is this kosher? likely a more effective way to do this, right??
            for rhs in splinter_list {
                self.splinter.merge(&rhs.splinter);
            };
            Ok(())
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                format!(
                    "merge() argument must be a Splinter or a list of Splinters, but received an object of type {:#?}", 
                    splinters.get_type().name()?
                )
            ))
        }
    }

    // for cut, not currently enabling multiple sequential cuts, since it's not clear what the
    // behavior on this is, and don't want to give the user a knife to cut themselves with
    pub fn cut(&mut self, splinters: &Bound<PyAny>) -> PyResult<Self> {
        if let Ok(rhs) = splinters.extract::<SplinterWrapper>() {
            let splinter = self.splinter.cut(&rhs.splinter);

            Ok(Self {
                splinter_type: SplinterType::Splinter,
                splinter
            })
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                format!(
                    "cut() argument must be a Splinter, but received an object of type {:#?}",
                    splinters.get_type().name()?
                )
            ))
        }
    }

    pub fn rank(&self, value: &Bound<PyAny>) -> PyResult<usize> {
        if let Ok(val) = value.extract::<u32>() {
            Ok(self.splinter.rank(val))
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                format!(
                    "rank() argument must be an unsigned integer, but received an object of type {:#?}",
                    value.get_type().name()?
                )
            ))
        }
    }

    pub fn select(&self, value: &Bound<PyAny>) -> PyResult<Option<u32>> {
        if let Ok(val) = value.extract::<usize>() {
            Ok(self.splinter.select(val))
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                format!(
                    "select() argument must be an unsigned integer, but received an object of type {:#?}",
                    value.get_type().name()?
                )
            ))
        }
    }

    // select() may be useful as well??
}

// pain in my goddamn ass new patch notes deprecating into_py making me define a return enum for
// bools
#[derive(IntoPyObject)]
pub enum BoolOrVec {
    Bool(bool),
    Vec(Vec<bool>),
}





impl SplinterWrapper {
    pub fn new(bytes: Bytes) -> Self {
        SplinterWrapper {
            splinter_type: SplinterType::Splinter,
            splinter: Splinter::from_iter(bytes.chunks_exact(4).map(|chunk| u32::from_ne_bytes(chunk.try_into().unwrap()))),
        }
    }

    /// want to mutate the contained splinter type, from a Splinter to a SplinterRef
    /// and return a boolean to tell us it was done correctly? the underlying operation can't fail,
    /// but we could call it when there is already a SplinterRef: maybe we don't return a bool,
    /// just check the type, if invalid, do nothing, maybe return a runtime exception?
    pub fn freeze<B>(&mut self) -> Result<Bytes, String> {
        match self.splinter_type {
            SplinterType::Splinter => {
                let bytes = self.splinter.encode_to_splinter_ref();
                self.splinter_type = SplinterType::SplinterRef;
                Ok(bytes.into_inner())
            },
            _ => Err("bla".to_string()) // a runtime exception later
        }
    }

    /// a similar function for reversing this process
    pub fn unfreeze(&mut self) -> bool { true }

    /// sugar over encoded_size to provide Python the .size method
    pub fn size(&self) -> usize {
        self.splinter.encoded_size()
    }
    
    /// returning the cardinality, which for our purposes will be presented as
    /// a unidimensional tuple usize. If we add support for multiple bitmaps, we should update this
    /// actually, that might not make any sense: .shape is used for N dimensions, with the
    /// assumption that the contents of each dimension are the same size, like a matrix, not like a
    /// random collection of splinters of different sizes.
    /// Maybe we want to create a different wrapper type for grids of splinters
    pub fn shape(&self) -> usize {
        
        0
    }

    // a higher-order wrapper for merging two splinter wrappers
    // maybe we even want to provide support for merging larger numbers of splinters from an
    // iterator: maybe this is a parallelism use case, group them up and do a few of the merges in
    // parallel, then only optimize once at the very end
    // pub fn merge(&mut self, _rhs: Self) {}
    //
    // pub fn merge_many(&mut self, _rhs: &[Self]) {}
    //
        // maybe additional support to allow `foo = merge([bar, bla, glorb])` in addition to
    // `foo.merge([bar, bla, glorb])`??


    // functions sugaring over bit-twiddling hacks??
    // provide some sugar over this so that it can be used as `s1 or s2` instead of always needing
    // the full function call??



    pub fn and_simple(&mut self, mut rhs: Self) -> Result<Bytes, String> {

        let a = self.freeze::<Bytes>()?;
        let b = rhs.freeze::<Bytes>()?;

        let result: Vec<u8> = a.iter()
            .zip(b.iter())
            .map(|(&byte_a, &byte_b)| byte_a & byte_b)
            .collect();

        Ok(Bytes::from(result))
    }

    pub fn and_chunked(&mut self, mut rhs: Self) -> Result<Bytes, String> {
        let a = self.freeze::<Bytes>()?;
        let b = rhs.freeze::<Bytes>()?;

        let len = a.len().min(b.len());
        let mut result = Vec::with_capacity(len);

        let chunk_size = std::mem::size_of::<u64>();
        let num_chunks = len / chunk_size;

        // Process the bulk of the data in u64 chunks for performance.
        // This is safe because we ensure we don't read past the end of the slices.
        for i in 0..num_chunks {
            let offset = i * chunk_size;
            
            // Read u64s directly from the byte slices.
            // The `from_ne_bytes` (native-endian) is typically fastest.
            let chunk_a = u64::from_ne_bytes(a[offset..offset + chunk_size].try_into().unwrap());
            let chunk_b = u64::from_ne_bytes(b[offset..offset + chunk_size].try_into().unwrap());
            
            let anded_chunk = chunk_a & chunk_b;
            
            result.extend_from_slice(&anded_chunk.to_ne_bytes());
        }

        // Handle any remaining bytes that didn't fit into a full chunk.
        let remainder_offset = num_chunks * chunk_size;
        for i in remainder_offset..len {
            result.push(a[i] & b[i]);
        }

        Ok(Bytes::from(result))
    }


    // simple version, enhance with chunking
    // or make both, and properly benchmark them
    pub fn or_simple(&mut self, mut rhs: Self) -> Result<Bytes, String> {
        let a = self.freeze::<Bytes>()?;
        let b = rhs.freeze::<Bytes>()?;


        let len_a = a.len();
        let len_b = b.len();
        let min_len = len_a.min(len_b);
        let max_len = len_a.max(len_b);

        let mut result = BytesMut::with_capacity(max_len);

        // Perform OR on the common part
        for i in 0..min_len {
            result.extend_from_slice(&[a[i] | b[i]]);
        }

        // Append the remainder from the longer slice
        if len_a > len_b {
            result.extend_from_slice(&a[min_len..]);
        } else if len_b > len_a {
            result.extend_from_slice(&b[min_len..]);
        }

        Ok(result.freeze())
    }

    pub fn xor(&self, _rhs: Self) {}
}

// the splinters crate itself doesn't seem to provide native support for bitwise operations
// through its api??


// sketch out the sugar and methods we want to provide

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn splynters(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}
