use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegisterUser {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoginUser {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoginResponse {
    pub jwt: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeleteUser {
    pub password: String,
}

/// Claims to store inside the JWT Token
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// User ID.
    pub sub: String,
    pub exp: usize,
}
