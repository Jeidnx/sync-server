use std::time::{Duration, SystemTime, UNIX_EPOCH};

use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, delete, error, post, web};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use diesel::result::DatabaseErrorKind;
use hmac::{Hmac, KeyInit, Mac as _};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sha2::Sha256;
use uuid::Uuid;

use crate::database::user::{
    delete_existing_user, find_user_by_id, find_user_by_name_hash, insert_new_user,
};
use crate::dto::{JwtClaims, LoginResponse};
use crate::handlers::ScopedHandler;
use crate::models::User;
use crate::util::bytes_to_hex_string;
use crate::{WebData, dto, models};

// TODO: make configurable
const SECRET_KEY: &str = "secret";

const AUTH_HEADER_KEY: &str = "Authorization";

pub struct UserHandler {}
impl ScopedHandler for UserHandler {
    fn get_service() -> actix_web::Scope {
        web::scope("/user")
            .service(register_user)
            .service(login_user)
            // services that require auth start here
            .service(
                web::scope("")
                    .wrap(actix_web::middleware::from_fn(auth_middleware))
                    .service(delete_user),
            )
    }
}

#[post("/register")]
async fn register_user(
    pool: WebData,
    form: web::Json<dto::RegisterUser>,
) -> actix_web::Result<impl Responder> {
    let mut conn = pool
        .get()
        .await
        .expect("Couldn't get db connection from the pool");

    let user = models::User {
        id: Uuid::now_v7().to_string(),
        name_hash: hash_username(&form.name),
        password_hash: hash_password(&form.password),
    };

    let user = insert_new_user(&mut conn, &user)
        .await
        .map_err(|err| match err {
            diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                error::ErrorConflict("username already taken")
            }
            _ => error::ErrorInternalServerError(err),
        })?;

    match generate_jwt(&user) {
        Ok(jwt) => {
            let resp = LoginResponse { jwt };
            Ok(HttpResponse::Created().json(resp))
        }
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[post("/login")]
async fn login_user(
    pool: WebData,
    form: web::Json<dto::LoginUser>,
) -> actix_web::Result<impl Responder> {
    let mut conn = pool
        .get()
        .await
        .expect("Couldn't get db connection from the pool");

    let name = hash_username(&form.name);
    let Some(user) = find_user_by_name_hash(&mut conn, &name)
        .await
        .ok()
        .flatten()
    else {
        return Err(error::ErrorForbidden("invalid username or password"));
    };

    if !verify_password(&form.password, &user.password_hash) {
        return Err(error::ErrorForbidden("invalid username or password"));
    }

    match generate_jwt(&user) {
        Ok(jwt) => {
            let resp = LoginResponse { jwt };
            Ok(HttpResponse::Ok().json(resp))
        }
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[delete("/delete")]
async fn delete_user(
    req: HttpRequest,
    pool: WebData,
    form: web::Json<dto::DeleteUser>,
) -> actix_web::Result<impl Responder> {
    let mut conn = pool
        .get()
        .await
        .expect("Couldn't get db connection from the pool");

    // make sure the refcell doesn't escape await, see https://rust-lang.github.io/rust-clippy/rust-1.95.0/index.html#await_holding_refcell_ref
    let user_id;
    {
        let extensions = req.extensions();
        let user = extensions.get::<User>().unwrap();

        if !verify_password(&form.password, &user.password_hash) {
            return Err(error::ErrorForbidden("invalid username or password"));
        }

        user_id = user.id.clone();
    }

    match delete_existing_user(&mut conn, &user_id).await {
        Ok(_) => Ok(HttpResponse::Ok()),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

/// Middleware that ensures that the user is authenticated.
async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let auth_header = req
        .headers()
        .get(AUTH_HEADER_KEY)
        .and_then(|header| header.to_str().ok())
        .map(|value| value.to_string());
    let auth_cookie = req
        .cookie(AUTH_HEADER_KEY)
        .map(|cookie| cookie.value().to_string());

    let Some(jwt) = auth_cookie.or(auth_header) else {
        return Err(error::ErrorUnauthorized("missing authentication token"));
    };
    let Ok(user_id) = verify_jwt(&jwt) else {
        return Err(error::ErrorUnauthorized("invalid authentication token"));
    };
    let pool: WebData = req.app_data().cloned().unwrap();

    let mut conn = pool
        .get()
        .await
        .expect("Couldn't get db connection from the pool");

    let Some(user) = find_user_by_id(&mut conn, &user_id).await.ok().flatten() else {
        return Err(error::ErrorBadRequest("user does not exist"));
    };

    // append user to request extensions so that it can be accessed with
    // `req.extensions().get::<User>()` by handlers
    req.extensions_mut().insert(user);

    // pre-processing
    next.call(req).await
    // post-processing
}

// TODO: move into util
fn generate_jwt(user: &User) -> jsonwebtoken::errors::Result<String> {
    let key = EncodingKey::from_secret(SECRET_KEY.as_bytes());
    // tokens are valid for one year, should be enough in most cases
    let expiration_date = SystemTime::now()
        .checked_add(Duration::from_hours(365 * 24))
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let claims = JwtClaims {
        sub: user.id.clone(),
        exp: expiration_date as usize,
    };
    encode(&Header::default(), &claims, &key)
}

/// Returns the User ID on success.
fn verify_jwt(encoded_jwt: &str) -> jsonwebtoken::errors::Result<String> {
    let key = DecodingKey::from_secret(SECRET_KEY.as_bytes());
    let claims: JwtClaims = decode(encoded_jwt.as_bytes(), &key, &Validation::default())?.claims;
    Ok(claims.sub)
}

fn argon2_instance<'a>() -> Argon2<'a> {
    Argon2::default()
}

fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    argon2_instance()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(password_hash) = PasswordHash::new(password_hash) else {
        return false;
    };
    argon2_instance()
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok()
}

fn hash_username(username: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(username.as_bytes()).unwrap();
    mac.update(SECRET_KEY.as_bytes());

    let result = &mac.finalize().into_bytes();
    bytes_to_hex_string(result)
}
