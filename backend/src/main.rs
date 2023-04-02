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

use schema::conversations;

// Model for conversations in the database with all fields
#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = conversations)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub body: String,
    pub public: bool,
    pub research: bool,
}

// Information that is required when making a new conversation
#[derive(Serialize, Deserialize)]
pub struct NewConversation {
    pub title: String,
    pub body: String,
    pub public: bool,
    pub research: bool,
}

fn find_conversation_by_id(
    conn: &mut DbConnection,
    convo_id: String,
) -> Result<Option<Conversation>, DbError> {
    use self::schema::conversations::dsl::*;
    let results = conversations
        .filter(id.eq(convo_id))
        .limit(1)
        .load::<Conversation>(conn)
        .expect("Error finding post");

    if results.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(results[0].clone()))
    }
}

fn new_conversation(
    conn: &mut DbConnection,
    conversation: Conversation,
) -> Result<String, DbError> {
    use self::schema::conversations::dsl::*;
    let new_uuid = conversation.id.clone();
    diesel::insert_into(conversations)
        .values(conversation)
        .execute(conn)
        .expect("Error saving new conversation");
    Ok(new_uuid)
}

#[get("/api/conversation/{id}")]
async fn get_conversation(
    pool: web::Data<DbPool>,
    id: web::Path<(String,)>
) -> actix_web::Result<impl Responder> {
    println!("Got a GET");
    let uid = id.into_inner().0;
    // Don't block server thread
    let conversation = web::block(move || {
        let mut conn = pool.get()?;
        find_conversation_by_id(&mut conn, uid)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(
        HttpResponse::Ok().body(format!("Getting conversation conversation.title={}", conversation.unwrap().title))
    )
}

#[post("/api/conversation")]
async fn post_conversation(
    pool: web::Data<DbPool>,
    form: web::Json<NewConversation>,
) -> actix_web::Result<impl Responder> {
    println!("Got a POST");
    let convo_id = web::block(move || {
        let mut conn = pool.get()?;
        let new_uuid = uuid::Uuid::new_v4().simple().to_string();
        let nc = Conversation { id: new_uuid, title: form.title.clone(), body: form.body.clone(), public: form.public, research: form.research};
        new_conversation(&mut conn, nc)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(
        HttpResponse::Created().json(convo_id)
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
