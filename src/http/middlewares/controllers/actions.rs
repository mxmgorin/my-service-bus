use async_trait::async_trait;
use my_http_utils::{HttpContext, HttpFailResult, HttpOkResult};

#[async_trait]
pub trait GetAction {
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult>;
}

#[async_trait]
pub trait PostAction {
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult>;
}

#[async_trait]
pub trait PutAction {
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult>;
}

#[async_trait]
pub trait DeleteAction {
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult>;
}
