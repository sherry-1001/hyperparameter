use std::cell::RefCell;
use std::ffi::c_void;

use pyo3::exceptions::PyValueError;
use pyo3::ffi::Py_DecRef;
use pyo3::ffi::Py_IncRef;
use pyo3::prelude::*;
use pyo3::types::PyBool;
use pyo3::types::PyDict;
use pyo3::types::PyFloat;
use pyo3::types::PyInt;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::FromPyPointer;

use crate::entry::Value;
use crate::storage::frozen_as_global_storage;
use crate::storage::Storage;
use crate::storage::StorageManager;
use crate::storage::MGR;

#[pyclass]
pub struct KVStorage {
    storage: Storage,
    isview: bool,
}

#[pymethods]
impl KVStorage {
    #[new]
    pub fn new() -> KVStorage {
        KVStorage {
            storage: Storage::new(),
            isview: false,
        }
    }

    pub unsafe fn storage(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        let res = PyDict::new(py);
        for k in self.storage.keys().iter() {
            match self.storage.get(k).unwrap() {
                Value::Empty => Ok(()),
                Value::Int(v) => res.set_item(k, v),
                Value::Float(v) => res.set_item(k, v),
                Value::Text(v) => res.set_item(k, v.as_str()),
                Value::Boolen(v) => res.set_item(k, v),
                Value::UserDefined(v, _) => res.set_item(k, v as u64),
                Value::PyObject(v, _) => {
                    res.set_item(k, PyAny::from_owned_ptr(py, v as *mut pyo3::ffi::PyObject))
                }
            }
            .unwrap();
        }
        Ok(res.into())
    }

    pub unsafe fn keys(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        let res = PyList::new(py, self.storage.keys());
        Ok(res.into())
    }

    pub unsafe fn _update(&mut self, kws: &PyDict, prefix: Option<String>) {
        kws.iter()
            .map(|(k, v)| {
                let key = match &prefix {
                    Some(p) => format!("{}.{}", p, k.extract::<String>().unwrap()),
                    None => k.extract::<String>().unwrap(),
                };
                if v.is_instance_of::<PyDict>().unwrap() {
                    self._update(&v.downcast::<PyDict>().unwrap(), Some(key));
                } else {
                    self.put(key, v).unwrap();
                }
            })
            .count();
    }

    pub unsafe fn update(&mut self, kws: &PyDict) {
        self._update(kws, None);
    }

    pub unsafe fn clear(&mut self) {
        for k in self.storage.keys().iter() {
            self.storage.put(k, Value::Empty);
        }
    }

    pub unsafe fn get(&mut self, py: Python<'_>, key: String) -> PyResult<Option<PyObject>> {
        match self.storage.get(key) {
            Some(val) => match val {
                Value::Empty => Err(PyValueError::new_err("not found")),
                Value::Int(v) => Ok(Some(v.into_py(py))),
                Value::Float(v) => Ok(Some(v.into_py(py))),
                Value::Text(v) => Ok(Some(v.into_py(py))),
                Value::Boolen(v) => Ok(Some(v.into_py(py))),
                Value::UserDefined(v, _) => Ok(Some((v as u64).into_py(py))),
                Value::PyObject(v, _) => Ok(Some(
                    PyAny::from_borrowed_ptr(py, v as *mut pyo3::ffi::PyObject).into(),
                )),
            },
            None => Err(PyValueError::new_err("not found")),
        }
    }

    pub unsafe fn put(&mut self, key: String, val: &PyAny) -> PyResult<()> {
        if self.isview {
            MGR.with(|mgr: &RefCell<StorageManager>| mgr.borrow_mut().put_key(key.clone()));
            // MGR.with_borrow_mut(|mgr| mgr.put_key(key.clone()));
        }
        if val.is_none() {
            self.storage.put(key, Value::Empty);
            return Ok(());
        }
        if val.is_instance_of::<PyBool>().unwrap() {
            self.storage.put(key, val.extract::<bool>().unwrap());
        } else if val.is_instance_of::<PyFloat>().unwrap() {
            self.storage.put(key, val.extract::<f64>().unwrap());
        } else if val.is_instance_of::<PyString>().unwrap() {
            self.storage.put(key, val.extract::<&str>().unwrap());
        } else if val.is_instance_of::<PyInt>().unwrap() {
            self.storage.put(key, val.extract::<i64>().unwrap());
        } else {
            // TODO support release pyobj
            Py_IncRef(val.into_ptr());
            self.storage.put(
                key,
                Value::pyobj(val.into_ptr() as *mut c_void, |obj: *mut c_void| {
                    Py_DecRef(obj as *mut pyo3::ffi::PyObject);
                }),
            );
        }
        Ok(())
    }

    pub fn enter(&mut self) {
        self.storage.enter();
    }

    pub fn exit(&mut self) {
        self.storage.exit();
    }

    #[staticmethod]
    pub fn current() -> KVStorage {
        let mut kv = KVStorage {
            storage: Storage::new(),
            isview: true,
        };
        kv.storage.isview = 1;
        kv
    }

    #[staticmethod]
    pub fn frozen() {
        frozen_as_global_storage();
    }
}

#[pymodule]
fn rbackend(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<KVStorage>()?;
    Ok(())
}
