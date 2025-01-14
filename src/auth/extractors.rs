use crate::auth::passcode;
use crate::auth::responses::{PasscodeExtractionError, PasscodeExtractionReason};
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Json;
use std::convert::Infallible;

pub struct User {
    pub public_id: String,
    pub private_id: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<PasscodeExtractionError>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(passcode) = parts.headers.get("Passcode") {
            match passcode::decode(passcode.to_str().unwrap()) {
                Ok(jwt_payload) => Ok(User {
                    public_id: jwt_payload.public_id,
                    private_id: jwt_payload.private_id,
                }),
                Err(_) => Err((
                    StatusCode::UNAUTHORIZED,
                    Json(PasscodeExtractionError {
                        error: true,
                        reason: PasscodeExtractionReason::InvalidPasscode,
                    }),
                )),
            }
        } else {
            Err((
                StatusCode::UNAUTHORIZED,
                Json(PasscodeExtractionError {
                    error: true,
                    reason: PasscodeExtractionReason::NoPasscodeHeaderProvided,
                }),
            ))
        }
    }
}

pub struct MaybeUser {
    #[allow(unused)]
    pub public_id: Option<String>,
    pub private_id: Option<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for MaybeUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Infallible);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(passcode) = parts.headers.get("Passcode") {
            match passcode::decode(passcode.to_str().unwrap()) {
                Ok(jwt_payload) => Ok(MaybeUser {
                    public_id: Some(jwt_payload.public_id),
                    private_id: Some(jwt_payload.private_id),
                }),
                Err(_) => Ok(MaybeUser {
                    public_id: None,
                    private_id: None,
                }),
            }
        } else {
            Ok(MaybeUser {
                public_id: None,
                private_id: None,
            })
        }
    }
}
