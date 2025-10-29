use pyo3::prelude::*;

pub(crate) fn spawn_thread_for_async<F, T>(
    asyncio_loop: &Bound<'_, PyAny>,
    f: F,
) -> PyResult<Py<PyAny>>
where
    F: FnOnce() -> PyResult<Py<T>>,
    F: Send + 'static,
{
    let fut = asyncio_loop.call_method0("create_future")?;

    // Extract the necessary methods and unbind them for later use in the thread
    let call_soon_threadsafe = asyncio_loop.getattr("call_soon_threadsafe")?.unbind();
    let set_result = fut.getattr("set_result")?.unbind();
    let set_exception = fut.getattr("set_exception")?.unbind();

    std::thread::spawn(move || {
        let result = f();
        Python::attach(|py| {
            let call_result = match result {
                Ok(res) => call_soon_threadsafe.call1(py, (set_result, res)),
                Err(err) => call_soon_threadsafe.call1(py, (set_exception, err)),
            };
            if let Err(e) = call_result {
                e.print(py);
            }
        });
    });

    return Ok(fut.into());
}
