extern crate diesel;

mod schema;

use actix_web::{
    delete, error, get, middleware, post, web, App, HttpResponse, HttpServer, Responder,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::offset::Utc;
use chrono::DateTime;
use diesel::{prelude::*, r2d2};
use handlebars::{handlebars_helper, Handlebars};
use log::info;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

// Types related to Postgres connection to database
type DbConnection = diesel::pg::PgConnection;
type DbConnectionManager = diesel::r2d2::ConnectionManager<DbConnection>;
type DbPool = diesel::r2d2::Pool<DbConnectionManager>;
type DbError = Box<dyn std::error::Error + Send + Sync>;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use schema::conversations;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

// Templates
const INDEX_HBS: &str = include_str!("../site/index.hbs");
const INDEX_CSS: &str = include_str!("../dist/index.css");
const CHATGPT_PNG: &[u8] = include_bytes!("../site/chatgpt.png");
const MAIN_JS: &str = include_str!("../dist/main.js");

// Google keys
#[derive(Debug, Deserialize)]
struct JsonWebKey {
    r#use: String,
    kid: String,
    alg: String,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize)]
struct JsonWebKeySetResponse {
    keys: Vec<JsonWebKey>,
}

struct JsonWebKeysSet {
    keys: std::collections::HashMap<String, JsonWebKey>,
    exp: u64,
}

// Main AppData
struct AppState {
    jwks: std::sync::Mutex<JsonWebKeysSet>,
}

// JWT stuff
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    aud: String,
    sub: String,
    nbf: u64,
    exp: u64,
}

// Check for string equality
handlebars_helper!(string_equal: |*args| args[0] == args[1]);

// Model for conversations in the database with all fields
#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = conversations)]
pub struct Conversation {
    pub id: String,
    pub contents: String, // JSON for ConversationContents
    pub metadata: String, // JSON for ConversationMetadata
    pub public: bool,
    pub research: bool,
    pub deleted: bool,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub title: String,
    pub model: String,
    pub creationdate: std::time::SystemTime,
    pub length: usize,
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
    pub contents: ConversationContents,
    pub metadata: ConversationMetadata,
    pub public: bool,
    pub research: bool,
}

// Information returned from GET for list of conversations
#[derive(Debug, Serialize, Deserialize)]
pub struct ShortConversationInfo {
    pub id: String,
    pub metadata: ConversationMetadata,
    pub public: bool,
    pub research: bool,
}

#[derive(Debug)]
enum JWKSError {
    Retrieval,
    DecodingKeyError,
    NotFound,
}

// Refresh our collection of Google public keys
// Find correct one to use
async fn retrieve_key(
    jwks: &mut JsonWebKeysSet,
    id: &str,
) -> Result<jsonwebtoken::DecodingKey, JWKSError> {
    let should_retrieve = jwks.keys.len() == 0 || true;
    if should_retrieve {
        info!("Refreshing public keys");
        let google_keys_url = "https://www.googleapis.com/oauth2/v3/certs";
        let client = awc::Client::new();
        let res = client
            .get(google_keys_url)
            .send()
            .await
            .ok()
            .expect("Google needs to be accessible")
            .json::<JsonWebKeySetResponse>()
            .await;
        match res {
            Ok(payload) => {
                info!("Got payload {}", payload.keys.len());
                for key in payload.keys {
                    jwks.keys.insert(key.kid.clone(), key);
                }
                info!("New hashmap has size {}", jwks.keys.len());
            }
            Err(_) => return Err(JWKSError::Retrieval),
        }
    }
    // Now get the needed key from the hashmap
    let key = match jwks.keys.get(id) {
        Some(k) => k,
        None => {
            return Err(JWKSError::NotFound);
        }
    };
    let decoding_key = jsonwebtoken::DecodingKey::from_rsa_components(&key.n, &key.e)
        .map_err(|_err| JWKSError::DecodingKeyError)?;
    return Ok(decoding_key);
}

// Look in DB for specific ID an return DB Conversation if found
fn find_conversation_by_id(
    conn: &mut DbConnection,
    convo_id: &String,
    deleted_entry: bool,
) -> Result<Option<Conversation>, DbError> {
    use self::schema::conversations::dsl::*;
    let results = conversations
        .filter(id.eq(convo_id))
        .filter(deleted.eq(deleted_entry))
        .limit(1)
        .load::<Conversation>(conn)
        .expect("Error finding conversation");

    if results.len() == 0 {
        Ok(None)
    } else {
        let result = results[0].clone();
        Ok(Some(result))
    }
}

// Look in DB for all conversations of a user
fn find_conversations_by_user(
    conn: &mut DbConnection,
    uid: &String,
    deleted_entry: bool,
) -> Result<Vec<ShortConversationInfo>, DbError> {
    use self::schema::conversations::dsl::*;
    conversations
        .filter(user_id.eq(uid))
        .filter(deleted.eq(deleted_entry))
        .load::<Conversation>(conn)
        .expect("Error finding conversation")
        .iter()
        .map(|conv| {
            Ok(ShortConversationInfo {
                id: conv.id.clone(),
                metadata: serde_json::from_str(&conv.metadata)?,
                public: conv.public,
                research: conv.research,
            })
        })
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
        find_conversation_by_id(&mut conn, &uid, /*deleted=*/ false)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    match conversation {
        Some(conv) => {
            let conversation_info = ConversationInfo {
                id: conv.id.clone(),
                contents: serde_json::from_str(&conv.contents)?,
                metadata: serde_json::from_str(&conv.metadata)?,
                public: conv.public,
                research: conv.research,
            };
            Ok(HttpResponse::Ok().json(conversation_info))
        }
        None => Ok(HttpResponse::NotFound().body(format!("Not found"))),
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
        find_conversation_by_id(&mut conn, &uid, /*deleted=*/ false)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    match conversation {
        Some(conv) => {
            let mut reg = Handlebars::new();
            reg.register_helper("string_equal", Box::new(string_equal));
            let contents: ConversationContents = serde_json::from_str(&conv.contents)?;
            let metadata: ConversationMetadata = serde_json::from_str(&conv.metadata)?;
            let chatgpt_uri: String =
                format!("data:image/png;base64,{}", base64::encode(CHATGPT_PNG));
            let timestamp: DateTime<Utc> = metadata.creationdate.into();
            let timestamp_str: String = format!("{}", timestamp.format("%Y/%m/%d %T UTC"));
            let body = reg
                .render_template(
                    INDEX_HBS,
                    &serde_json::json!({
                        "style": INDEX_CSS,
                        "title": metadata.title,
                        "model": metadata.model,
                        "avatar": contents.avatar,
                        "chatgpt_uri": chatgpt_uri,
                        "dialog": contents.dialog,
                        "timestamp": timestamp_str,
                        "main_js": MAIN_JS,
                    }),
                )
                .map_err(error::ErrorInternalServerError)?;
            Ok(HttpResponse::Ok().body(body))
        }
        None => Ok(HttpResponse::NotFound().body(format!("Not found"))),
    }
}

enum TokenError {
    Invalid,
    DecodingKeyError,
    DecodeError,
    NotValidBefore,
    Expired,
    AudienceMismatch,
    Issuer,
    JWKSProblem,
}

// Extract needed key id from token without validating anything
fn get_kid(token: &str) -> Result<String, TokenError> {
    let header = jsonwebtoken::decode_header(&token).map_err(|_err| TokenError::Invalid)?;
    match header.kid {
        Some(key_id) => Ok(key_id),
        None => Err(TokenError::Invalid),
    }
}

/// Validate identity token and return user_id
async fn validate_bearer_identity_token(
    jwks: &mut JsonWebKeysSet,
    token: &str,
) -> Result<String, TokenError> {
    let google_project_id =
        std::env::var("GOOGLE_PROJECT_ID").expect("GOOGLE_PROJECT_ID should be set");
    let kid = get_kid(&token)?;
    let decoding_key = retrieve_key(jwks, &kid)
        .await
        .map_err(|_err| TokenError::JWKSProblem)?;
    let token_message = jsonwebtoken::decode::<Claims>(
        &token,
        &decoding_key,
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256),
    )
    .map_err(|_err| TokenError::DecodeError)?;
    info!(
        "Decoded claims in JWT, user_id={}",
        token_message.claims.sub
    );
    let start = std::time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards from UNIX_EPOCH")
        .as_secs() as u64;
    let slop = 2; // Allow a few seconds of wiggle room for clock skew
    if since_the_epoch + slop < token_message.claims.nbf {
        info!(
            "Current time is too early for bearer token, nbf={} time={}",
            token_message.claims.nbf, since_the_epoch
        );
        return Err(TokenError::NotValidBefore);
    }
    if since_the_epoch > token_message.claims.exp {
        info!(
            "Current time is too late for bearer token, exp={} time={}",
            token_message.claims.exp, since_the_epoch
        );
        return Err(TokenError::Expired);
    }
    if token_message.claims.aud != google_project_id {
        info!(
            "Google project id does not match token audience, aud={} google_project_id={}",
            token_message.claims.aud, google_project_id
        );
        return Err(TokenError::AudienceMismatch);
    }
    if token_message.claims.iss != "https://accounts.google.com" {
        info!("Token issuer was not https://accounts.google.com");
        return Err(TokenError::Issuer);
    }
    Ok(token_message.claims.sub)
}

#[derive(Deserialize)]
struct GoogleTokenCheckResponse {
    user_id: String,
}

/// Validate access token (from extension using chrome.identity.getAuthToken()) and return user_id
/// Validate token and return user_id
async fn validate_bearer_access_token(token: &str) -> Result<String, TokenError> {
    let google_validate_url = format!(
        "https://www.googleapis.com/oauth2/v1/tokeninfo?access_token={}",
        token
    );
    let client = awc::Client::new();
    let res = client
        .get(google_validate_url)
        .send()
        .await
        .map_err(|_err| TokenError::Invalid)?
        .json::<GoogleTokenCheckResponse>()
        .await;
    match res {
        Ok(resok) => Ok(resok.user_id),
        Err(_) => return Err(TokenError::Invalid),
    }
}

#[get("/conversations")]
async fn get_my_conversations(
    auth: BearerAuth,
    pool: web::Data<DbPool>,
    state: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    let token = auth.token();
    info!("Bearer token was: {}", &token);
    let mut jwks = state.jwks.lock().unwrap(); //map_err(error::ErrorInternalServerError)?;
    let user_id = match validate_bearer_identity_token(&mut jwks, &token).await {
        Err(_err) => return Ok(HttpResponse::Unauthorized().body("Token authorization failed")),
        Ok(uid) => uid,
    };
    // Don't block server thread, db stuff is synchronous
    let conversations = web::block(move || {
        let mut conn = pool.get()?;
        find_conversations_by_user(&mut conn, &user_id, /*deleted=*/ false)
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

#[post("/conversation/")]
async fn post_conversation(
    auth: BearerAuth,
    pool: web::Data<DbPool>,
    form: web::Json<NewConversation>,
) -> actix_web::Result<impl Responder> {
    let userid = match validate_bearer_access_token(auth.token()).await {
        Ok(resok) => resok,
        Err(_) => return Ok(HttpResponse::Unauthorized().body("Token authorization failed")),
    };
    let convo_id = web::block(move || -> Result<String, LocalError> {
        let json_contents = serde_json::to_string(&form.contents)?;
        let meta_data = ConversationMetadata {
            title: form.title.clone(),
            model: form.model.clone(),
            creationdate: chrono::Utc::now().into(),
            length: form.contents.dialog.len(),
        };
        let json_metadata = serde_json::to_string(&meta_data)?;
        let mut conn = pool.get()?;
        let new_uuid = uuid::Uuid::new_v4().simple().to_string();
        let nc = Conversation {
            id: new_uuid.clone(),
            contents: json_contents,
            metadata: json_metadata,
            public: form.public,
            research: form.research,
            user_id: userid,
            deleted: false,
        };
        use self::schema::conversations::dsl::*;
        diesel::insert_into(conversations)
            .values(nc)
            .execute(&mut conn)
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
    state: web::Data<AppState>,
    id: web::Path<(String,)>,
) -> actix_web::Result<impl Responder> {
    let uid = id.into_inner().0;
    let token = auth.token();
    let mut jwks = state.jwks.lock().unwrap();
    info!("Bearer token was: {}", &token);
    let userid = match validate_bearer_identity_token(&mut jwks, &token).await {
        Err(_err) => return Ok(HttpResponse::Unauthorized().body("Token authorization failed")),
        Ok(uid) => uid,
    };
    web::block(move || -> Result<(), LocalError> {
        let mut conn = pool.get()?;
        let convo = find_conversation_by_id(&mut conn, &uid, /*deleted=*/ false)?;
        match convo {
            Some(conv) => {
                use self::schema::conversations::dsl::*;
                if conv.user_id != userid {
                    info!("Converation to delete owner does not match requestor");
                    Err(LocalError::AuthorizationProblem)
                } else {
                    diesel::update(conversations.filter(id.eq(uid)))
                        .set(deleted.eq(true))
                        .execute(&mut conn)
                        .expect("Error deleting conversation");
                    Ok(())
                }
            }
            _ => {
                info!("Converation to delete not found");
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
    // Set info log level by default unless you set things manually from .env file
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // Initialize database pool outside server and copy it in
    let pool = initialize_db_pool();
    let mut conn = pool.get().expect("db pool could not produce a connection");
    // Check for pending migrations
    info!("Checking for pending database migrations (stored internally to binary)");
    let cnt = conn
        .pending_migrations(MIGRATIONS)
        .expect("could not get list of migrations")
        .len();
    if cnt > 0 {
        info!("Applying {} pending migrations", cnt);
        conn.run_pending_migrations(MIGRATIONS)
            .expect("could not run pending migrations");
    }

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(AppState {
                jwks: std::sync::Mutex::new(JsonWebKeysSet {
                    keys: std::collections::HashMap::new(),
                    exp: 0,
                }),
            }))
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
