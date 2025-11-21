use axum::{Router, http::StatusCode, response::Html, routing::get};
use askama::Template;

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

struct Rsvp {
    name: String,
}

async fn index() -> Result<Html<String>, StatusCode> {
    let mut rsvps = Vec::new();
    rsvps.push(Rsvp {
        name: "Roland".to_string(),
    });
    let page = Home { rsvps }
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(page))
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(index));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
