use std::sync::Arc;

use axum::{
    body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use bcrypt::verify;
use tracing::{debug, error};

use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::models::{ApiResponse, AppState, Data, TokenClaims, User, UserSchema, UserRegister};

pub fn user_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", routing::post(login))
        .route("/logout", routing::get(logout))
        .route("/register", routing::post(register))
}

type Result = std::result::Result<ApiResponse, ApiResponse>;

pub async fn login(State(app_state): State<Arc<AppState>>, Json(user_schema): Json<UserSchema>) -> Result {
    //) -> Result<Json<serde_json::Value>,(StatusCode, Json<serde_json::Value>)>{
    tracing::info!("init login");
    tracing::info!("User schema: {:?}", user_schema);
    let user = User::get_by_email(&app_state.pool, &user_schema.email)
        .await
        .map_err(|e| {
            let message = &format!("Error: {}", e);
            ApiResponse::new(StatusCode::FORBIDDEN, message, Data::None)
        })?;
    if !user.active || !verify(&user_schema.password, &user.hashed_password).unwrap() {
        let message = "Invalid name or password. Please <a href='/login'>log in</a>";
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, message, Data::None));
    }

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::minutes(60)).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: user.email.to_string(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(app_state.secret.as_bytes()),
    )
    .map_err(|e| {
        let message = format!("Encoding JWT error: {}", e);
        ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, &message, Data::None)
    })
    .map(|token| {
        let value = serde_json::json!({"token": token});
        ApiResponse::new(StatusCode::OK, "Ok", Data::One(value))
    })
}

pub async fn register(
    State(app_state): State<Arc<AppState>>,
    Json(user_data): Json<UserRegister>,
) -> impl IntoResponse {
    debug!("User data: {:?}", user_data);
    match User::create(&app_state.pool, &user_data.username, &user_data.email, &user_data.password).await {
        Ok(user) => {
            debug!("User created: {:?}", user);
            ApiResponse::new(StatusCode::CREATED, "User created", Data::One(serde_json::to_value(user).unwrap()))
        },
        Err(e) => {
            error!("Error creating user: {:?}", e);
            ApiResponse::new(StatusCode::BAD_REQUEST, "Error creating user", Data::None)
        }
    }
}

pub async fn logout() -> impl IntoResponse {
    debug!("Logout");
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(cookie::time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .build();

    tracing::info!("The cookie: {}", cookie.to_string());

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/")
        .header(header::SET_COOKIE, cookie.to_string())
        .body(body::Body::empty())
        .unwrap()
}

