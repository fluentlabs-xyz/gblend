//! Per-RPC method-rewrite layer for Fluent's storage-truth endpoints.
//!
//! Fluent stores EVM contracts in an OwnableAccount wrapper; the standard `eth_getCode` and
//! `eth_getAccountInfo` return a normalized EVM-compatible view that strips that wrapper.
//! For local revm execution the compatibility view is *wrong* — the wrapper is needed so
//! the patched runtime can route calls to the correct delegated runtime. Fluent exposes raw
//! variants (`eth_getRawCode`, `eth_getRawAccountInfo`) that return the storage-truth bytes.
//!
//! This layer transparently rewrites the standard methods to the raw variants for any
//! provider built via [`super::ProviderBuilder`]. On non-Fluent endpoints the raw method
//! does not exist — we detect the JSON-RPC `-32601` (method not found) reply on the first
//! such call, remember the result, and use the original method on subsequent calls.
//!
//! Per-endpoint classification lives in an `Arc<AtomicU8>` shared between clones of the
//! service, so the probe runs at most once per provider instance.

use alloy_json_rpc::{Id, Request, RequestPacket, ResponsePacket, RpcError, SerializedRequest};
use alloy_transport::{TransportError, TransportFut};
use std::{
    borrow::Cow,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    task::{Context, Poll},
};
use tower::{Layer, Service};

/// Methods to rewrite. Each pair is `(standard, raw)`; the raw variant is tried first and
/// the standard is used as a fallback when the endpoint replies with `-32601`.
const REWRITES: &[(&str, &str)] = &[
    ("eth_getCode", "eth_getRawCode"),
    ("eth_getAccountInfo", "eth_getRawAccountInfo"),
];

/// JSON-RPC 2.0 "method not found" error code. Authoritative for distinguishing
/// "endpoint does not implement this RPC" from a transport / params / business error.
const RPC_METHOD_NOT_FOUND: i64 = -32601;

// Endpoint classification — populated lazily on the first rewritten call per service.
const UNKNOWN: u8 = 0;
const RAW_SUPPORTED: u8 = 1;
const RAW_UNSUPPORTED: u8 = 2;

/// Layer that rewrites `eth_getCode` / `eth_getAccountInfo` to their `Raw*` variants when
/// the endpoint supports them. See module docs for context.
#[derive(Clone, Debug, Default)]
pub struct FluentMethodRewriteLayer;

impl<S> Layer<S> for FluentMethodRewriteLayer {
    type Service = FluentMethodRewriteService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        FluentMethodRewriteService { inner, mode: Arc::new(AtomicU8::new(UNKNOWN)) }
    }
}

#[derive(Clone, Debug)]
pub struct FluentMethodRewriteService<S> {
    inner: S,
    /// Per-endpoint classification of raw-method support.
    mode: Arc<AtomicU8>,
}

impl<S> Service<RequestPacket> for FluentMethodRewriteService<S>
where
    S: Service<RequestPacket, Future = TransportFut<'static>, Error = TransportError>
        + Send
        + 'static
        + Clone,
{
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: RequestPacket) -> Self::Future {
        // Find a method rewrite + build the rewritten request. Anything that doesn't match
        // (batch packets, unrelated methods, params we can't re-serialize) passes through.
        let rewritten_opt = match &req {
            RequestPacket::Single(orig) => REWRITES
                .iter()
                .find_map(|(standard, raw)| (*standard == orig.method()).then_some(*raw))
                .and_then(|raw_method| rewrite_method(orig, raw_method)),
            RequestPacket::Batch(_) => None,
        };

        let Some(rewritten) = rewritten_opt else {
            return self.inner.call(req);
        };

        // Endpoint already classified as not supporting the raw variant — skip the probe.
        if self.mode.load(Ordering::Relaxed) == RAW_UNSUPPORTED {
            return self.inner.call(req);
        }

        let mut inner = self.inner.clone();
        let mode = self.mode.clone();

        Box::pin(async move {
            let raw_resp = inner.call(RequestPacket::Single(rewritten)).await;

            if is_method_not_found(&raw_resp) {
                // Endpoint is not Fluent (or doesn't expose raw methods). Remember and
                // forward the original request so subsequent calls skip the probe entirely.
                mode.store(RAW_UNSUPPORTED, Ordering::Relaxed);
                return inner.call(req).await;
            }

            // Successful raw response → mark the endpoint as supporting raw methods.
            // Transport errors leave the classification at Unknown so the next call retries
            // the probe rather than locking in a misclassification from a transient blip.
            if raw_resp.is_ok() {
                mode.compare_exchange(
                    UNKNOWN,
                    RAW_SUPPORTED,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .ok();
            }
            raw_resp
        })
    }
}

/// Builds a fresh [`SerializedRequest`] with the same id and params as `orig` but a
/// different method name. Returns `None` if the params couldn't be re-serialized; the
/// caller should then forward the original request unchanged.
fn rewrite_method(orig: &SerializedRequest, new_method: &'static str) -> Option<SerializedRequest> {
    let params: serde_json::Value = orig
        .params()
        .and_then(|raw| serde_json::from_str(raw.get()).ok())
        .unwrap_or_else(|| serde_json::Value::Array(Vec::new()));
    Request::new(Cow::Borrowed(new_method), <Id as Clone>::clone(orig.id()), params)
        .serialize()
        .ok()
}

fn is_method_not_found(r: &Result<ResponsePacket, TransportError>) -> bool {
    match r {
        Ok(packet) => packet.as_error().is_some_and(|e| e.code == RPC_METHOD_NOT_FOUND),
        Err(RpcError::ErrorResp(e)) => e.code == RPC_METHOD_NOT_FOUND,
        Err(_) => false,
    }
}
