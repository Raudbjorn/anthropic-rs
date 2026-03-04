//! Platform abstractions for native vs WASM environments.
//!
//! This module provides unified APIs for operations that differ between
//! native (tokio) and WASM (browser/worker) runtimes: sleeping, spawning,
//! and stream type aliases.

use std::pin::Pin;

use bytes::Bytes;
use futures_core::Stream;

use crate::error::Result;

// ── Stream type aliases ──────────────────────────────────────────────

/// Byte stream from reqwest — `+ Send` on native, bare on WASM.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type BoxByteStream =
    Pin<Box<dyn Stream<Item = std::result::Result<Bytes, reqwest::Error>> + Send>>;

#[cfg(target_arch = "wasm32")]
pub(crate) type BoxByteStream =
    Pin<Box<dyn Stream<Item = std::result::Result<Bytes, reqwest::Error>>>>;

/// Typed result stream — `+ Send` on native, bare on WASM.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type BoxResultStream<T> = Pin<Box<dyn Stream<Item = Result<T>> + Send>>;

#[cfg(target_arch = "wasm32")]
pub(crate) type BoxResultStream<T> = Pin<Box<dyn Stream<Item = Result<T>>>>;

// ── Sleep ────────────────────────────────────────────────────────────

/// Async sleep — delegates to `tokio::time::sleep` on native,
/// `setTimeout` via JS interop on WASM.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn sleep(duration: std::time::Duration) {
    tokio::time::sleep(duration).await;
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn sleep(duration: std::time::Duration) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;

    let ms = duration.as_secs_f64() * 1000.0;
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        // Use globalThis.setTimeout — works in browsers, Node, Deno, CF Workers
        let global = js_sys::global();
        let Ok(set_timeout) =
            js_sys::Reflect::get(&global, &JsValue::from_str("setTimeout"))
        else {
            // No setTimeout available — resolve immediately
            let _ = resolve.call0(&JsValue::UNDEFINED);
            return;
        };
        let Ok(set_timeout_fn) = set_timeout.dyn_into::<js_sys::Function>() else {
            let _ = resolve.call0(&JsValue::UNDEFINED);
            return;
        };
        let args = js_sys::Array::of2(&resolve, &JsValue::from_f64(ms));
        let _ = set_timeout_fn.apply(&global, &args);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

// ── HTTP client builder ──────────────────────────────────────────────

/// Build a `reqwest::Client` with platform-appropriate timeout settings.
///
/// On native: applies `connect_timeout` and `request` timeout.
/// On WASM: skips timeouts (not supported by browser fetch API).
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn build_http_client(
    timeout: &crate::config::Timeout,
) -> std::result::Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .connect_timeout(timeout.connect)
        .timeout(timeout.request)
        .build()
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn build_http_client(
    _timeout: &crate::config::Timeout,
) -> std::result::Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder().build()
}

// ── Spawn + channel-based stream ─────────────────────────────────────

/// Spawn a background task and stream results through a channel.
///
/// On native: uses `tokio::spawn` + `futures_channel::mpsc`.
/// On WASM: uses `wasm_bindgen_futures::spawn_local` + `futures_channel::mpsc`.
///
/// The `producer` closure receives a `futures_channel::mpsc::Sender` and
/// should use `SinkExt::send()` to emit items.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn spawn_stream<T, F, Fut>(producer: F) -> BoxResultStream<T>
where
    T: Send + 'static,
    F: FnOnce(futures_channel::mpsc::Sender<Result<T>>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let (tx, rx) = futures_channel::mpsc::channel(32);
    tokio::spawn(async move {
        producer(tx).await;
    });
    Box::pin(rx)
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn spawn_stream<T, F, Fut>(producer: F) -> BoxResultStream<T>
where
    T: 'static,
    F: FnOnce(futures_channel::mpsc::Sender<Result<T>>) -> Fut + 'static,
    Fut: std::future::Future<Output = ()> + 'static,
{
    let (tx, rx) = futures_channel::mpsc::channel(32);
    wasm_bindgen_futures::spawn_local(async move {
        producer(tx).await;
    });
    Box::pin(rx)
}
