use std::collections::HashMap;

use bit_tree;

use numpy::{IntoPyArray, PyArray1};
use pyo3::{prelude::*, types::PyString};

#[pyclass(frozen)]
struct Tokenizer {
    internal: bit_tree::Tokenizer,
}

#[pymethods]
impl Tokenizer {
    #[staticmethod]
    fn from_byte_dict(byte_token_dict: HashMap<Vec<u8>, u64>) -> Self {
        Self {
            internal: bit_tree::Tokenizer::from_byte_token_dict(
                byte_token_dict
                    .iter()
                    .map(|(k, &v)| (k.as_slice(), v))
                    .collect(),
            ),
        }
    }

    #[staticmethod]
    fn from_str_dict(token_dict: HashMap<String, u64>) -> Self {
        Self {
            internal: bit_tree::Tokenizer::from_token_dict(
                token_dict.iter().map(|(k, v)| (k.as_str(), *v)).collect(),
            ),
        }
    }

    fn tokenize<'py>(&self, py: Python<'py>, s: Bound<'py, PyString>) -> Bound<'py, PyArray1<u64>> {
        let raw_data = unsafe { s.data() }.expect("Data failure!");
        let mut bytes = raw_data.as_bytes().iter().map(|b| *b).peekable();
        let mut token_stream = vec![];
        loop {
            let next_tok = self.internal.nibble_token(&mut bytes);
            if next_tok == u64::MAX {
                break;
            }
            token_stream.push(next_tok);
        }
        token_stream.into_pyarray_bound(py)
    }
}

#[pymodule]
fn btoken(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Tokenizer>()?;
    Ok(())
}
