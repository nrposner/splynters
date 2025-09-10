use std::vec;

use pyo3::{exceptions::PyValueError, prelude::*, types::{PyBytes, PyType}};
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
#[pyclass(name="Splinter")]
#[derive(Clone)] 
pub struct SplinterWrapper(Splinter);

#[pymethods]
impl SplinterWrapper {
    #[new]
    pub fn __new__() -> Self {
        let splinter = Splinter::from_iter(std::iter::empty::<u32>());

        Self(splinter)
    }
    #[staticmethod]
    pub fn from_list(data: Vec<u32>) -> Self {
        // `pyo3` automatically converts the Python list into a `Vec<u32>`.
        // `Splinter::from_iter` can then consume the vector directly via `into_iter`.
        let splinter = Splinter::from_iter(data);

        Self(splinter)
    }
    pub fn to_list(&self) -> Vec<u32> { self.0.iter().collect() }

    pub fn to_bytes(&mut self, py: Python) -> PyResult<Py<PyBytes>> {
        // optimize before serializing
        self.0.optimize();
        let bytes = self.0.encode_to_splinter_ref().into_inner();
        let py_bytes = PyBytes::new(py, &bytes);
        Ok( py_bytes.into() )
    }
    pub fn __len__(&self) -> usize {
        self.0.cardinality()
    }
    pub fn __sizeof__(&self) -> usize {
        self.0.encoded_size()
    }
    pub fn __repr__(&self) -> String {
        let s = format!("SplinterWrapper(len = {}, compressed_byte_size = {})", self.0.cardinality(), self.0.encoded_size());
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
    ///                         or a list of booleans if the input was a list.
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
            Err(pyo3::exceptions::PyTypeError::new_err(
                format!(
                    "contains() argument must be an integer or a list of integers, but received an object of type {:#?}", 
                    value.get_type().name()?
                )
            ))
        }
    }

    pub fn contains_many_parallel(
        &self, 
        values: &Bound<PyAny>
    ) -> PyResult<Vec<bool>> {
        if let Ok(vals) = values.extract::<Vec<u32>>() {
            let res = vals
                .par_iter()
                .map(|&val| self.0.contains(val))
                .collect();
            Ok(res)
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                format!(
                    "contains_many_parallel() argument must be a list of integers, but received an object of type {:#?}", 
                    values.get_type().name()?
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
        Ok(self.0.contains(value))
    }
    
    // mimicking python's syntax for sets, instead of lists

    pub fn add(&mut self, values: &Bound<PyAny>) -> PyResult<()> {

        if let Ok(val) = values.extract::<u32>() {
            self.0.insert(val);
            Ok(())
        } else if let Ok(vals) = values.extract::<Vec<u32>>() {
            vals.iter().for_each(|val| {
                self.0.insert(*val);
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
            Err(pyo3::exceptions::PyTypeError::new_err(
                format!(
                    "discard() argument must be an integer or a list of integers, but received an object of type {:#?}", 
                    value.get_type().name()?
                )
            ))
        }
    }



    // pub fn merge(&mut self, splinters: &Bound<PyAny>) -> PyResult<()> {
    //     if let Ok(rhs) = splinters.extract::<SplinterWrapper>() {
    //         self.0.merge(&rhs.0);
    //         Ok(())
    //     } else if let Ok(splinter_list) = splinters.extract::<Vec<SplinterWrapper>>() {
    //         // is this kosher? likely a more effective way to do this, right??
    //         for rhs in splinter_list {
    //             self.0.merge(&rhs.0);
    //         };
    //         Ok(())
    //     } else {
    //         Err(pyo3::exceptions::PyTypeError::new_err(
    //             format!(
    //                 "merge() argument must be a Splinter or a list of Splinters, but received an object of type {:#?}", 
    //                 splinters.get_type().name()?
    //             )
    //         ))
    //     }
    // }

    // for cut, not currently enabling multiple sequential cuts, since it's not clear what the
    // behavior on this is, and don't want to give the user a knife to cut themselves with
    pub fn cut(&mut self, splinters: &Bound<PyAny>) -> PyResult<Self> {
        if let Ok(rhs) = splinters.extract::<SplinterWrapper>() {
            let splinter = self.0.cut(&rhs.0);

            Ok(Self(splinter))
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
            Ok(self.0.rank(val))
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
            Ok(self.0.select(val))
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                format!(
                    "select() argument must be an unsigned integer, but received an object of type {:#?}",
                    value.get_type().name()?
                )
            ))
        }
    }

    // implementing support for bitwise operations using Python's standard 
    // operators (via dunder class methods)
    // this implementation should accomplish the goal without intermediate copying, 
    // but check with Carl
    pub fn __and__(&self, rhs: Self) -> Self { Self(&self.0 & &rhs.0) }
    pub fn __or__(&self, rhs: Self) -> Self { Self(&self.0 | &rhs.0) }
    pub fn __xor__(&self, rhs: Self) -> Self { Self(&self.0 ^ &rhs.0) }
    pub fn __sub__(&self, rhs: Self) -> Self { Self(&self.0 - &rhs.0) }

    // are these redundant? implement them anyway
    pub fn __rand__(&self, rhs: Self) -> Self { Self(&self.0 & &rhs.0) }
    pub fn __ror__(&self, rhs: Self) -> Self { Self(&self.0 | &rhs.0) }
    pub fn __rxor__(&self, rhs: Self) -> Self { Self(&self.0 ^ &rhs.0) }
    pub fn __rsub__(&self, rhs: Self) -> Self { Self(&self.0 - &rhs.0) }

    pub fn __iand__(&mut self, rhs: Self) { self.0 &= &rhs.0 }
    pub fn __ior__(&mut self, rhs: Self) { self.0 |= &rhs.0 }
    pub fn __ixor__(&mut self, rhs: Self) { self.0 ^= &rhs.0 }
    pub fn __isub__(&mut self, rhs: Self) { self.0 -= &rhs.0 }
    
    fn __iter__(&self) -> SplinterIter {
        SplinterIter {
            inner: self.0.iter().collect::<Vec<u32>>().into_iter(),
        }
    }
}

#[pyclass(name = "SplinterIter")]
struct SplinterIter {
    inner: vec::IntoIter<u32>
}

#[pymethods]
impl SplinterIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<u32> {
        slf.inner.next()
    }
}

// pain in my goddamn ass new patch notes deprecating into_py making me define a return enum for
// bools
#[derive(IntoPyObject)]
pub enum BoolOrVec {
    Bool(bool),
    Vec(Vec<bool>),
}


/// A Python module implemented in Rust.
#[pymodule]
fn splynters(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SplinterWrapper>()?;
    m.add_class::<SplinterIter>()?;
    Ok(())
}
