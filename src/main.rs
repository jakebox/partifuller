use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Form, Router,
};
use askama::Template;
use sqlx::sqlite::SqlitePool;

////////////////////
// HTML templates //
////////////////////
#[derive(Template)]
#[template(
    ext = "html",
    source = r#"
{% extends "base.html" %}
{% block content %}
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
#[template(path="rsvp_list.html")]
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
    attending: i64,
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
    // Validate input
    if rsvp.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name is required".to_string()));
    }
    if rsvp.email.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Email is required".to_string()));
    }
    if rsvp.attending < 0 {
        return Err((StatusCode::BAD_REQUEST, "Attending must be a positive number".to_string()));
    }

    let _result =
        sqlx::query("INSERT INTO rsvps (name, email, attending) VALUES (?, ?, ?) RETURNING *")
            .bind(rsvp.name.trim())
            .bind(rsvp.email.trim())
            .bind(rsvp.attending)
            .fetch_one(&pool)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Your name and email must be unique!".to_string()))?;

    let rsvps = sqlx::query_as("SELECT * from rsvps")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load RSVPs: {}", e)))?;

    let rsvp_html = RsvpList { rsvps }
        .render()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to render page: {}", e)))?;

    Ok(Html(rsvp_html))
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite:./partifuller.db").await?;

    // build our application with a single route
    let app = Router::new()
        .route("/", get(index))
        .route("/rsvp", post(add_rsvp))
        .with_state(pool);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3500").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    println!("FAIL");

    Ok(())
}
