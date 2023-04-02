#[macro_use]
extern crate diesel;

use actix_web::{error, get, post, web, App, HttpResponse, HttpServer, Responder};
//use serde::{Deserialize, Serialize};
use diesel::{prelude::*, r2d2};

// Types related to Postgres connection to database
type ConnectionManager = r2d2::ConnectionManager<diesel::pg::PgConnection>;
type DbPool = r2d2::Pool<ConnectionManager>;
type DbError = Box<dyn std::error::Error + Send + Sync>;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!\n")
}

fn find_user_by_id(id: i32) -> Result<Option<i32>, DbError> {
    Ok(Some(5))
}

#[get("/post/{id}")]
async fn get_post(
    pool: web::Data<DbPool>,
    id: web::Path<(i32,)>
) -> actix_web::Result<impl Responder> {
    let uid = id.into_inner().0;
    // Don't block server thread
    let user: Option<i32> = web::block(move || {
        let mut conn = pool.get()?;
        //actions::find_user_by_uid(&mut conn, user_uid)
        find_user_by_id(uid)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(
        HttpResponse::Ok().body(format!("Getting post uid={}\nuser={}", uid, user.unwrap()))
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set debug log level by default unless you set things manually from .env file
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    // Initialize database pool outside server and copy it in
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

fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = ConnectionManager::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to Postgres database with username and password")
}
