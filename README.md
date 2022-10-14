# Prometheus metrics for axum

[Prometheus](https://prometheus.io) metrics for [axum](https://docs.rs/axum/latest/axum/) web applications.

This crate is similar to:
- [`actix-web-prom`](https://github.com/nlopes/actix-web-prom)
- [`rocket_prometheus`](https://github.com/sd2k/rocket_prometheus)
- [`axum-prometheus`](https://github.com/Ptrskay3/axum-prometheus)


## features:
- The metrics are exposed given a certain namespace that *you* define
- The requests to the metrics endpoint are **not** taken into account in the exposed metrics.
  (You can change this behavior by setting the `endpoint` to `None`)
- By default, two metrics are recorded:
  - the number of requests (as [counter](https://prometheus.io/docs/concepts/metric_types/#counter)) 
  - the requests` durations (as [histogram](https://prometheus.io/docs/concepts/metric_types/#histogram))

# Example

```
$ cargo run --example simple

# send a request to /
$ curl localhost:3000
Hello, World!

# send 2 requests to "/hello/:name"
$ curl localhost:3000/hello/you
Hello you!

$ curl localhost:3000/hello/universe
Hello universe!

# The metrics contains data for 2 endpoints: "/" and "/hello/:name"
$ curl localhost:3000/metrics
# HELP myapp_http_requests_duration_seconds HTTP request duration in seconds for all requests
# TYPE myapp_http_requests_duration_seconds histogram
myapp_http_requests_duration_seconds_bucket{endpoint="/",method="GET",status="200",le="0.005"} 1
myapp_http_requests_duration_seconds_bucket{endpoint="/",method="GET",status="200",le="0.01"} 1
myapp_http_requests_duration_seconds_bucket{endpoint="/",method="GET",status="200",le="0.025"} 1
myapp_http_requests_duration_seconds_bucket{endpoint="/",method="GET",status="200",le="0.05"} 1
myapp_http_requests_duration_seconds_bucket{endpoint="/",method="GET",status="200",le="0.1"} 1
myapp_http_requests_duration_seconds_bucket{endpoint="/",method="GET",status="200",le="0.25"} 1
myapp_http_requests_duration_seconds_bucket{endpoint="/",method="GET",status="200",le="0.5"} 1
myapp_http_requests_duration_seconds_bucket{endpoint="/",method="GET",status="200",le="1"} 1
myapp_http_requests_duration_seconds_bucket{endpoint="/",method="GET",status="200",le="2.5"} 1
myapp_http_requests_duration_seconds_bucket{endpoint="/",method="GET",status="200",le="5"} 1
myapp_http_requests_duration_seconds_bucket{endpoint="/",method="GET",status="200",le="10"} 1
myapp_http_requests_duration_seconds_bucket{endpoint="/",method="GET",status="200",le="+Inf"} 1
myapp_http_requests_duration_seconds_sum{endpoint="/",method="GET",status="200"} 0.000081435
myapp_http_requests_duration_seconds_count{endpoint="/",method="GET",status="200"} 1
myapp_http_requests_duration_seconds_bucket{endpoint="/hello/:name",method="GET",status="200",le="0.005"} 2
myapp_http_requests_duration_seconds_bucket{endpoint="/hello/:name",method="GET",status="200",le="0.01"} 2
myapp_http_requests_duration_seconds_bucket{endpoint="/hello/:name",method="GET",status="200",le="0.025"} 2
myapp_http_requests_duration_seconds_bucket{endpoint="/hello/:name",method="GET",status="200",le="0.05"} 2
myapp_http_requests_duration_seconds_bucket{endpoint="/hello/:name",method="GET",status="200",le="0.1"} 2
myapp_http_requests_duration_seconds_bucket{endpoint="/hello/:name",method="GET",status="200",le="0.25"} 2
myapp_http_requests_duration_seconds_bucket{endpoint="/hello/:name",method="GET",status="200",le="0.5"} 2
myapp_http_requests_duration_seconds_bucket{endpoint="/hello/:name",method="GET",status="200",le="1"} 2
myapp_http_requests_duration_seconds_bucket{endpoint="/hello/:name",method="GET",status="200",le="2.5"} 2
myapp_http_requests_duration_seconds_bucket{endpoint="/hello/:name",method="GET",status="200",le="5"} 2
myapp_http_requests_duration_seconds_bucket{endpoint="/hello/:name",method="GET",status="200",le="10"} 2
myapp_http_requests_duration_seconds_bucket{endpoint="/hello/:name",method="GET",status="200",le="+Inf"} 2
myapp_http_requests_duration_seconds_sum{endpoint="/hello/:name",method="GET",status="200"} 0.000136117
myapp_http_requests_duration_seconds_count{endpoint="/hello/:name",method="GET",status="200"} 2
# HELP myapp_http_requests_total Total number of HTTP requests
# TYPE myapp_http_requests_total counter
myapp_http_requests_total{endpoint="/",method="GET",status="200"} 1
myapp_http_requests_total{endpoint="/hello/:name",method="GET",status="200"} 2
```