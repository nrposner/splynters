use bytes::{Bytes, BytesMut};

use pyo3::prelude::*;
use splinter_rs::{Splinter, Encodable};

#[derive(Clone)]
pub enum SplinterType {
    Splinter,
    SplinterRef,
    CowSplinter,
}

/// A wrapper for higher-order functionality over the Splinter 
/// crate
#[pyclass]
#[derive(Clone)]
pub struct SplinterWrapper {
    pub splinter_type: SplinterType,
    pub splinter: Splinter,

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

    /// a higher-order wrapper for merging two splinter wrappers
    /// maybe we even want to provide support for merging larger numbers of splinters from an
    /// iterator: maybe this is a parallelism use case, group them up and do a few of the merges in
    /// parallel, then only optimize once at the very end
    pub fn merge(&mut self, _rhs: Self) {}

    pub fn merge_many(&mut self, _rhs: &[Self]) {}

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
