use actix_web::{
    dev::Payload, error::ErrorUnauthorized, web, Error as ActixWebError, FromRequest, HttpRequest,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

use crate::PlayerId;

pub const COOKIE_NAME: &'static str = "jwt_token";

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: usize,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct AuthenticationToken {
    pub id: usize,
}

impl Into<PlayerId> for AuthenticationToken {
    fn into(self) -> PlayerId {
        self.id
    }
}

impl FromRequest for AuthenticationToken {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();

        let authentication_token = match req
            .cookies()
            .expect("Panicked on getting cookies from request")
            .iter()
            .filter(|c| c.name() == COOKIE_NAME)
            .next()
        {
            Some(cookie) => cookie.value().to_string(),
            None => return ready(Err(ErrorUnauthorized("No authentication token sent!"))),
        };

        if authentication_token.is_empty() {
            return ready(Err(ErrorUnauthorized(
                "Authentication token has foreign chars!",
            )));
        }

        let secret = &req
            .app_data::<web::Data<String>>()
            .expect("no secret in app_data");

        let token_result = decode::<Claims>(
            &authentication_token,
            &DecodingKey::from_secret(secret.as_ref().as_bytes()),
            &Validation::new(Algorithm::HS256),
        );

        match token_result {
            Ok(token) => ready(Ok(AuthenticationToken {
                id: token.claims.id,
            })),
            Err(_e) => ready(Err(ErrorUnauthorized("Invalid authentication token sent!"))),
        }
    }
}
