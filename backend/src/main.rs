extern crate diesel;

#[macro_use]
extern crate lazy_static;

mod schema;

use actix_session::{
    config::PersistentSession, storage::CookieSessionStore, Session, SessionMiddleware,
};
use actix_web::{
    cookie::time::Duration, cookie::Key, delete, error, get, middleware, patch, post, web, App,
    HttpResponse, HttpServer, Responder,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::offset::Utc;
use chrono::DateTime;
use diesel::{prelude::*, r2d2};
use handlebars::{handlebars_helper, Handlebars};
use log::info;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::string::String;
use std::vec::Vec;

// Types related to Postgres connection to database
type DbConnection = diesel::pg::PgConnection;
type DbConnectionManager = diesel::r2d2::ConnectionManager<DbConnection>;
type DbPool = diesel::r2d2::Pool<DbConnectionManager>;
type DbError = Box<dyn std::error::Error + Send + Sync>;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use schema::conversations;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

// True constants
const EXPIRATION_SECONDS: u64 = 60 * 60 * 5;

// Templates
// Can't load during initialization.
// Lazy static means they are actually loaded when referenced.
lazy_static! {
    static ref INDEX_HBS: String =
        std::fs::read_to_string("./site/index.hbs").expect("Read INDEX_HBS");
    static ref INDEX_CSS: String =
        std::fs::read_to_string("./site/index.css").expect("Read INDEX_CSS");
    static ref CHATGPT_PNG: Vec<u8> =
        std::fs::read("./site/chatgpt.png").expect("Read CHATGPT_PNG");
    static ref MAIN_JS: String = std::fs::read_to_string("./site/main.js").expect("Read MAIN_JS");
    static ref MAX_FREE_USER_COUNT: i64 = std::env::var("MAX_FREE_USER_COUNT")
        .expect("MAX_FREE_USER_COUNT should be set")
        .parse()
        .expect("Cound not parse MAX_FREE_USER_COUNT");
    static ref MARKDOWN_OPTIONS: pulldown_cmark::Options = {
        let mut options = pulldown_cmark::Options::empty();
        options.insert(pulldown_cmark::Options::ENABLE_TABLES);
        options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
        options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
        options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
        options
    };
}

// Google keys
#[derive(Debug, Deserialize)]
struct JsonWebKey {
    r#_use: String,
    kid: String,
    _alg: String,
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
// Handle markdown
handlebars_helper!(markdown: |*args| {
    let txt = match args[0] {
        serde_json::Value::String(s) => s,
        _ => "Invalid JSON value for markdown string",
    };
    let parser = pulldown_cmark::Parser::new_ext(&txt, *MARKDOWN_OPTIONS);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    html_output
});

// Model for conversations in the database with all fields
#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = conversations)]
pub struct Conversation {
    pub id: String,
    pub hmac: String,
    pub contents: String, // JSON for ConversationContents
    pub metadata: String, // JSON for ConversationMetadata
    pub public: bool,
    pub research: bool,
    pub deleted: bool,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct Utterance {
    pub who: String, // either "gpt" or "human"
    pub what: String,
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct ConversationContents {
    pub avatar: String, // data URL of avatar, (may be anonymized)
    pub dialog: Vec<Utterance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub title: String,
    pub openaiid: String,
    pub model: String,
    pub creationdate: std::time::SystemTime,
    pub length: usize,
}

impl std::hash::Hash for ConversationMetadata {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.openaiid.hash(state);
        self.model.hash(state);
        // Ignore creationdate for hash
        self.length.hash(state);
    }
}

// Information that is required when making a new conversation
#[derive(Serialize, Deserialize)]
pub struct NewConversation {
    pub openaiid: String,
    pub title: String,
    pub contents: ConversationContents,
    pub model: String,
    pub public: bool,
    pub research: bool,
    pub paiduser: bool,
}

// Information that is required when patching an existing conversation
#[derive(Serialize, Deserialize)]
pub struct PatchConversation {
    pub id: String,
    pub contents: ConversationContents,
    pub metadata: ConversationMetadata,
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
    pub deleted: bool,
    pub hmac: String,
}

// Information returned from GET for list of conversations
#[derive(Debug, Serialize, Deserialize)]
pub struct ShortConversationInfo {
    pub id: String,
    pub metadata: ConversationMetadata,
    pub public: bool,
    pub research: bool,
    pub deleted: bool,
    pub hmac: String,
}

#[derive(Debug)]
enum JWKSError {
    Retrieval,
    DecodingKeyError,
    NotFound,
}

fn get_epoch_time() -> u64 {
    let start = std::time::SystemTime::now();
    start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards from UNIX_EPOCH")
        .as_secs() as u64
}

// Refresh our collection of Google public keys
// Find correct one to use
async fn retrieve_key(
    jwks: &mut JsonWebKeysSet,
    id: &str,
) -> Result<jsonwebtoken::DecodingKey, JWKSError> {
    let now = get_epoch_time();
    info!(
        "Checking for expiration, len={} now={} exp={}",
        jwks.keys.len(),
        now,
        jwks.exp
    );
    let should_retrieve = jwks.keys.len() == 0 || now >= jwks.exp;
    if should_retrieve {
        info!("Refreshing Google public keys");
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
                jwks.exp = now + EXPIRATION_SECONDS;
                info!("New expiration of public keys {}", jwks.exp);
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

// Get conversation count so we can limit free users
// Only counts non-deleted posts
fn get_conversation_count(conn: &mut DbConnection, userid: &String) -> Result<i64, DbError> {
    use self::schema::conversations::dsl::*;
    let results: i64 = conversations
        .filter(user_id.eq(userid))
        .filter(deleted.eq(false))
        .count()
        .get_result(conn)
        .expect("Error finding conversations for counting");
    Ok(results)
}

// See if a conversation already exists (by hmac)
// If exists, returns Some<id>, otherwise None
fn conversation_exists(
    conn: &mut DbConnection,
    uid: &String,
    hmac_digest: &String,
) -> Result<Option<String>, DbError> {
    use self::schema::conversations::dsl::*;
    let results = conversations
        .filter(user_id.eq(uid))
        .filter(hmac.eq(hmac_digest))
        .filter(deleted.eq(false))
        .limit(1)
        .load::<Conversation>(conn)
        .expect("Error finding conversation");
    if results.len() == 0 {
        Ok(None)
    } else {
        let result = results[0].id.clone();
        Ok(Some(result))
    }
}

// Look in DB for all conversations of a user
fn find_conversations_by_user(
    conn: &mut DbConnection,
    uid: &String,
) -> Result<Vec<ShortConversationInfo>, DbError> {
    use self::schema::conversations::dsl::*;
    conversations
        .filter(user_id.eq(uid))
        .order_by(id.desc())
        .load::<Conversation>(conn)
        .expect("Error finding conversation")
        .iter()
        .map(|conv| {
            Ok(ShortConversationInfo {
                id: conv.id.clone(),
                metadata: serde_json::from_str(&conv.metadata)?,
                public: conv.public,
                research: conv.research,
                deleted: conv.deleted,
                hmac: conv.hmac.clone(),
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
                deleted: conv.deleted,
                hmac: conv.hmac,
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
            reg.register_helper("markdown", Box::new(markdown));
            let contents: ConversationContents = serde_json::from_str(&conv.contents)?;
            let metadata: ConversationMetadata = serde_json::from_str(&conv.metadata)?;
            let chatgpt_uri: String =
                format!("data:image/png;base64,{}", base64::encode(&*CHATGPT_PNG));
            let timestamp: DateTime<Utc> = metadata.creationdate.into();
            let timestamp_str: String = format!("{}", timestamp.format("%Y/%m/%d %T UTC"));
            let body = reg
                .render_template(
                    &INDEX_HBS,
                    &serde_json::json!({
                        "style": *INDEX_CSS,
                        "main_js": *MAIN_JS,
                        "title": metadata.title,
                        "model": metadata.model,
                        "openaiid": metadata.openaiid,
                        "avatar": contents.avatar,
                        "dialog": contents.dialog,
                        "chatgpt_uri": chatgpt_uri,
                        "timestamp": timestamp_str,
                        "hmac": &conv.hmac,
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
    let since_the_epoch = get_epoch_time();
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

/// Check if user is authenticated
// This endpoint does not perform authentication.
// Respond with 200 if authenticated, 401 if not
#[post("/authenticated")]
async fn authenticated(session: Session) -> actix_web::Result<impl Responder> {
    info!("Checking cookie");
    if let Some(session_user_id) = session.get::<String>("user_id")? {
        info!("user_id is {}", session_user_id);
        return Ok(HttpResponse::Ok().body("Authenticated"));
    }
    info!("Cookie check failed");
    Ok(HttpResponse::Unauthorized().body("Not authenticated"))
}

/// Log out user
#[post("/logout")]
async fn logout(session: Session) -> actix_web::Result<impl Responder> {
    session.purge();
    Ok(HttpResponse::Ok().body("Logged out"))
}

/// Authenticate user
// Client provides Google identity token (from GIS button thingy)
// Sends that token as "authorization: Bearer ..."
// If it works, response will be 200 with "set-cookie: ..."
// The session cookie is a JWT token that represents the authenticated session
#[post("/authenticate")]
async fn authenticate(
    auth: BearerAuth,
    state: web::Data<AppState>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    if let Some(session_user_id) = session.get::<String>("user_id")? {
        info!("user_id is {}", session_user_id);
        return Ok(HttpResponse::Ok().body("Authenticated"));
    }
    info!("Starting authentication");
    let token = auth.token();
    info!("Bearer token was: {}", &token);
    let mut jwks = state.jwks.lock().unwrap();
    let user_id = match validate_bearer_identity_token(&mut jwks, &token).await {
        Err(_err) => return Ok(HttpResponse::Unauthorized().body("Authorization failed")),
        Ok(uid) => uid,
    };
    info!("Setting session to have user_id={}", user_id);
    session.insert("user_id", user_id)?;
    info!("Session inserted");
    Ok(HttpResponse::Ok().body("Authenticated"))
}

#[post("/conversations")]
async fn get_my_conversations(
    pool: web::Data<DbPool>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let user_id = match session.get::<String>("user_id")? {
        Some(session_user_id) => session_user_id,
        None => return Ok(HttpResponse::Unauthorized().body("Authorization failed")),
    };
    // Don't block server thread, db stuff is synchronous
    let conversations = web::block(move || {
        let mut conn = pool.get()?;
        find_conversations_by_user(&mut conn, &user_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(conversations))
}

#[get("/conversation/count")]
async fn get_conversation_count_user(
    pool: web::Data<DbPool>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let user_id = match session.get::<String>("user_id")? {
        Some(session_user_id) => session_user_id,
        None => return Ok(HttpResponse::Unauthorized().body("Authorization failed")),
    };
    // Don't block server thread, db stuff is synchronous
    let count = web::block(move || {
        let mut conn = pool.get()?;
        get_conversation_count(&mut conn, &user_id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(count))
}

#[derive(Debug)]
enum LocalError {
    DbConnectionProblem,
    SerializationFailed,
    DbError,
    AuthorizationProblem,
    NotFound,
    MaxCount,
}

impl std::fmt::Display for LocalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            LocalError::DbConnectionProblem => write!(f, "problem with r2d2 db connection"),
            LocalError::SerializationFailed => write!(f, "json serialization of contents failed"),
            LocalError::DbError => write!(f, "problem with db connection"),
            LocalError::AuthorizationProblem => write!(f, "authorization problem"),
            LocalError::NotFound => write!(f, "conversation not found"),
            LocalError::MaxCount => write!(f, "Maximum free share count reached"),
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

fn compute_digest(contents: &ConversationContents, metadata: &ConversationMetadata, userid: &String) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    contents.hash(&mut h);
    metadata.hash(&mut h);
    userid.hash(&mut h);
    format!("{:#x}", h.finish())
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
            openaiid: form.openaiid.clone(),
            model: form.model.clone(),
            creationdate: chrono::Utc::now().into(),
            length: form.contents.dialog.len(),
        };
        let json_metadata = serde_json::to_string(&meta_data)?;
        let mut conn = pool.get()?;
        let digest = compute_digest(&form.contents, &meta_data, &userid);
        if let Some(uuid) = conversation_exists(&mut conn, &userid, &digest)? {
            return Ok(uuid);
        }
        // Check if the user can post more
        let count = get_conversation_count(&mut conn, &userid)?;
        let allowed_post = form.paiduser || count < *MAX_FREE_USER_COUNT;
        if !allowed_post {
            return Err(LocalError::MaxCount);
        }
        let new_uuid = uuid::Uuid::new_v4().simple().to_string();
        let nc = Conversation {
            id: new_uuid.clone(),
            hmac: digest,
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

#[patch("/conversation/{id}")]
async fn patch_conversation(
    pool: web::Data<DbPool>,
    form: web::Json<PatchConversation>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let userid = match session.get::<String>("user_id")? {
        Some(session_user_id) => session_user_id,
        None => return Ok(HttpResponse::Unauthorized().body("Authorization failed")),
    };
    web::block(move || -> Result<(), LocalError> {
        let mut conn = pool.get()?;
        let conversation = find_conversation_by_id(&mut conn, &form.id, /*deleted=*/ false)?;
        match conversation {
            Some(conv) => {
                if conv.user_id != userid {
                    info!("Conversation to patch owner does not match requestor");
                    return Err(LocalError::AuthorizationProblem)
                }
                let contents_json = serde_json::to_string(&form.contents)?;
                let metadata_json = serde_json::to_string(&form.metadata)?;
                let digest = compute_digest(&form.contents, &form.metadata, &userid);
                use self::schema::conversations::dsl::*;
                diesel::update(conversations.filter(id.eq(&form.id)))
                    .set((
                        contents.eq(contents_json),
                        metadata.eq(metadata_json),
                        public.eq(form.public),
                        research.eq(form.research),
                        hmac.eq(digest),
                    ))
                    .execute(&mut conn)
                    .expect("Error patching conversation");
                Ok(())
            },
            None => {
                info!("Conversation to patch not found");
                return Err(LocalError::NotFound)
            }
        }
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().into())
}

#[post("/conversation/undelete/{id}")]
async fn undelete_conversation(
    pool: web::Data<DbPool>,
    postid_path: web::Path<(String,)>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let postid = postid_path.0.clone();
    let uid = match session.get::<String>("user_id")? {
        Some(session_user_id) => session_user_id,
        None => return Ok(HttpResponse::Unauthorized().body("Authorization failed")),
    };
    web::block(move || -> Result<(), LocalError> {
        let mut conn = pool.get()?;
        let conversation = find_conversation_by_id(&mut conn, &postid, /*deleted=*/ true)?;
        match conversation {
            Some(conv) => {
                use self::schema::conversations::dsl::*;
                if conv.user_id != uid {
                    info!("Conversation to undelete owner does not match requestor");
                    return Err(LocalError::AuthorizationProblem)
                }
                diesel::update(conversations.filter(id.eq(postid)))
                    .set(deleted.eq(false))
                    .execute(&mut conn)
                    .expect("Error undeleting conversation");
                Ok(())
            }
            _ => {
                info!("Conversation to undelete not found");
                return Err(LocalError::AuthorizationProblem)
            }
        }
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().into())
}

#[delete("/conversation/{id}")]
async fn delete_conversation(
    pool: web::Data<DbPool>,
    postid_path: web::Path<(String,)>,
    session: Session,
) -> actix_web::Result<impl Responder> {
    let postid = postid_path.0.clone();
    let uid = match session.get::<String>("user_id")? {
        Some(session_user_id) => session_user_id,
        None => return Ok(HttpResponse::Unauthorized().body("Authorization failed")),
    };
    web::block(move || -> Result<(), LocalError> {
        let mut conn = pool.get()?;
        let convo = find_conversation_by_id(&mut conn, &postid, /*deleted=*/ false)?;
        match convo {
            Some(conv) => {
                use self::schema::conversations::dsl::*;
                if conv.user_id != uid {
                    info!("Conversation to delete owner does not match requestor");
                    Err(LocalError::AuthorizationProblem)
                } else {
                    diesel::update(conversations.filter(id.eq(postid)))
                        .set(deleted.eq(true))
                        .execute(&mut conn)
                        .expect("Error deleting conversation");
                    Ok(())
                }
            }
            _ => {
                info!("Conversation to delete not found");
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
    // Setup cookie secret key
    info!("Generating cookie secret key");
    let secret = std::env::var("SECRET").expect("SECRET should be set");
    let secret_key = Key::derive_from(secret.as_bytes());

    let state = web::Data::new(AppState {
        jwks: std::sync::Mutex::new(JsonWebKeysSet {
            keys: std::collections::HashMap::new(),
            exp: 0,
        }),
    });

    const COOKIE_DURATION_SECS: i64 = 60 * 60 * 24 * 30; // 30 days

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .session_lifecycle(
                        PersistentSession::default()
                            .session_ttl(Duration::seconds(COOKIE_DURATION_SECS)),
                    )
                    .build(),
            )
            .app_data(web::Data::new(pool.clone()))
            .app_data(state.clone())
            .service(get_conversation_json)
            .service(get_conversation_html)
            .service(post_conversation)
            .service(delete_conversation)
            .service(undelete_conversation)
            .service(get_my_conversations)
            .service(authenticate)
            .service(authenticated)
            .service(logout)
            .service(get_conversation_count_user)
            .service(patch_conversation)
    })
    .bind("0.0.0.0:9090")?
    .run()
    .await
}
