use std::collections::VecDeque;

use pyo3::{
    exceptions::{PyKeyError, PyTypeError, PyValueError},
    prelude::*,
    types::{PyDict, PyTuple, PyType},
};

struct Entry {
    key: Py<PyAny>,
    value: Py<PyAny>,
}

/// LRU(size, callback=None) -> new LRU dict that can store up to size elements
///
/// An LRU dict behaves like a standard dict, except that it stores only fixed
/// set of elements. Once the size overflows, it evicts least recently used
/// items.  If a callback is set it will call the callback with the evicted key
/// and item.
///
/// Eg:
/// >>> l = LRU(3)
/// >>> for i in range(5):
/// >>>   l[i] = str(i)
/// >>> l.keys()
/// [2,3,4]
///
/// Note: An LRU(n) can be thought of as a dict that will have the most
/// recently accessed n items.
#[pyclass(name = "LRU", module = "bot7685_ext")]
pub struct PyLRU {
    /// entries[0] is MRU, entries[last] is LRU
    entries: VecDeque<Entry>,
    size: usize,
    hits: usize,
    misses: usize,
    callback: Option<Py<PyAny>>,
}

impl PyLRU {
    /// Find the index of a key in the deque (O(n)).
    /// Returns None if not found.
    fn find_idx(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Option<usize>> {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.key.bind(py).eq(key)? {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    /// Move an existing entry at `idx` to the front (MRU).
    fn move_to_front(&mut self, idx: usize) {
        if idx != 0 {
            let entry = self.entries.remove(idx).unwrap();
            self.entries.push_front(entry);
        }
    }

    /// Evict the LRU item (back of deque), calling callback if set.
    fn evict_lru(&mut self, py: Python<'_>) -> PyResult<()> {
        if let Some(entry) = self.entries.pop_back() {
            if let Some(cb) = &self.callback {
                cb.bind(py).call1((entry.key, entry.value))?;
            }
        }
        Ok(())
    }

    /// Internal set: insert or update key/value, evict if over capacity.
    fn internal_set(
        &mut self,
        py: Python<'_>,
        key: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        if let Some(idx) = self.find_idx(py, key)? {
            // Update existing
            self.entries[idx].value = value.clone().unbind();
            self.move_to_front(idx);
        } else {
            // Insert new
            self.entries.push_front(Entry {
                key: key.clone().unbind(),
                value: value.clone().unbind(),
            });
            if self.entries.len() > self.size {
                self.evict_lru(py)?;
            }
        }
        Ok(())
    }
}

#[pymethods]
impl PyLRU {
    #[new]
    #[pyo3(signature = (size, callback=None))]
    fn new(py: Python<'_>, size: usize, callback: Option<Py<PyAny>>) -> PyResult<Self> {
        if size <= 0 {
            return Err(PyValueError::new_err("Size should be a positive number"));
        }
        if let Some(ref cb) = callback {
            if !cb.bind(py).is_callable() {
                return Err(PyTypeError::new_err("parameter must be callable"));
            }
        }
        Ok(Self {
            entries: VecDeque::new(),
            size,
            hits: 0,
            misses: 0,
            callback,
        })
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let dict = PyDict::new(py);
        for entry in &self.entries {
            dict.set_item(entry.key.bind(py), entry.value.bind(py))?;
        }
        Ok(format!("{}", dict.repr()?))
    }

    fn __len__(&self) -> usize {
        self.entries.len()
    }

    fn __contains__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<bool> {
        Ok(self.find_idx(py, key)?.is_some())
    }

    fn __getitem__(&mut self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        match self.find_idx(py, key)? {
            Some(idx) => {
                self.move_to_front(idx);
                self.hits += 1;
                Ok(self.entries[0].value.clone_ref(py))
            }
            None => {
                self.misses += 1;
                Err(PyKeyError::new_err(key.clone().unbind()))
            }
        }
    }

    fn __setitem__(
        &mut self,
        py: Python<'_>,
        key: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        self.internal_set(py, key, value)
    }

    fn __delitem__(&mut self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<()> {
        match self.find_idx(py, key)? {
            Some(idx) => {
                self.entries.remove(idx);
                Ok(())
            }
            None => Err(PyKeyError::new_err(key.clone().unbind())),
        }
    }

    /// L.get(key[, instead]) -> If L has key return its value, otherwise instead
    #[pyo3(signature = (key, instead=None))]
    fn get(
        &mut self,
        py: Python<'_>,
        key: &Bound<'_, PyAny>,
        instead: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        match self.find_idx(py, key)? {
            Some(idx) => {
                self.move_to_front(idx);
                self.hits += 1;
                Ok(self.entries[0].value.clone_ref(py))
            }
            None => {
                self.misses += 1;
                match instead {
                    Some(v) => Ok(v.clone().unbind()),
                    None => Ok(py.None()),
                }
            }
        }
    }

    /// L.setdefault(key, default=None) -> If L has key return its value,
    /// otherwise insert key with a value of default and return default
    #[pyo3(signature = (key, default=None))]
    fn setdefault(
        &mut self,
        py: Python<'_>,
        key: &Bound<'_, PyAny>,
        default: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        if let Some(idx) = self.find_idx(py, key)? {
            self.move_to_front(idx);
            self.hits += 1;
            return Ok(self.entries[0].value.clone_ref(py));
        }
        self.misses += 1;
        let val = default
            .map(|v| v.clone().unbind())
            .unwrap_or_else(|| py.None());
        self.internal_set(py, key, val.bind(py))?;
        Ok(val)
    }

    /// L.pop(key[, default]) -> If L has key return its value and remove it from L,
    /// otherwise return default. If default is not given and key is not in L, a KeyError is raised.
    #[pyo3(signature = (key, default=None))]
    fn pop(
        &mut self,
        py: Python<'_>,
        key: &Bound<'_, PyAny>,
        default: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        match self.find_idx(py, key)? {
            Some(idx) => {
                let entry = self.entries.remove(idx).unwrap();
                Ok(entry.value)
            }
            None => match default {
                Some(v) => Ok(v.clone().unbind()),
                None => Err(PyKeyError::new_err(key.clone().unbind())),
            },
        }
    }

    /// L.popitem([least_recent=True]) -> Returns and removes a (key, value) pair.
    /// The pair returned is the least-recently used if least_recent is true,
    /// or the most-recently used if false.
    #[pyo3(signature = (least_recent=true))]
    fn popitem(&mut self, py: Python<'_>, least_recent: bool) -> PyResult<Py<PyTuple>> {
        if self.entries.is_empty() {
            return Err(PyKeyError::new_err("popitem(): LRU dict is empty"));
        }
        let entry = if least_recent {
            self.entries.pop_back().unwrap()
        } else {
            self.entries.pop_front().unwrap()
        };
        Ok(PyTuple::new(py, [entry.key.bind(py), entry.value.bind(py)])?.unbind())
    }

    /// L.keys() -> list of L's keys in MRU order
    fn keys(&self, py: Python<'_>) -> PyResult<Vec<Py<PyAny>>> {
        Ok(self.entries.iter().map(|e| e.key.clone_ref(py)).collect())
    }

    /// L.values() -> list of L's values in MRU order
    fn values(&self, py: Python<'_>) -> PyResult<Vec<Py<PyAny>>> {
        Ok(self.entries.iter().map(|e| e.value.clone_ref(py)).collect())
    }

    /// L.items() -> list of L's items (key,value) in MRU order
    fn items(&self, py: Python<'_>) -> PyResult<Vec<Py<PyTuple>>> {
        self.entries
            .iter()
            .map(|e| Ok(PyTuple::new(py, [e.key.bind(py), e.value.bind(py)])?.unbind()))
            .collect()
    }

    /// L.has_key(key) -> Check if key is there in L
    fn has_key(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<bool> {
        Ok(self.find_idx(py, key)?.is_some())
    }

    /// L.set_size(size) -> set size of LRU
    fn set_size(&mut self, py: Python<'_>, size: usize) -> PyResult<()> {
        if size == 0 {
            return Err(PyValueError::new_err("Size should be a positive number"));
        }
        while self.entries.len() > size {
            self.evict_lru(py)?;
        }
        self.size = size;
        Ok(())
    }

    /// L.get_size() -> get size of LRU
    fn get_size(&self) -> usize {
        self.size
    }

    /// L.clear() -> clear LRU
    fn clear(&mut self) {
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// L.get_stats() -> returns a tuple with cache hits and misses
    fn get_stats(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.hits, self.misses])?.unbind())
    }

    /// L.peek_first_item() -> returns the MRU item (key,value) without changing key order
    fn peek_first_item(&self, py: Python<'_>) -> PyResult<Option<Py<PyTuple>>> {
        match self.entries.front() {
            Some(e) => Ok(Some(
                PyTuple::new(py, [e.key.bind(py), e.value.bind(py)])?.unbind(),
            )),
            None => Ok(None),
        }
    }

    /// L.peek_last_item() -> returns the LRU item (key,value) without changing key order
    fn peek_last_item(&self, py: Python<'_>) -> PyResult<Option<Py<PyTuple>>> {
        match self.entries.back() {
            Some(e) => Ok(Some(
                PyTuple::new(py, [e.key.bind(py), e.value.bind(py)])?.unbind(),
            )),
            None => Ok(None),
        }
    }

    /// L.update() -> update value for key in LRU
    #[pyo3(signature = (*args, **kwargs))]
    fn update(
        &mut self,
        py: Python<'_>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        if let Some(arg) = args.get_item(0).ok() {
            if let Ok(dict) = arg.cast::<PyDict>() {
                for (k, v) in dict.iter() {
                    self.internal_set(py, &k, &v)?;
                }
            }
        }
        if let Some(kw) = kwargs {
            for (k, v) in kw.iter() {
                self.internal_set(py, &k, &v)?;
            }
        }
        Ok(())
    }

    /// Support generic syntax: LRU[KT, VT]
    #[classmethod]
    fn __class_getitem__(cls: &Bound<'_, PyType>, item: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let py = item.py();
        let generic_alias = py.import("types")?.getattr("GenericAlias")?;
        Ok(generic_alias.call1((cls, item))?.unbind())
    }

    /// L.set_callback(callback) -> set a callback to call when an item is evicted.
    fn set_callback(&mut self, callback: &Bound<'_, PyAny>) -> PyResult<()> {
        if callback.is_none() {
            self.callback = None;
        } else if !callback.is_callable() {
            return Err(PyTypeError::new_err("parameter must be callable"));
        } else {
            self.callback = Some(callback.clone().unbind());
        }
        Ok(())
    }
}
