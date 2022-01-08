use std::{collections::HashMap, sync::Arc};

use my_http_utils::{http_path::PathSegments, HttpContext, HttpFailResult, MiddleWareResult};

use super::actions::{DeleteAction, GetAction, PostAction, PutAction};

pub struct GetRouteAction {
    route: PathSegments,
    action: Arc<dyn GetAction + Send + Sync + 'static>,
}

pub struct GetRoute {
    no_keys: HashMap<String, GetRouteAction>,
    with_keys: Vec<GetRouteAction>,
}

impl GetRoute {
    pub fn new() -> Self {
        Self {
            no_keys: HashMap::new(),
            with_keys: Vec::new(),
        }
    }

    pub fn register(&mut self, route: &str, action: Arc<dyn GetAction + Send + Sync + 'static>) {
        let route = PathSegments::new(route);

        let action = GetRouteAction { route, action };

        if action.route.keys_amount == 0 {
            self.no_keys
                .insert(action.route.path.to_lowercase(), action);
        } else {
            self.with_keys.push(action);
        }
    }

    pub async fn handle_request(
        &self,
        ctx: HttpContext,
    ) -> Result<MiddleWareResult, HttpFailResult> {
        let path = ctx.get_path_lower_case();
        if let Some(route_action) = self.no_keys.get(path) {
            let result = route_action.action.handle_request(ctx).await?;
            return Ok(MiddleWareResult::Ok(result));
        }

        for route_action in &self.with_keys {
            if route_action.route.is_my_path(ctx.get_path_lower_case()) {
                let result = route_action.action.handle_request(ctx).await?;
                return Ok(MiddleWareResult::Ok(result));
            }
        }

        Ok(MiddleWareResult::Next(ctx))
    }
}

pub struct PostRouteAction {
    route: PathSegments,
    action: Arc<dyn PostAction + Send + Sync + 'static>,
}

pub struct PostRoute {
    no_keys: HashMap<String, PostRouteAction>,
    with_keys: Vec<PostRouteAction>,
}

impl PostRoute {
    pub fn new() -> Self {
        Self {
            no_keys: HashMap::new(),
            with_keys: Vec::new(),
        }
    }

    pub fn register(&mut self, route: &str, action: Arc<dyn PostAction + Send + Sync + 'static>) {
        let route = PathSegments::new(route);

        let action = PostRouteAction { route, action };

        if action.route.keys_amount == 0 {
            self.no_keys
                .insert(action.route.path.to_lowercase(), action);
        } else {
            self.with_keys.push(action);
        }
    }

    pub async fn handle_request(
        &self,
        ctx: HttpContext,
    ) -> Result<MiddleWareResult, HttpFailResult> {
        if let Some(route_action) = self.no_keys.get(ctx.get_path_lower_case()) {
            let result = route_action.action.handle_request(ctx).await?;
            return Ok(MiddleWareResult::Ok(result));
        }

        for route_action in &self.with_keys {
            if route_action.route.is_my_path(ctx.get_path_lower_case()) {
                let result = route_action.action.handle_request(ctx).await?;
                return Ok(MiddleWareResult::Ok(result));
            }
        }

        Ok(MiddleWareResult::Next(ctx))
    }
}

pub struct PutRouteAction {
    route: PathSegments,
    action: Arc<dyn PutAction + Send + Sync + 'static>,
}

pub struct PutRoute {
    no_keys: HashMap<String, PutRouteAction>,
    with_keys: Vec<PutRouteAction>,
}

impl PutRoute {
    pub fn new() -> Self {
        Self {
            no_keys: HashMap::new(),
            with_keys: Vec::new(),
        }
    }

    pub fn register(&mut self, route: &str, action: Arc<dyn PutAction + Send + Sync + 'static>) {
        let route = PathSegments::new(route);

        let action = PutRouteAction { route, action };

        if action.route.keys_amount == 0 {
            self.no_keys
                .insert(action.route.path.to_lowercase(), action);
        } else {
            self.with_keys.push(action);
        }
    }

    pub async fn handle_request(
        &self,
        ctx: HttpContext,
    ) -> Result<MiddleWareResult, HttpFailResult> {
        if let Some(route_action) = self.no_keys.get(ctx.get_path_lower_case()) {
            let result = route_action.action.handle_request(ctx).await?;
            return Ok(MiddleWareResult::Ok(result));
        }

        for route_action in &self.with_keys {
            if route_action.route.is_my_path(ctx.get_path_lower_case()) {
                let result = route_action.action.handle_request(ctx).await?;
                return Ok(MiddleWareResult::Ok(result));
            }
        }

        Ok(MiddleWareResult::Next(ctx))
    }
}

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
        ctx: HttpContext,
    ) -> Result<MiddleWareResult, HttpFailResult> {
        if let Some(route_action) = self.no_keys.get(ctx.get_path_lower_case()) {
            let result = route_action.action.handle_request(ctx).await?;
            return Ok(MiddleWareResult::Ok(result));
        }

        for route_action in &self.with_keys {
            if route_action.route.is_my_path(ctx.get_path_lower_case()) {
                let result = route_action.action.handle_request(ctx).await?;
                return Ok(MiddleWareResult::Ok(result));
            }
        }

        Ok(MiddleWareResult::Next(ctx))
    }
}
