use askama::Template;
use axum::{
    Form, Router,
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use sqlx::sqlite::SqlitePool;
use thiserror::Error;

/////////////
// Errors //
/////////////

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Template error: {0}")]
    Template(#[from] askama::Error),
    #[error("Invalid attending value")]
    InvalidAttending,
    #[error("Name and email must be unique")]
    DuplicateRsvp,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::InvalidAttending | AppError::DuplicateRsvp => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}

////////////////////
// HTML templates //
////////////////////
#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage {
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

#[allow(dead_code)]
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

//////////////
// Database //
//////////////

async fn db_add_rsvp(pool: &SqlitePool, rsvp: RsvpNew) -> Result<(), AppError> {
    let attending = match rsvp.attending.as_str() {
        "yes" => 1,
        "no" => 0,
        _ => return Err(AppError::InvalidAttending),
    };

    sqlx::query_as!(
        Rsvp,
        "INSERT INTO rsvps (name, email, attending) VALUES (?, ?, ?) RETURNING *",
        rsvp.name,
        rsvp.email,
        attending
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(_) => AppError::DuplicateRsvp,
        _ => AppError::Database(e),
    })?;

    Ok(())
}

async fn get_rsvps(pool: &SqlitePool) -> Result<Vec<Rsvp>, AppError> {
    let rsvps = sqlx::query_as!(Rsvp, "SELECT * from rsvps")
        .fetch_all(pool)
        .await?;

    Ok(rsvps)
}

/////////////////////
// Include script  //
/////////////////////

const APP_SCRIPT: &str = include_str!("../static/frames.js");

async fn frames_js_handler() -> impl axum::response::IntoResponse {
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/javascript")],
        APP_SCRIPT,
    )
}

////////////
// Routes //
////////////

async fn index(State(pool): State<SqlitePool>) -> Result<Html<String>, AppError> {
    let rsvps = get_rsvps(&pool).await?;
    let page = IndexPage { rsvps }.render()?;
    Ok(Html(page))
}

async fn add_rsvp(
    State(pool): State<SqlitePool>,
    Form(rsvp): Form<RsvpNew>,
) -> Result<Html<String>, AppError> {
    db_add_rsvp(&pool, rsvp).await?;
    let rsvps = get_rsvps(&pool).await?;
    let rsvp_html = RsvpList { rsvps }.render()?;
    Ok(Html(rsvp_html))
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite:./partifuller.db").await?;

    let app = Router::new()
        .route("/", get(index))
        .route("/rsvp", post(add_rsvp))
        .route("/static/frames.js", get(frames_js_handler))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
