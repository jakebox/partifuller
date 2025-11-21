use axum::{
    Form, Router, extract::State, http::StatusCode, response::Html, routing::{get, post}
};

use askama::Template;
use sqlx::sqlite::SqlitePool;

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
    let _result =
        sqlx::query("INSERT INTO rsvps (name, email, attending) VALUES (?, ?, ?) RETURNING *")
            .bind(rsvp.name)
            .bind(rsvp.email)
            .bind(rsvp.attending)
            .fetch_one(&pool)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Not good".to_string()))?;

    let rsvps = sqlx::query_as("SELECT * from rsvps")
        .fetch_all(&pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "EE".to_string()))?;

    let rsvp_html = RsvpList { rsvps }
        .render()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "EE".to_string()))?;

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
