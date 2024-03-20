// This file is dual licensed under the terms of the Apache License, Version
// 2.0, and the BSD License. See the LICENSE file in the root of this repository
// for complete details.

use crate::types;
use pyo3::types::IntoPyDict;
use std::slice;

pub(crate) struct CffiBuf<'p> {
    _pyobj: &'p pyo3::PyAny,
    _bufobj: &'p pyo3::PyAny,
    buf: &'p [u8],
}

fn _extract_buffer_length(
    pyobj: &pyo3::PyAny,
    mutable: bool,
) -> pyo3::PyResult<(&pyo3::PyAny, usize)> {
    let py = pyobj.py();
    let bufobj = if mutable {
        let kwargs = [(pyo3::intern!(py, "require_writable"), true)].into_py_dict(py);
        types::FFI_FROM_BUFFER
            .get(py)?
            .call((pyobj,), Some(kwargs))?
    } else {
        types::FFI_FROM_BUFFER.get(py)?.call1((pyobj,))?
    };
    let ptrval = types::FFI_CAST
        .get(py)?
        .call1((pyo3::intern!(py, "uintptr_t"), bufobj))?
        .call_method0(pyo3::intern!(py, "__int__"))?
        .extract::<usize>()?;
    Ok((bufobj, ptrval))
}

impl CffiBuf<'_> {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.buf
    }
}

impl<'a> pyo3::conversion::FromPyObject<'a> for CffiBuf<'a> {
    fn extract(pyobj: &'a pyo3::PyAny) -> pyo3::PyResult<Self> {
        let (bufobj, ptrval) = _extract_buffer_length(pyobj, false)?;
        let len = bufobj.len()?;
        let buf = if len == 0 {
            &[]
        } else {
            // SAFETY: _extract_buffer_length ensures that we have a valid ptr
            // and length (and we ensure we meet slice's requirements for
            // 0-length slices above), we're keeping pyobj alive which ensures
            // the buffer is valid. But! There is no actually guarantee
            // against concurrent mutation. See
            // https://alexgaynor.net/2022/oct/23/buffers-on-the-edge/
            // for details. This is the same as our cffi status quo ante, so
            // we're doing an unsound thing and living with it.
            unsafe { slice::from_raw_parts(ptrval as *const u8, len) }
        };

        Ok(CffiBuf {
            _pyobj: pyobj,
            _bufobj: bufobj,
            buf,
        })
    }
}

pub(crate) struct CffiMutBuf<'p> {
    _pyobj: &'p pyo3::PyAny,
    _bufobj: &'p pyo3::PyAny,
    buf: &'p mut [u8],
}

impl CffiMutBuf<'_> {
    pub(crate) fn as_mut_bytes(&mut self) -> &mut [u8] {
        self.buf
    }
}

impl<'a> pyo3::conversion::FromPyObject<'a> for CffiMutBuf<'a> {
    fn extract(pyobj: &'a pyo3::PyAny) -> pyo3::PyResult<Self> {
        let (bufobj, ptrval) = _extract_buffer_length(pyobj, true)?;

        let len = bufobj.len()?;
        let buf = if len == 0 {
            &mut []
        } else {
            // SAFETY: _extract_buffer_length ensures that we have a valid ptr
            // and length (and we ensure we meet slice's requirements for
            // 0-length slices above), we're keeping pyobj alive which ensures
            // the buffer is valid. But! There is no actually guarantee
            // against concurrent mutation. See
            // https://alexgaynor.net/2022/oct/23/buffers-on-the-edge/
            // for details. This is the same as our cffi status quo ante, so
            // we're doing an unsound thing and living with it.
            unsafe { slice::from_raw_parts_mut(ptrval as *mut u8, len) }
        };

        Ok(CffiMutBuf {
            _pyobj: pyobj,
            _bufobj: bufobj,
            buf,
        })
    }
}
