use crate::util::jwt::generate::generate;
use crate::{
    models::{
        appstate::Appstate,
        user::*,
    },
    util::validation,
};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::Error;
use std::sync::Arc;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;

#[derive(Serialize, Deserialize)]
pub struct Body {
    username: String,
    email: String,
    password: String,
}

#[axum_macros::debug_handler]
pub async fn new(
    State(appstate): State<Arc<Appstate>>,
    jar: CookieJar,
    Json(body): Json<Body>,
) -> Result<(StatusCode, CookieJar), (StatusCode, String)> {

    // validate username & password
    match validation::username(&body.username) {
        (true, _) => {},
        (false, e) => {
                return Err((StatusCode::BAD_REQUEST, format!("Username is not valid: {}", e)))
        }
    }

    match validation::password(&body.password) {
        (true, _) => {}
        (false, e) => {
            return Err((StatusCode::BAD_REQUEST, format!("Password is not valid: {}", e)))
        }
    }

    // hash password
    // hashing the password should be done after checking for unique username
    let salt = SaltString::generate(&mut OsRng);
    let argon = Argon2::default();
    let hashed_password = match argon.hash_password(body.password.as_ref(), &salt) {
        Ok(o) => o.to_string(),
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string()))
    };

    // construct user model
    let user = User::new(
        body.username,
        hashed_password,
        body.email,
        Permission::USER /* Hard coded user permission, admin rights yet to be implemented */
    );

    // TODO! send user email to validate

    // generate jwt for user
    let token = generate(&appstate.jwt_secret, &user);
    let Ok(token) = token else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate jwt".to_string()))
    };

    // write user to db
    let conn = &appstate.db_pool;
    let query =
        r"INSERT INTO users (uuid, username, email, password, permission, tokenid) VALUES ($1, $2, $3, $4, $5, $6)";

    let query_result = sqlx::query(query)
        .bind(&user.uuid.to_string())
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password)
        .bind(&user.permission)
        .bind(&user.tokenid.to_string())
        .execute(conn.as_ref()).await;

    if let Err(e) = query_result {
        return match e {
            Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    Err((StatusCode::BAD_REQUEST, "Username is already taken".to_string()))
                } else {
                    Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write to database".to_string()))
                }
            }
            _ => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to write to database".to_string()))
        }
    }

    // set cookie
    let jar = jar.add(Cookie::new("token", token));

    Ok((StatusCode::CREATED, jar))
}