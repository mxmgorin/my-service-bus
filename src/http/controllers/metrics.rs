use crate::{
    app::AppContext,
    http::{HttpFailResult, HttpOkResult},
};

pub async fn get(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let data = app.prometheus.build_prometheus_content();
    Ok(data.into())
}
