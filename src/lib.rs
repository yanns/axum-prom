use axum::extract::MatchedPath;
use http::{Method, Request, Response, StatusCode};
use pin_project::pin_project;
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder,
};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};

pub const DEFAULT_ENDPOINT: &str = "/metrics";

pub struct PrometheusMetricsBuilder {
    namespace: String,
    endpoint: Option<String>,
    const_labels: HashMap<String, String>,
    registry: Registry,
    buckets: Vec<f64>,
}

impl PrometheusMetricsBuilder {
    /// Create new `PrometheusMetricsBuilder`
    ///
    /// namespace example: "shadow"
    #[must_use]
    pub fn new(namespace: &str) -> Self {
        Self {
            namespace: namespace.into(),
            endpoint: Some(DEFAULT_ENDPOINT.into()),
            const_labels: HashMap::new(),
            registry: Registry::new(),
            buckets: prometheus::DEFAULT_BUCKETS.to_vec(),
        }
    }

    /// Set axum web endpoint
    ///
    /// Example: "/metrics"
    #[must_use]
    pub fn endpoint(mut self, value: Option<&str>) -> Self {
        self.endpoint = value.map(|s| s.into());
        self
    }

    /// Set histogram buckets
    #[must_use]
    pub fn buckets(mut self, value: &[f64]) -> Self {
        self.buckets = value.to_vec();
        self
    }

    /// Set labels to add on every metrics
    #[must_use]
    pub fn const_labels(mut self, value: HashMap<String, String>) -> Self {
        self.const_labels = value;
        self
    }

    /// Set registry
    ///
    /// By default one is set and is internal to `PrometheusMetrics`
    #[must_use]
    pub fn registry(mut self, value: Registry) -> Self {
        self.registry = value;
        self
    }

    /// Instantiate `PrometheusMetrics` struct
    pub fn pair(
        self,
    ) -> Result<(PrometheusMetrics, PrometheusMetricsRegistry), Box<dyn std::error::Error>> {
        let http_requests_total_opts =
            Opts::new("http_requests_total", "Total number of HTTP requests")
                .namespace(&self.namespace)
                .const_labels(self.const_labels.clone());

        let http_requests_total =
            IntCounterVec::new(http_requests_total_opts, &["endpoint", "method", "status"])?;

        let http_requests_duration_seconds_opts = HistogramOpts::new(
            "http_requests_duration_seconds",
            "HTTP request duration in seconds for all requests",
        )
            .namespace(&self.namespace)
            .buckets(self.buckets.clone())
            .const_labels(self.const_labels.clone());

        let http_requests_duration_seconds = HistogramVec::new(
            http_requests_duration_seconds_opts,
            &["endpoint", "method", "status"],
        )?;

        self.registry
            .register(Box::new(http_requests_total.clone()))?;
        self.registry
            .register(Box::new(http_requests_duration_seconds.clone()))?;

        let prometheus_metrics = PrometheusMetrics {
            http_requests_total,
            http_requests_duration_seconds,
            namespace: self.namespace,
            endpoint: self.endpoint,
            const_labels: self.const_labels,
        };
        let prometheus_metrics_registry = PrometheusMetricsRegistry {
            registry: self.registry,
        };
        Ok((prometheus_metrics, prometheus_metrics_registry))
    }
}

#[derive(Debug, Clone)]
pub struct PrometheusMetrics {
    pub http_requests_total: IntCounterVec,
    pub http_requests_duration_seconds: HistogramVec,

    pub namespace: String,
    pub endpoint: Option<String>,
    pub const_labels: HashMap<String, String>,
}

impl PrometheusMetrics {
    fn matches(&self, path: &str, method: &Method) -> bool {
        match &self.endpoint {
            Some(endpoint) => endpoint == path && method == Method::GET,
            None => false,
        }
    }

    fn update_metrics(&self, path: &str, method: &Method, status: StatusCode, clock: Instant) {
        let method = method.to_string();
        let status = status.as_u16().to_string();

        let elapsed = clock.elapsed();
        let duration = elapsed.as_secs_f64();
        self.http_requests_duration_seconds
            .with_label_values(&[path, &method, &status])
            .observe(duration);

        self.http_requests_total
            .with_label_values(&[path, &method, &status])
            .inc();
    }
}

#[derive(Debug, Clone)]
pub struct PrometheusMetricsRegistry {
    /// exposed registry for custom prometheus metrics
    pub registry: Registry,
}

impl PrometheusMetricsRegistry {
    #[must_use]
    pub fn metrics(&self) -> String {
        let mut buffer = vec![];
        TextEncoder::new()
            .encode(&self.registry.gather(), &mut buffer)
            .unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

impl<S> Layer<S> for PrometheusMetrics {
    type Service = AxumMetrics<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AxumMetrics {
            inner,
            prometheus_metrics: self.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AxumMetrics<S> {
    inner: S,
    prometheus_metrics: PrometheusMetrics,
}

impl<S, R, ResBody> Service<Request<R>> for AxumMetrics<S>
    where
        S: Service<Request<R>, Response = Response<ResBody>>,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = ObservedResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<R>) -> Self::Future {
        let method = req.method().clone();
        let path = req
            .extensions()
            .get::<MatchedPath>() // the matched path is the route with placeholders, like "/:project_key/graphql"
            .map_or_else(|| req.uri().path().to_string(), |p| p.as_str().to_string());
        ObservedResponseFuture {
            inner: self.inner.call(req),
            time: Instant::now(),
            method,
            path,
            prometheus_metrics: Arc::new(self.prometheus_metrics.clone()),
        }
    }
}

#[pin_project]
pub struct ObservedResponseFuture<F> {
    #[pin]
    inner: F,
    time: Instant,
    method: Method,
    path: String,
    prometheus_metrics: Arc<PrometheusMetrics>,
}

impl<F, B, E> Future for ObservedResponseFuture<F>
    where
        F: Future<Output = Result<Response<B>, E>>,
{
    type Output = Result<Response<B>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let response: Response<B> = futures_core::ready!(this.inner.poll(cx))?;

        let prometheus_metrics = this.prometheus_metrics;
        let path = &this.path;
        let method = &this.method;

        if !prometheus_metrics.matches(path, method) {
            prometheus_metrics.update_metrics(path, method, response.status(), *this.time);
        }

        Poll::Ready(Ok(response))
    }
}
