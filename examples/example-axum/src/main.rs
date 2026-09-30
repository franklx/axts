use std::sync::Arc;

use axts::{
    axum::ApiRouter,
    openapi::{OpenApi, Tag},
    transform::TransformOpenApi,
};
use axum::{http::StatusCode, Extension, Json};
use docs::docs_routes;
use errors::AppError;
use state::AppState;
use todos::routes::todo_routes;
use tokio::net::TcpListener;
use uuid::Uuid;

pub mod docs;
pub mod errors;
pub mod state;
pub mod todos;

#[tokio::main]
async fn main() {
    axts::generate::on_error(|error| {
        println!("{error}");
    });

    let state = AppState::default();

    let mut api = OpenApi::default();

    let app = ApiRouter::new()
        .nest_api_service("/todo", todo_routes(state.clone()))
        .nest_api_service("/docs", docs_routes(state.clone()))
        .finish_api_with(&mut api, api_docs)
        .layer(Extension(Arc::new(api.clone()))) // Arc is very important here or you will face massive memory and performance issues
        .with_state(state);

    // Instead of an Open API document, axts now generates TypeScript
    // types (via `ts-rs`) and a typed `fetch` client for the routes
    // above.
    if let Err(err) = axts::generate::export_types("client/_/api", true) {
        eprintln!("failed to export TypeScript types: {err}");
    } else {
        println!("TypeScript types written to ./client/_/api");
    }

    let client = axts::typescript::to_client(&api, &Default::default());
    if let Err(err) = std::fs::write("client/_/api/index.ts", client) {
        eprintln!("failed to write generated TypeScript client: {err}");
    } else {
        println!("TypeScript client written to ./client/_/api/index.ts");
    }

    println!("Example docs are accessible at http://127.0.0.1:3000/docs");

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Axts axum Open API")
        .summary("An example Todo application")
        .description(include_str!("README.md"))
        .tag(Tag {
            name: "todo".into(),
            description: Some("Todo Management".into()),
            ..Default::default()
        })
        .security_scheme(
            "ApiKey",
            axts::openapi::SecurityScheme::ApiKey {
                location: axts::openapi::ApiKeyLocation::Header,
                name: "X-Auth-Key".into(),
                description: Some("A key that is ignored.".into()),
                extensions: Default::default(),
            },
        )
        .default_response_with::<Json<AppError>, _>(|res| {
            res.example(AppError {
                error: "some error happened".to_string(),
                error_details: None,
                error_id: Uuid::nil(),
                // This is not visible.
                status: StatusCode::IM_A_TEAPOT,
            })
        })
}
