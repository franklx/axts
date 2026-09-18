use std::sync::Arc;

use crate::state::AppState;
use aide::{
    axum::{
        routing::{get_with},
        ApiRouter, IntoApiResponse,
    },
    openapi::OpenApi,
    NoApi,
};
use axum::{response::Html, Extension, Json};

pub fn docs_routes(state: AppState) -> ApiRouter {
    // We infer the return types for these routes
    // as an example.
    aide::generate::infer_responses(true);

    let router: ApiRouter = ApiRouter::new()
        .api_route_with(
            "/",
            get_with(serve_index, |op| {
                op.description("This documentation page.")
            }),
            |p| p.security_requirement("ApiKey"),
        )
        .api_route_with(
            "/private/api.json",
            get_with(serve_api_manifest, |op| {
                op.description("A JSON manifest of the routes known to `aide`.")
            }),
            |p| p.security_requirement("ApiKey"),
        )
        .api_route_with(
            "/private/client.ts",
            get_with(serve_client, |op| {
                op.description("The generated TypeScript API client.")
            }),
            |p| p.security_requirement("ApiKey"),
        )
        .with_state(state);

    // Afterwards we disable response inference because
    // it might be incorrect for other routes.
    aide::generate::infer_responses(false);

    router
}

async fn serve_index() -> impl IntoApiResponse {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
  <head><title>Aide Axum Worker example</title></head>
  <body>
    <h1>Aide Axum Worker example</h1>
    <p>This example generates TypeScript instead of an Open API UI.</p>
    <ul>
      <li><a href="/docs/private/api.json">Route manifest (JSON)</a></li>
      <li><a href="/docs/private/client.ts">Generated TypeScript client</a></li>
    </ul>
  </body>
</html>"#,
    )
}

/// A JSON manifest of the routes known to `aide`, including the name of
/// the TypeScript type used for each parameter/request body/response.
///
/// `OpenApi` is just bookkeeping and does not derive `TS` itself, so we
/// wrap the response in [`NoApi`] to serve it without needing to
/// document its own shape.
async fn serve_api_manifest(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    NoApi(Json(api))
}

/// Serves a typed TypeScript `fetch` client generated from the routes.
///
/// Note: unlike the `example-axum` example, this Worker cannot write
/// the corresponding TypeScript type declarations to disk (there is no
/// filesystem), so consumers of this endpoint are expected to run
/// `aide::generate::export_types` from a regular (non-Worker) build to
/// obtain the accompanying `.ts` type files.
async fn serve_client(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    aide::typescript::to_client(&api, &Default::default())
}
