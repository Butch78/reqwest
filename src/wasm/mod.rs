use std::convert::TryInto;
use std::time::Duration;

use js_sys::Function;
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{AbortController, AbortSignal};

mod body;
mod client;
/// TODO
#[cfg(feature = "multipart")]
pub mod multipart;
mod request;
mod response;

pub use self::body::Body;
pub use self::client::{Client, ClientBuilder};
pub use self::request::{Request, RequestBuilder};
pub use self::response::Response;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "setTimeout")]
    fn set_timeout(handler: &Function, timeout: i32) -> JsValue;

    #[wasm_bindgen(js_name = "clearTimeout")]
    fn clear_timeout(handle: JsValue) -> JsValue;
}

async fn promise<T>(promise: js_sys::Promise) -> Result<T, crate::error::BoxError>
where
    T: JsCast,
{
    use wasm_bindgen_futures::JsFuture;

    let js_val = JsFuture::from(promise).await.map_err(crate::error::wasm)?;

    js_val
        .dyn_into::<T>()
        .map_err(|_js_val| "promise resolved to unexpected type".into())
}

/// A guard that cancels a fetch request when dropped.
// The per-request timeout's `Closure<dyn FnMut()>` and another crate's timer closure
// share one wasm-bindgen export name on emscripten ("duplicate export name" at wasm-opt), so a
// timeout is not supported in this build; the abort controller alone remains.
struct AbortGuard {
    ctrl: AbortController,
}

impl AbortGuard {
    fn new() -> crate::Result<Self> {
        Ok(AbortGuard {
            ctrl: AbortController::new()
                .map_err(crate::error::wasm)
                .map_err(crate::error::builder)?,
        })
    }

    fn signal(&self) -> AbortSignal {
        self.ctrl.signal()
    }

    fn timeout(&mut self, _timeout: Duration) {
        // See the struct.
    }
}

impl Drop for AbortGuard {
    fn drop(&mut self) {
        self.ctrl.abort();
    }
}
