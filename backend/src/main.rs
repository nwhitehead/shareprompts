extern crate diesel;

mod schema;

use log::{info};
use actix_web::{delete, error, get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use diesel::{prelude::*, r2d2};
use serde::{Deserialize, Serialize};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use handlebars::{handlebars_helper, Handlebars};
use chrono::offset::Utc;
use chrono::DateTime;

// Types related to Postgres connection to database
type DbConnection = diesel::pg::PgConnection;
type DbConnectionManager = diesel::r2d2::ConnectionManager<DbConnection>;
type DbPool = diesel::r2d2::Pool<DbConnectionManager>;
type DbError = Box<dyn std::error::Error + Send + Sync>;

use schema::conversations;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

// Templates
const INDEX_HBS: &str = include_str!("../site/index.hbs");
const INDEX_CSS: &str = include_str!("../dist/index.css");
const CHATGPT_PNG: &[u8] = include_bytes!("../site/chatgpt.png");
const MAIN_JS: &str = include_str!("../dist/main.js");

// Check for string equality
handlebars_helper!(nargs: |*args| args.len());
handlebars_helper!(string_equal: |*args| args[0] == args[1]);

// Model for conversations in the database with all fields
#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = conversations)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub contents: String, // JSON for ConversationContents
    pub model: String,
    pub public: bool,
    pub research: bool,
    pub creationdate: std::time::SystemTime,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Utterance {
    pub who: String, // either "gpt" or "human"
    pub what: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationContents {
    pub avatar: String, // data URL of avatar, (may be anonymized)
    pub dialog: Vec<Utterance>,
}

// Information that is required when making a new conversation
#[derive(Serialize, Deserialize)]
pub struct NewConversation {
    pub title: String,
    pub contents: ConversationContents,
    pub model: String,
    pub public: bool,
    pub research: bool,
}

// Information returned from GET
#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationInfo {
    pub id: String,
    pub title: String,
    pub contents: ConversationContents,
    pub public: bool,
    pub research: bool,
    pub model: String,
    pub creationdate: std::time::SystemTime,
}

// Look in DB for specific ID an return DB Conversation if found
fn find_conversation_by_id(
    conn: &mut DbConnection,
    convo_id: &String,
) -> Result<Option<Conversation>, DbError> {
    use self::schema::conversations::dsl::*;
    let results = conversations
        .filter(id.eq(convo_id))
        .limit(1)
        .load::<Conversation>(conn)
        .expect("Error finding conversation");

    if results.len() == 0 {
        Ok(None)
    } else {
        let result = results[0].clone();
        if result.public {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

// Look in DB for specific ID an return DB Conversation if found
fn find_conversations_by_user(
    conn: &mut DbConnection,
    uid: &String,
) -> Result<Vec<ConversationInfo>, DbError> {
    use self::schema::conversations::dsl::*;
    conversations
        .filter(user_id.eq(uid))
        .load::<Conversation>(conn)
        .expect("Error finding conversation")
        .iter()
        .map(|conv| Ok(ConversationInfo {
            id: conv.id.clone(),
            title: conv.title.clone(),
            contents: serde_json::from_str(&conv.contents)?,
            public: conv.public,
            research: conv.research,
            model: conv.model.clone(),
            creationdate: conv.creationdate,
        }))
        .collect()
}

#[get("/conversation/json/{id}")]
async fn get_conversation_json(
    pool: web::Data<DbPool>,
    id: web::Path<(String,)>,
) -> actix_web::Result<impl Responder> {
    let uid = id.into_inner().0;
    // Don't block server thread, db stuff is synchronous
    let conversation = web::block(move || {
        let mut conn = pool.get()?;
        find_conversation_by_id(&mut conn, &uid)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    match conversation {
        Some(conv) => {
            let conversation_info = ConversationInfo {
                id: conv.id.clone(),
                title: conv.title.clone(),
                contents: serde_json::from_str(&conv.contents)?,
                public: conv.public,
                research: conv.research,
                model: conv.model,
                creationdate: conv.creationdate,
            };
            Ok(HttpResponse::Ok().json(conversation_info))
        }
        None => {
            Ok(HttpResponse::NotFound().body(format!(
                "Not found"
            )))
        }
    }
}

#[get("/conversation/html/{id}")]
async fn get_conversation_html(
    pool: web::Data<DbPool>,
    id: web::Path<(String,)>,
) -> actix_web::Result<impl Responder> {
    let uid = id.into_inner().0;
    // Don't block server thread, db stuff is synchronous
    let conversation = web::block(move || {
        let mut conn = pool.get()?;
        find_conversation_by_id(&mut conn, &uid)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    match conversation {
        Some(conv) => {
            let mut reg = Handlebars::new();
            reg.register_helper("string_equal", Box::new(string_equal));
            let contents: ConversationContents = serde_json::from_str(&conv.contents)?;
            let chatgpt_uri: String = format!("data:image/png;base64,{}", base64::encode(CHATGPT_PNG));
            let timestamp: DateTime<Utc> = conv.creationdate.into();
            let timestamp_str: String = format!("{}", timestamp.format("%Y/%m/%d %T UTC"));
            let body = reg.render_template(INDEX_HBS, &serde_json::json!({
                "style": INDEX_CSS,
                "title": conv.title,
                "model": conv.model,
                "avatar": contents.avatar,
                "chatgpt_uri": chatgpt_uri,
                "dialog": contents.dialog,
                "timestamp": timestamp_str,
                "main_js": MAIN_JS,
            })).map_err(error::ErrorInternalServerError)?;
            Ok(HttpResponse::Ok().body(body))
        }
        None => {
            Ok(HttpResponse::NotFound().body(format!(
                "Not found"
            )))
        }
    }
}

#[get("/conversations")]
async fn get_my_conversations(
    auth: BearerAuth,
    pool: web::Data<DbPool>,
) -> actix_web::Result<impl Responder> {
    let token = auth.token();
    info!("Bearer token was: {}", token);
    let client = awc::Client::new();
    let google_validate_url = format!("https://www.googleapis.com/oauth2/v1/tokeninfo?id_token={}", &token);
    let res = client
        .get(google_validate_url)
        .send()
        .await
        .map_err(error::ErrorInternalServerError)?
        .json::<GoogleTokenCheckResponse>()
        .await;
    let userid =
        match res {
            Ok(resok) => resok.user_id,
            Err(_) => return Ok(HttpResponse::Unauthorized().body("Token authorization failed"))
        };
    // Don't block server thread, db stuff is synchronous
    let conversations = web::block(move || {
        let mut conn = pool.get()?;
        find_conversations_by_user(&mut conn, &userid)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(conversations))
}

#[derive(Debug)]
enum LocalError {
    DbConnectionProblem,
    SerializationFailed,
    DbError,
    AuthorizationProblem,
}

impl std::fmt::Display for LocalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            LocalError::DbConnectionProblem => write!(f, "problem with r2d2 db connection"),
            LocalError::SerializationFailed => write!(f, "json serialization of contents failed"),
            LocalError::DbError => write!(f, "problem with db connection"),
            LocalError::AuthorizationProblem => write!(f, "authorization problem"),
        }
    }
}
impl std::convert::From<r2d2::PoolError> for LocalError {
    fn from(_err: r2d2::PoolError) -> LocalError {
        LocalError::DbConnectionProblem
    }
}
impl std::convert::From<serde_json::Error> for LocalError {
    fn from(_err: serde_json::Error) -> LocalError {
        LocalError::SerializationFailed
    }
}
impl std::convert::From<DbError> for LocalError {
    fn from(_err: DbError) -> LocalError {
        LocalError::DbError
    }
}

#[derive(Deserialize)]
struct GoogleTokenCheckResponse
{
    user_id: String
}

#[post("/conversation/")]
async fn post_conversation(
    auth: BearerAuth,
    pool: web::Data<DbPool>,
    form: web::Json<NewConversation>,
) -> actix_web::Result<impl Responder> {
    let client = awc::Client::new();
    let google_validate_url = format!("https://www.googleapis.com/oauth2/v1/tokeninfo?access_token={}", auth.token());
    let res = client
        .get(google_validate_url)
        .send()
        .await
        .map_err(error::ErrorInternalServerError)?
        .json::<GoogleTokenCheckResponse>()
        .await;
    let userid =
        match res {
            Ok(resok) => resok.user_id,
            Err(_) => return Ok(HttpResponse::Unauthorized().body("Token authorization failed"))
        };
    let convo_id = web::block(move || -> Result<String, LocalError> {
        let json_contents = serde_json::to_string(&form.contents)?;
        let mut conn = pool.get()?;
        let new_uuid = uuid::Uuid::new_v4().simple().to_string();
        let nc = Conversation {
            id: new_uuid.clone(),
            title: form.title.clone(),
            contents: json_contents,
            model: form.model.clone(),
            public: form.public,
            research: form.research,
            creationdate: chrono::Utc::now().into(),
            user_id: userid,
        };
        use self::schema::conversations::dsl::*;
        diesel::insert_into(conversations)
            .values(nc)
            .execute(& mut conn)
            .expect("Error saving new conversation");
        Ok(new_uuid)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Created().json(convo_id))
}

fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = DbConnectionManager::new(conn_spec);
    diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to Postgres database with username and password")
}

#[delete("/conversation/{id}")]
async fn delete_conversation(
    auth: BearerAuth,
    pool: web::Data<DbPool>,
    id: web::Path<(String,)>,
) -> actix_web::Result<impl Responder> {
    let uid = id.into_inner().0;
    let client = awc::Client::new();
    let google_validate_url = format!("https://www.googleapis.com/oauth2/v1/tokeninfo?access_token={}", auth.token());
    let res = client
        .get(google_validate_url)
        .send()
        .await
        .map_err(error::ErrorInternalServerError)?
        .json::<GoogleTokenCheckResponse>()
        .await;
    let userid =
        match res {
            Ok(resok) => resok.user_id,
            Err(_) => return Ok(HttpResponse::Unauthorized().body("Token authorization failed"))
        };
    web::block(move || -> Result<(), LocalError> {
        let mut conn = pool.get()?;
        let convo = find_conversation_by_id(&mut conn, &uid)?;
        match convo {
            Some(conv) => {
                use self::schema::conversations::dsl::*;
                if conv.user_id != userid {
                    Err(LocalError::AuthorizationProblem)
                } else {
                    diesel::delete(conversations.filter(id.eq(uid)))
                        .execute(& mut conn)
                        .expect("Error deleting conversation");
                    Ok(())
                }
            }
            _ => {
                Err(LocalError::AuthorizationProblem)
            }
        }
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().into())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set debug log level by default unless you set things manually from .env file
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // Initialize database pool outside server and copy it in
    let pool = initialize_db_pool();
    let mut conn = pool.get().expect("db pool could not produce a connection");
    // Check for pending migrations
    info!("Checking for pending database migrations (stored internally to binary)");
    let cnt = conn.pending_migrations(MIGRATIONS).expect("could not get list of migrations").len();
    if cnt > 0 {
        info!("Applying {} pending migrations", cnt);
        conn.run_pending_migrations(MIGRATIONS).expect("could not run pending migrations");
    }

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .service(get_conversation_json)
            .service(get_conversation_html)
            .service(post_conversation)
            .service(delete_conversation)
            .service(get_my_conversations)
    })
    .bind("0.0.0.0:9090")?
    .run()
    .await
}
