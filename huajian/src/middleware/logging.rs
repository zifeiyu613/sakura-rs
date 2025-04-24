use axum::extract::Request;
use axum::response::Response;
use time::Duration;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use rlog::span::Span;

// pub fn trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
//     TraceLayer::new_for_http()
//         .make_span_with(|request: &Request<_>| {
//             let path = request.uri().path().to_owned();
//             let method = request.method().clone();
//             rlog::info_span!("request", %path, %method)
//         })
//         .on_response(|response: &Response, latency: Duration, _span: &Span| {
//             let status = response.status().as_u16();
//             rlog::info!("response: status={} latency={}ms", status, latency.as_seconds_f32());
//         })
// }
