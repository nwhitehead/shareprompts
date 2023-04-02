#[macro_use]
extern crate diesel;

mod schema;

use actix_web::{error, get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use diesel::{prelude::*, r2d2};

// Types related to Postgres connection to database
type DbConnection = diesel::pg::PgConnection;
type DbConnectionManager = r2d2::ConnectionManager<DbConnection>;
type DbPool = r2d2::Pool<DbConnectionManager>;
type DbError = Box<dyn std::error::Error + Send + Sync>;

use schema::posts;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub public: bool,
    pub research: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost {
    pub title: String,
    pub body: String,
    pub public: bool,
    pub research: bool,
}

fn find_post_by_id(
    conn: &mut DbConnection,
    id: i32,
) -> Result<Option<i32>, DbError> {
    use self::schema::posts::dsl::*;
    let results = posts
        .filter(public.eq(true))
        .limit(5)
        .load::<Post>(conn)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in results {
        println!("{}", post.title);
        println!("-----------\n");
        println!("{}", post.body);
    }

    Ok(Some(5))
}

fn new_post(
    conn: &mut DbConnection,
    post: NewPost,
) -> Result<i32, DbError> {
    use self::schema::posts::dsl::*;

    let mut inserted_post = diesel::insert_into(posts)
        .values(post)
        .get_result::<Post>(conn)
        .expect("Error saving new post");
    Ok(inserted_post.id)
}

#[get("/api/conversation/{id}")]
async fn get_conversation(
    pool: web::Data<DbPool>,
    id: web::Path<(i32,)>
) -> actix_web::Result<impl Responder> {
    println!("Got a GET");
    let uid = id.into_inner().0;
    // Don't block server thread
    let post: Option<i32> = web::block(move || {
        let mut conn = pool.get()?;
        find_post_by_id(&mut conn, uid)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(
        HttpResponse::Ok().body(format!("Getting post uid={}\npost={}", uid, post.unwrap()))
    )
}

#[post("/api/conversation")]
async fn post_conversation(
    pool: web::Data<DbPool>,
    form: web::Json<NewPost>,
) -> actix_web::Result<impl Responder> {
    println!("Got a POST");
    let postid = web::block(move || {
        let mut conn = pool.get()?;
        let np = NewPost { title: form.title.clone(), body: form.body.clone(), public: form.public, research: form.research};
        new_post(&mut conn, np)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(
        HttpResponse::Created().json(postid)
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
            .service(get_conversation)
            .service(post_conversation)
    })
    .bind("0.0.0.0:9090")?
    .run()
    .await
}

fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = DbConnectionManager::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to Postgres database with username and password")
}
