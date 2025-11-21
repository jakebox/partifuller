use axum::{
    routing::get,
    Router,
    response::{Html, IntoResponse},
    http::StatusCode,
};

use askama::Template;

#[derive(Template)]
#[template(
    ext = "html",
    source = "<p>© {{ year }} {{ enterprise|upper }}</p>"
)]
struct Footer<'a> {
    year: u16,
    enterprise: &'a str,
}

async fn index() -> Result<Html<String>, StatusCode> {
    let footer = Footer {
        year: 2024,
        enterprise: "My Company",
    }
    .render()
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Html(footer))
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(index));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}