use pyo3::{IntoPyObjectExt, prelude::*};

pub(crate) fn spawn_thread_for_async<F, T>(
    asyncio_loop: &Bound<'_, PyAny>,
    f: F,
) -> PyResult<Py<PyAny>>
where
    F: FnOnce() -> PyResult<Py<T>>,
    F: Send + 'static,
{
    let fut = asyncio_loop.call_method0("create_future")?.unbind();
    let fut_ref = fut.clone_ref(asyncio_loop.py());
    let asyncio_loop = asyncio_loop.clone().unbind();

    std::thread::spawn(move || {
        let result = f();
        Python::attach(|py| -> PyResult<()> {
            let fut = fut_ref.into_bound(py);
            let call_args = match result {
                Ok(res) => (fut.getattr("set_result")?, res.into_py_any(py)?),
                Err(err) => (fut.getattr("set_exception")?, err.into_py_any(py)?),
            };
            let call_result = asyncio_loop
                .into_bound(py)
                .call_method1("call_soon_threadsafe", call_args);
            if let Err(e) = call_result {
                e.print(py);
            }
            Ok(())
        })
    });

    return Ok(fut.into());
}
