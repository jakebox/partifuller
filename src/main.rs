use axum::{
    Router, extract::State, http::StatusCode, response::{Html}, routing::get
};

use askama::Template;
use sqlx::sqlite::SqlitePool;

#[derive(Template)]
#[template(
    ext = "html",
    source = r#"
{% extends "base.html" %}
{% block content %}
<ul>
{% for rsvp in rsvps %}
  <li>{{ rsvp.name }}</li>
{% endfor %}
</ul>
{% endblock %}
"#
)]
struct Home {
    rsvps: Vec<Rsvp>,
}

#[derive(sqlx::FromRow)]
struct Rsvp {
    name: String,
}

async fn index(
    State(pool): State<SqlitePool>,
) -> Result<Html<String>, StatusCode> {
    let rsvps = sqlx::query_as("SELECT name from rsvps")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let page = Home { rsvps }
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(page))
}

#[tokio::main]
async fn main()  -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite:./partifuller.db").await?;

    // build our application with a single route
    let app = Router::new().route("/", get(index)).with_state(pool);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
