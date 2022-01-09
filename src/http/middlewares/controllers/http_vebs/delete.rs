use std::{collections::HashMap, sync::Arc};

use my_http_utils::{http_path::PathSegments, HttpContext, HttpFailResult, MiddleWareResult};

use crate::http::middlewares::controllers::actions::DeleteAction;

pub struct DeleteRouteAction {
    route: PathSegments,
    action: Arc<dyn DeleteAction + Send + Sync + 'static>,
}

pub struct DeleteRoute {
    no_keys: HashMap<String, DeleteRouteAction>,
    with_keys: Vec<DeleteRouteAction>,
}

impl DeleteRoute {
    pub fn new() -> Self {
        Self {
            no_keys: HashMap::new(),
            with_keys: Vec::new(),
        }
    }

    pub fn register(&mut self, route: &str, action: Arc<dyn DeleteAction + Send + Sync + 'static>) {
        let route = PathSegments::new(route);

        let action = DeleteRouteAction { route, action };

        if action.route.keys_amount == 0 {
            self.no_keys
                .insert(action.route.path.to_lowercase(), action);
        } else {
            self.with_keys.push(action);
        }
    }

    pub async fn handle_request(
        &self,
        mut ctx: HttpContext,
    ) -> Result<MiddleWareResult, HttpFailResult> {
        if let Some(route_action) = self.no_keys.get(ctx.get_path_lower_case()) {
            let result = route_action.action.handle_request(ctx).await?;
            return Ok(MiddleWareResult::Ok(result));
        }

        for route_action in &self.with_keys {
            if route_action.route.is_my_path(ctx.get_path_lower_case()) {
                ctx.route = Some(route_action.route.clone());
                let result = route_action.action.handle_request(ctx).await?;
                return Ok(MiddleWareResult::Ok(result));
            }
        }

        Ok(MiddleWareResult::Next(ctx))
    }
}
