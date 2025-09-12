use std::vec;

use pyo3::{exceptions::{PyTypeError, PyValueError}, prelude::*, types::{PyBytes, PyTuple, PyType}, PyTypeInfo};
use rayon::prelude::*;
use splinter_rs::{Cut, Encodable, Optimizable, PartitionRead, PartitionWrite, Splinter, SplinterRef};

#[derive(Clone)]
pub enum SplinterType {
    Splinter,
    SplinterRef,
    CowSplinter,
}

/// A wrapper for higher-order functionality over the Splinter 
/// crate
#[pyclass(name="Splinter", module="splynters")]
#[derive(Clone)] 
pub struct SplinterWrapper(Splinter);

#[pymethods]
impl SplinterWrapper {
    #[new]
    pub fn __new__() -> Self {
        let splinter = Splinter::from_iter(std::iter::empty::<u32>());
        Self(splinter)
    }
    pub fn __len__(&self) -> usize { self.0.cardinality() }
    pub fn __sizeof__(&self) -> usize { self.0.encoded_size() }
    pub fn __repr__(&self) -> String {
        let s = format!("SplinterWrapper(len = {}, compressed_byte_size = {})", self.0.cardinality(), self.0.encoded_size());
        s
    }
    fn __iter__(&self) -> SplinterIter {
        SplinterIter {
            inner: self.0.iter().collect::<Vec<u32>>().into_iter(),
        }
    }

    fn __getitem__(&self, index: isize) -> PyResult<u32> {
        let len = self.0.cardinality();
        let mut actual_index = index;

        if actual_index < 0 {
            actual_index += len as isize;
        }

        match self.0.select(actual_index as usize) {
            Some(value) => Ok(value),
            None => Err(pyo3::exceptions::PyIndexError::new_err( "splinter index out of range"))
        }
    }

    #[staticmethod]
    /// Constructs a Splinter from an iterator of unsigned integers.
    ///
    /// Args:
    ///     data list[int]: The iterator from which to construct the Splinter.
    ///
    /// Returns: 
    ///     Splinter: A Splinter object constructed from the input items
    pub fn from_list(data: Vec<u32>) -> Self {
        // `pyo3` automatically converts the Python list into a `Vec<u32>`.
        // `Splinter::from_iter` can then consume the vector directly via `into_iter`
        let mut splinter = Splinter::from_iter(data);
        splinter.optimize();

        Self(splinter)
    }
    pub fn to_list(&self) -> Vec<u32> { self.0.iter().collect() }

    pub fn to_bytes(&self, py: Python) -> Py<PyBytes> {
        let bytes = self.0.encode_to_splinter_ref().into_inner();
        let py_bytes = PyBytes::new(py, &bytes);
        py_bytes.into()
    }

    #[classmethod]
    /// Constructs a Splinter from raw byte data.
    ///
    /// Panics: 
    ///     This method may cause a panic if the bytes are not formatted correctly.
    ///     splinter-rs provides a checksum in the serialized data to protect against 
    ///     corrupted or modified data, and will not cause any undefined behavior, but
    ///     malicious input can cause a panic.
    ///     Only use this method on trusted data.
    ///
    /// Args:
    ///     data array[byte]: The byte data from which to construct the Splinter.
    ///
    /// Returns: 
    ///     Splinter: A Splinter object, or else an error explaining why
    ///     construction failed.
    pub fn from_bytes(
        _cls: &Bound<'_, PyType>,
        data: &[u8],
    ) -> PyResult<Self> {

        let splinter_ref = SplinterRef::from_bytes(data).map_err(|e| {
            PyValueError::new_err(format!("Splinter could not be constructed from bytes: {e}"))
        })?;

        let splinter = splinter_ref.decode_to_splinter();

        Ok(Self(splinter))
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
    ///     or a list of booleans if the input was a list.
    pub fn contains(&self, value: &Bound<PyAny>) -> PyResult<BoolOrVec> {
        if let Ok(single_val) = value.extract::<u32>() {
            let result = self.0.contains(single_val);
            Ok(BoolOrVec::Bool(result))
        } else if let Ok(vals) = value.extract::<Vec<u32>>() {
            let results: Vec<bool> = vals.iter().map(|val| {
                self.0.contains(*val)
            }).collect();

            Ok(BoolOrVec::Vec(results))
        } else { 
            Err(PyTypeError::new_err(
                format!(
                    "contains() argument must be an integer or a list of integers, but received an object of type {:#?}", 
                    value.get_type().name()?
                )
            ))
        }
    }

    /// Checks if the bitmap contains multiple values in parallel.
    ///
    /// Note: 
    ///     This parallelized implementation introduces considerable overhead
    ///     compared to an individual check. It is not recommended to use
    ///     this unless you are checking for the presence of at least 
    ///     10,000 elements
    ///     
    /// Args:
    ///     values list[int]: The values values to check for.
    ///
    /// Returns:
    ///     list[bool]: A list of booleans.
    pub fn contains_many_parallel(
        &self, 
        values: Vec<u32>,
    ) -> Vec<bool> {
        values
            .par_iter()
            .map(|&val| self.0.contains(val))
            .collect()
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
        Ok(self.0.contains(value))
    }
    
    // mimicking python's syntax for sets, instead of lists

    /// Inserts a value into the Splinter
    ///
    /// This method is overloaded. It can accept either a single integer or an
    /// iterable of integers.
    ///
    /// Args:
    ///     values (int | list[int]): The value or values to check for.
    pub fn add(&mut self, values: &Bound<PyAny>) -> PyResult<()> {
        if let Ok(val) = values.extract::<u32>() {
            self.0.insert(val);
            self.0.optimize();
            Ok(())
        } else if let Ok(vals) = values.extract::<Vec<u32>>() {
            vals.iter().for_each(|val| {
                self.0.insert(*val);
            });
            self.0.optimize();
            Ok(())
        } else {
            Err(PyTypeError::new_err(
                format!(
                    "discard() argument must be an integer or a list of integers, but received an object of type {:#?}", 
                    values.get_type().name()?
                )
            ))
        }
    }

    /// Removes a value into the Splinter and returns an error if the value is missing.
    ///
    /// This method is overloaded. It can accept either a single integer or an
    /// iterable of integers.
    ///
    /// Args:
    ///     values (int | list[int]): The value or values to check for.
    pub fn remove(&mut self, value: &Bound<PyAny>) -> PyResult<()> {
        if let Ok(single_val) = value.extract::<u32>() {
            if !self.0.remove(single_val) {
                Err(pyo3::exceptions::PyKeyError::new_err(
                    format!(
                        "remove() could not find the key {single_val} in the splinter. For a fault-tolerant alternative to remove(), consider discard()"
                    )
                ))
            } else {
                // if we get to this point, the operation completed successfully, and we optimize
                self.0.optimize();
                Ok(())
            }
        } else if let Ok(vals) = value.extract::<Vec<u32>>() {
            for val in &vals {
                // check to see that all values are actually present: don't mutate anything unless
                // we know the entire transaction would be successful
                // for a version of this operation which is fault tolerant, discard is the choice
                if !self.0.contains(*val) {
                    return Err(pyo3::exceptions::PyKeyError::new_err(
                        format!(
                            "remove() could not find the key {val} in the splinter.\nFor a fault-tolerant alternative to remove(), consider discard()"
                        )
                    ));
                }
            }
            // actually remove them
            vals.iter().for_each(|val| {
                self.0.remove(*val);
            });
            // if we ge to this point, the operation completed successfully
            self.0.optimize();
            Ok(())
        } else { 
            Err(PyTypeError::new_err(
                format!(
                    "discard() argument must be an integer or a list of integers, but received an object of type {:#?}", 
                    value.get_type().name()?
                )
            ))
        }
    }

    /// Removes a value into the Splinter, or does nothing if the value is missing.
    ///
    /// This method is overloaded. It can accept either a single integer or an
    /// iterable of integers.
    ///
    /// Args:
    ///     values (int | list[int]): The value or values to check for.
    pub fn discard(&mut self, value: &Bound<PyAny>) -> PyResult<()> {
        if let Ok(single_val) = value.extract::<u32>() {
            self.0.remove(single_val);
            self.0.optimize();
            Ok(())
        } else if let Ok(vals) = value.extract::<Vec<u32>>() {
            vals.iter().for_each(|val| {
                self.0.remove(*val);
            });
            self.0.optimize();
            Ok(())
        } else { 
            Err(PyTypeError::new_err(
                format!(
                    "discard() argument must be an integer or a list of integers, but received an object of type {:#?}", 
                    value.get_type().name()?
                )
            ))
        }
    }

    /// Merges two or more splinters together
    ///
    /// This method is overloaded. It can accept either a single Splinter or an
    /// iterable of Splinters.
    ///
    /// Args:
    ///     splinters (Splinter | list[Splinter]): The object or objects to merge with
    pub fn merge(&mut self, splinters: &Bound<PyAny>) -> PyResult<()> {
        if let Ok(rhs) = splinters.extract::<SplinterWrapper>() {
            self.0 |= &rhs.0;
            self.0.optimize();
            Ok(())
        } else if let Ok(splinter_list) = splinters.extract::<Vec<SplinterWrapper>>() {
            // is this kosher? likely a more effective way to do this, right??
            for rhs in splinter_list {
                self.0 |= &rhs.0;
            };
            self.0.optimize();
            Ok(())
        } else {
            Err(PyTypeError::new_err(
                format!(
                    "merge() argument must be a Splinter or a list of Splinters, but received an object of type {:#?}", 
                    splinters.get_type().name()?
                )
            ))
        }
    }

    // for cut, not currently enabling multiple sequential cuts, since it's not clear what the
    // behavior on this is, and don't want to give the user a knife to cut themselves with

    /// Removes and returns the intersection between self and splinter.
    ///
    /// If self and splinter have no overlap, it returns an empty Splinter and does not modify
    /// self. Otherwise, any elements in common between the two will be removed from self and
    /// returned to the caller.
    ///
    /// Args:
    ///     splinter Splinter: A Splinter object to intersect with
    ///
    /// Returns: 
    ///     Splinter
    pub fn cut(&mut self, rhs: SplinterWrapper) -> Self { Self(self.0.cut(&rhs.0)) }

    /// Returns the number of elements in the Splinter that are less than or equal to the given
    /// value.
    ///
    /// Args:
    ///     value int: the value to compare against. Cannot be negative
    ///
    /// Returns:
    ///     int: an integer indicating the number of elements less than or equalt to the given
    ///     value
    pub fn rank(&self, value: u32) -> usize { self.0.rank(value) }

    // sugar over this to allow selecting using the [] notation, including negative indices??

    /// Returns the element at the given index in the sorted sequence, or None if it is out of
    /// bounds.
    ///
    /// Args: 
    ///     idx int: the index of the sequence to grab. Negative indices count back
    ///     from the end.
    ///
    /// Returns: 
    ///     (int | None): the element at the relevant index, or else a None if overflowed
    pub fn select(&self, idx: &Bound<PyAny>) -> PyResult<Option<u32>> {
        if let Ok(val) = idx.extract::<usize>() {
            Ok(self.0.select(val))
        } else if let Ok(val) = idx.extract::<isize>() {
            if let Some(index) = self.0.cardinality().checked_sub(val as usize) {
                Ok(self.0.select(index))
            } else { Ok(None) }
        } else {
            Err(PyTypeError::new_err(
                format!(
                    "select() argument must be an integer, but received an object of type {:#?}",
                    idx.get_type().name()?
                )
            ))
        }
    }
    
    pub fn position(&self, value: u32) -> PyResult<usize> {
        if let Some(pos) = self.0.position(value) {
            Ok(pos)
        } else {
            Err(PyValueError::new_err(format!("element {value} does not exist in this Splinter")))
        }

    }

    // basic bitwise set operators
    fn __and__(&self, rhs: &Self) -> Self { Self(&self.0 & &rhs.0) }
    fn __or__(&self, rhs: &Self) -> Self { Self(&self.0 | &rhs.0) }
    fn __xor__(&self, rhs: &Self) -> Self { Self(&self.0 ^ &rhs.0) }
    fn __sub__(&self, rhs: &Self) -> Self { Self(&self.0 - &rhs.0) }

    // reverse bitwise set operators, for completeness
    fn __rand__(&self, rhs: &Self) -> Self { Self(&self.0 & &rhs.0) }
    fn __ror__(&self, rhs: &Self) -> Self { Self(&self.0 | &rhs.0) }
    fn __rxor__(&self, rhs: &Self) -> Self { Self(&self.0 ^ &rhs.0) }
    fn __rsub__(&self, rhs: &Self) -> Self { Self(&self.0 - &rhs.0) }

    // assign bitwise set operators
    fn __iand__(&mut self, rhs: &Self) { self.0 &= &rhs.0 }
    fn __ior__(&mut self, rhs: &Self) { self.0 |= &rhs.0 }
    fn __ixor__(&mut self, rhs: &Self) { self.0 ^= &rhs.0 }
    fn __isub__(&mut self, rhs: &Self) { self.0 -= &rhs.0 }

    // set comparison operations
    fn __eq__(&self, rhs: &Self) -> bool { self.0 == rhs.0 }
    fn __ne__(&self, rhs: &Self) -> bool { self.0 != rhs.0 }
    fn __le__(&self, rhs: &Self) -> bool { (&self.0 & &rhs.0) == self.0 }
    fn __lt__(&self, rhs: &Self) -> bool { (self.0.cardinality() < rhs.0.cardinality()) && self.__le__(rhs) }
    fn __ge__(&self, rhs: &Self) -> bool { (&self.0 & &rhs.0) == rhs.0 }
    fn __gt__(&self, rhs: &Self) -> bool { self.0.cardinality() > rhs.0.cardinality() &&  self.__ge__(rhs) }

    // for serialization with pickle
    fn __getstate__(&self, py: Python) -> PyObject {
        let bytes = self.to_bytes(py);
        bytes.into()
    }
    // for deserializing from pickle
    fn __setstate__(&mut self, bytes: &[u8]) -> PyResult<()> {
        let splinter_ref = SplinterRef::from_bytes(bytes).map_err(|e| {
            PyValueError::new_err(format!("Failed to deserialize Splinter from bytes: {e}"))
        })?;

        let new_splinter = splinter_ref.decode_to_splinter();

        self.0 = new_splinter;
        Ok(())
    }

    /// tells pickle how to find the class and serialize it
    fn __reduce__<'py>(&self, py: Python<'py>,) -> (PyObject, PyObject, PyObject) {
        let class = Self::type_object(py).into();
        let args = PyTuple::empty(py).into();
        let state = self.__getstate__(py);
        (class, args, state)
    }

    // copy protocol
    fn copy(&self) -> Self { self.clone() }
    // making it easily available from python
    fn __copy__(&self) -> Self { self.clone() }


    // explicit set methods
    // omitting the usual snake_case _ to more closely fit the Python idiom

    /// Returns true if self and rhs have no overlap, and false otherwise.
    ///
    /// This is an explicit implementation of (self & rhs).is_empty().
    ///
    /// Args:
    ///     rhs Splinter: a Splinter object to compare against   
    ///
    /// Returns:
    ///     bool: true if there is no overlap, false otherwise
    fn isdisjoint(&self, rhs: &Self) -> bool { (&self.0 & &rhs.0).is_empty() }

    /// Returns true if self is a subset of rhs, and false otherwise.
    ///
    /// This is an explicit implementation of (self & rhs) == self.
    ///
    /// Args:
    ///     rhs Splinter: a Splinter object to compare against   
    ///
    /// Returns:
    ///     bool: true if self is a subset of rhs, false otherwise
    fn issubset(&self, rhs: &Self) -> bool { self.__le__(rhs) }
    
    /// Returns true if self is a superset of rhs, and false otherwise.
    ///
    /// This is an explicit implementation of (self & rhs) == rhs.
    ///
    /// Args:
    ///     rhs Splinter: a Splinter object to compare against   
    ///
    /// Returns:
    ///     bool: true if self is a subset of rhs, false otherwise
    fn issuperset(&self, rhs: &Self) -> bool { self.__ge__(rhs) }

    // todo: consolidate this with merge???

    /// Returns the union of one or more Splinters
    ///
    /// Args:
    ///     rhs list[Splinter]: an iterable of one or more Splinters to combine
    ///
    /// Returns:
    ///     Splinter: a combined splinter made up of the union of all provided values
    #[pyo3(signature = (*rhs))]
    fn union(&self, rhs: &Bound<PyTuple>) -> PyResult<Self> {
        let mut result = self.0.clone();
        for other in rhs.iter() {
            let other_splinter = other.extract::<PyRef<Self>>()?;
            result |= &other_splinter.0;
        }
        Ok(Self(result))
    }

    /// Returns the intersection of one or more Splinters
    ///
    /// Args:
    ///     rhs: list[Splinter]: an iterable of one or more Splinters to combine
    ///
    /// Return: 
    ///     Splinter: a combined splinter made up of the intersection of all provided values
    #[pyo3(signature = (*rhs))]
    fn intersection(&self, rhs: &Bound<PyTuple>) -> PyResult<Self> { 
        let mut result = self.0.clone();
        for other in rhs.iter() {
            let other_splinter = other.extract::<PyRef<Self>>()?;
            result &= &other_splinter.0;
        }
        Ok(Self(result))
    }
}

/// Iterator class to implement __iter__ on SplinterWrapper
#[pyclass(name = "SplinterIter")]
struct SplinterIter {
    inner: vec::IntoIter<u32>
}

#[pymethods]
impl SplinterIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> { slf }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<u32> { slf.inner.next() }
}

// as of new patch notes, it's just more straightforward to define an enum for varying outputs
#[derive(IntoPyObject)]
pub enum BoolOrVec {
    Bool(bool),
    Vec(Vec<bool>),
}

#[pymodule]
fn splynters(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SplinterWrapper>()?;
    m.add_class::<SplinterIter>()?;
    Ok(())
}
