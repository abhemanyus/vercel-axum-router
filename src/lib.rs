use axum::body::HttpBody;
use axum::response::IntoResponse;
use std::{future::Future, pin::Pin};
use tower::Layer;
use tower_service::Service;
use vercel_runtime::{Body, Error, Request, Response};

#[derive(Default, Clone, Copy)]
pub struct LambdaLayer {
    trim_stage: bool,
}

impl LambdaLayer {
    pub fn trim_stage(mut self) -> Self {
        self.trim_stage = true;
        self
    }
}

impl<S> Layer<S> for LambdaLayer {
    type Service = LambdaService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LambdaService {
            inner,
            layer: *self,
        }
    }
}

pub struct LambdaService<S> {
    inner: S,
    layer: LambdaLayer,
}

impl<S> Service<Request> for LambdaService<S>
where
    S: Service<axum::http::Request<axum::body::Body>>,
    S::Response: axum::response::IntoResponse + Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let uri = req.uri().clone();
        let rawpath = uri.path().to_owned();
        let (mut parts, body) = req.into_parts();
        let body = match body {
            Body::Empty => axum::body::Body::default(),
            Body::Text(t) => t.into(),
            Body::Binary(v) => v.into(),
        };

        if self.layer.trim_stage {
            let mut url = match uri.host() {
                None => rawpath,
                Some(host) => format!(
                    "{}://{}{}",
                    uri.scheme_str().unwrap_or("https"),
                    host,
                    rawpath
                ),
            };

            if let Some(query) = uri.query() {
                url.push('?');
                url.push_str(query);
            }
            parts.uri = url.parse::<axum::http::Uri>().unwrap();
        }

        let request = axum::http::Request::from_parts(parts, body);

        let fut = self.inner.call(request);
        let fut = async move {
            let resp = fut.await?;
            let (parts, mut body) = resp.into_response().into_parts();
            let bytes = body.data().await.unwrap()?;
            let bytes: &[u8] = &bytes;
            let resp: axum::response::Response<Body> = match std::str::from_utf8(bytes) {
                Ok(s) => axum::response::Response::from_parts(parts, s.into()),
                Err(_) => axum::response::Response::from_parts(parts, bytes.into()),
            };
            Ok(resp)
        };

        Box::pin(fut)
    }
}
