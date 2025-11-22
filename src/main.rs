use askama::Template;
use axum::{
    Form, Router,
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, get_service, post},
};
use sqlx::sqlite::SqlitePool;
use tower_http::services::ServeDir;

////////////////////
// HTML templates //
////////////////////
#[derive(Template)]
#[template(
    ext = "html",
    source = r#"
{% extends "base.html" %}
{% block content %}
{% include "rsvp_form.html" %}
<div id="rsvp-result">
{% include "rsvp_list.html" %}
</div>
{% endblock %}
"#
)]
struct Home {
    rsvps: Vec<Rsvp>,
}

#[derive(Template)]
#[template(path = "rsvp_list.html")]
struct RsvpList {
    rsvps: Vec<Rsvp>,
}

/////////////
// Structs //
/////////////

#[derive(sqlx::FromRow)]
struct Rsvp {
    id: i64,
    name: String,
    email: String,
    timestamp: i64,
    attending: i64,
}

#[derive(serde::Deserialize)]
struct RsvpNew {
    name: String,
    email: String,
    attending: String,
}

////////////
// Routes //
////////////

async fn index(State(pool): State<SqlitePool>) -> Result<Html<String>, StatusCode> {
    let rsvps = sqlx::query_as("SELECT * from rsvps")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let page = Home { rsvps }
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(page))
}

async fn add_rsvp(
    State(pool): State<SqlitePool>,
    Form(rsvp): Form<RsvpNew>,
) -> Result<Html<String>, (StatusCode, String)> {

    let attending = match rsvp.attending.as_str() {
        "yes" => 1,
        "no" => 0,
        _ => return Err((StatusCode::BAD_REQUEST, "FAIL".to_string())),
    };

    let _result =
        sqlx::query("INSERT INTO rsvps (name, email, attending) VALUES (?, ?, ?) RETURNING *")
            .bind(rsvp.name.trim())
            .bind(rsvp.email.trim())
            .bind(attending)
            .fetch_one(&pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Your name and email must be unique!".to_string(),
                )
            })?;

    let rsvps = sqlx::query_as("SELECT * from rsvps")
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to load RSVPs: {}", e),
            )
        })?;

    let rsvp_html = RsvpList { rsvps }.render().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to render page: {}", e),
        )
    })?;

    Ok(Html(rsvp_html))
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite:./partifuller.db").await?;

    let app = Router::new()
        .route("/", get(index))
        .route("/rsvp", post(add_rsvp))
        .nest_service("/static", get_service(ServeDir::new("./static")))
        .with_state(pool);


    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    println!("FAIL");

    Ok(())
}
