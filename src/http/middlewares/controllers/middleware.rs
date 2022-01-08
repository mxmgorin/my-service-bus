use async_trait::async_trait;
use std::sync::Arc;

use hyper::Method;
use my_http_utils::{HttpContext, HttpFailResult, HttpServerMiddleware, MiddleWareResult};

use super::{
    actions::{DeleteAction, GetAction, PostAction, PutAction},
    route::{DeleteRoute, GetRoute, PostRoute, PutRoute},
};

pub struct ControllersMiddleware {
    get: GetRoute,
    post: PostRoute,
    put: PutRoute,
    delete: DeleteRoute,
}

impl ControllersMiddleware {
    pub fn new() -> Self {
        Self {
            get: GetRoute::new(),
            post: PostRoute::new(),
            put: PutRoute::new(),
            delete: DeleteRoute::new(),
        }
    }

    pub fn register_get_action(
        &mut self,
        route: &str,
        action: Arc<dyn GetAction + Send + Sync + 'static>,
    ) {
        self.get.register(route, action);
    }

    pub fn register_post_action(
        &mut self,
        route: &str,
        action: Arc<dyn PostAction + Send + Sync + 'static>,
    ) {
        self.post.register(route, action);
    }

    pub fn register_put_action(
        &mut self,
        route: &str,
        action: Arc<dyn PutAction + Send + Sync + 'static>,
    ) {
        self.put.register(route, action);
    }

    pub fn register_delete_action(
        &mut self,
        route: &str,
        action: Arc<dyn DeleteAction + Send + Sync + 'static>,
    ) {
        self.delete.register(route, action);
    }
}

#[async_trait]
impl HttpServerMiddleware for ControllersMiddleware {
    async fn handle_request(&self, ctx: HttpContext) -> Result<MiddleWareResult, HttpFailResult> {
        let ref method = *ctx.get_method();
        match method {
            &Method::GET => {
                return self.get.handle_request(ctx).await;
            }
            &Method::POST => {
                return self.post.handle_request(ctx).await;
            }
            &Method::PUT => {
                return self.put.handle_request(ctx).await;
            }
            &Method::DELETE => {
                return self.delete.handle_request(ctx).await;
            }
            _ => {}
        }

        return Ok(MiddleWareResult::Next(ctx));
    }
}
