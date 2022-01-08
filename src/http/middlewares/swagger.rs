use std::collections::HashMap;

use async_trait::async_trait;
use my_http_utils::{
    HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware, MiddleWareResult,
    WebContentType,
};

pub struct SwaggerMiddleware {}

#[async_trait]
impl HttpServerMiddleware for SwaggerMiddleware {
    async fn handle_request(&self, ctx: HttpContext) -> Result<MiddleWareResult, HttpFailResult> {
        let path = ctx.get_path_lower_case();

        if !path.starts_with("/swagger") {
            return Ok(MiddleWareResult::Next(ctx));
        }

        if path == "/swagger/index.html" {
            let result = HttpOkResult::Content {
                content_type: Some(WebContentType::Html),
                content: get_swagger_index().to_vec(),
            };
            return Ok(MiddleWareResult::Ok(result));
        }

        let scheme = ctx.get_scheme();

        let host = ctx.get_host();

        if path == "/swagger" {
            let new_url = format!("{}://{}/swagger/index.html", scheme, host);
            return Ok(MiddleWareResult::Ok(HttpOkResult::Redirect {
                url: new_url,
            }));
        }

        if path == "/swagger/v1/swagger.json" {
            let mut placehloders = HashMap::new();

            placehloders.insert("SCHEME", ctx.get_scheme());

            placehloders.insert("HOST", host.to_string());
            placehloders.insert("VERSION", crate::app::APP_VERSION.to_string());

            let result = super::files::serve_file_with_placeholders(
                format!("./wwwroot{}", path).as_str(),
                None,
                &placehloders,
            )
            .await?;

            return Ok(MiddleWareResult::Ok(result));
        }

        let result = super::files::get(format!("./wwwroot{}", path).as_str()).await;

        match result {
            Ok(content) => {
                let result = HttpOkResult::Content {
                    content_type: None,
                    content,
                };
                return Ok(MiddleWareResult::Ok(result));
            }
            _ => {
                let new_url = format!("{}://{}/swagger/index.html", scheme, host);
                return Ok(MiddleWareResult::Ok(HttpOkResult::Redirect {
                    url: new_url,
                }));
            }
        }
    }
}

fn get_swagger_index() -> &'static [u8] {
    let result = r###"
    <!-- HTML for static distribution bundle build -->
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <title>Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="/swagger/swagger-ui.css" >
    <link rel="icon" type="image/png" href="/swagger/favicon-32x32.png" sizes="32x32" />
    <link rel="icon" type="image/png" href="/swagger/favicon-16x16.png" sizes="16x16" />
    <style>
      html
      {
        box-sizing: border-box;
        overflow: -moz-scrollbars-vertical;
        overflow-y: scroll;
      }

      *,
      *:before,
      *:after
      {
        box-sizing: inherit;
      }

      body
      {
        margin:0;
        background: #fafafa;
      }
    </style>
    
    
  </head>

  <body>
    <div id="swagger-ui"></div>

    <script src="/swagger/swagger-ui-bundle.js"> </script>
    <script src="/swagger/swagger-ui-standalone-preset.js"> </script>
    <script>
window.onload = function() {
  var url = window.location.search.match(/url=([^&]+)/);
  if (url && url.length > 1) {
    url = decodeURIComponent(url[1]);
  } else {
    url = undefined;
  }
  var urls = [{"url":"/swagger/v1/swagger.json","name":"v1"}];

  const disableTryItOutPlugin = function() {
    return {
      statePlugins: {
      spec: {
        wrapSelectors: {
          allowTryItOutFor: function() {
            return function() {
                return true;
              }
            }
          }
        }
      }
    }
  }

  // Build a system
  var ui = SwaggerUIBundle({
    url: url,
    urls: urls,  
    validatorUrl: null,
    oauth2RedirectUrl: window.location.origin + "/swagger/oauth2-redirect.html",

    docExpansion: "none", 
    operationsSorter: "none", 
    defaultModelsExpandDepth: 1, 
    defaultModelExpandDepth: 1, 
    tagsSorter: "none", 
    
    dom_id: '#swagger-ui',
    deepLinking: true,
    presets: [
      SwaggerUIBundle.presets.apis,
      SwaggerUIStandalonePreset
    ],
    plugins: [
        SwaggerUIBundle.plugins.DownloadUrl,
        disableTryItOutPlugin
    ],
    layout: "StandaloneLayout"
  });

  if ("client_id") {
    ui.initOAuth({
      clientId: "client_id",
      clientSecret: "client_secret",
      realm: "realm",
      appName: "app_name",
      scopeSeparator: " ",
      additionalQueryStringParams: {},
      usePkceWithAuthorizationCodeGrant: false
    });
  }

  window.ui = ui;
}
    </script>
    
  </body>
</html>

    "###;

    result.as_bytes()
}
