#[macro_use]
extern crate diesel;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
//use serde::{Deserialize, Serialize};
use diesel::pg::PgConnection;
use diesel::{prelude::*, r2d2};
use dotenvy::dotenv;
use env_logger::init_from_env;
use std::env;

// Database pool type for Postgres connections
type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!\n")
}

#[get("/post/{id}")]
async fn get_post(id: web::Path<(i32,)>) -> impl Responder {
    HttpResponse::Ok().body(format!("Getting post id={}\n", id.into_inner().0))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // initialize DB pool outside of `HttpServer::new` so that it is shared across all workers
    dotenv().ok();
    // Set debug log level by default unless you set things manually from .env file
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let pool = initialize_db_pool();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(hello)
            .service(get_post)
    })
    .bind("0.0.0.0:9090")?
    .run()
    .await
}

/// Initialize database connection pool based on `DATABASE_URL` environment variable.
///
/// See more: <https://docs.rs/diesel/latest/diesel/r2d2/index.html>.
fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to Postgres database with username and password")
}
