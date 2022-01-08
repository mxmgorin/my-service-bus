use std::sync::Arc;

use async_trait::async_trait;
use my_http_utils::{HttpContext, HttpFailResult, HttpServerMiddleware, MiddleWareResult};
use prometheus::{Encoder, TextEncoder};

use super::PrometheusDataSource;
pub struct PrometheusMiddleware<TPrometheusDataSource>
where
    TPrometheusDataSource: PrometheusDataSource + Send + Sync + 'static,
{
    data_source: Arc<TPrometheusDataSource>,
}

impl<TPrometheusDataSource> PrometheusMiddleware<TPrometheusDataSource>
where
    TPrometheusDataSource: PrometheusDataSource + Send + Sync + 'static,
{
    pub fn new(data_source: Arc<TPrometheusDataSource>) -> Self {
        Self { data_source }
    }
}

#[async_trait]
impl<TPrometheusDataSource> HttpServerMiddleware for PrometheusMiddleware<TPrometheusDataSource>
where
    TPrometheusDataSource: PrometheusDataSource + Send + Sync + 'static,
{
    async fn handle_request(&self, ctx: HttpContext) -> Result<MiddleWareResult, HttpFailResult> {
        if ctx.get_path_lower_case() != "/metrics" {
            return Ok(MiddleWareResult::Next(ctx));
        };

        let registry = self.data_source.get();

        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let data = String::from_utf8(buffer).unwrap();

        Ok(MiddleWareResult::Ok(data.into()))
    }
}
