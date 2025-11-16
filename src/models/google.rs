use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GoogleIdTokenClaims {
    #[allow(dead_code)]
    pub iss: String,
    #[allow(dead_code)]
    pub aud: String,
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
    #[allow(dead_code)]
    pub exp: i64,
    #[allow(dead_code)]
    pub iat: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GoogleJwks {
    pub keys: Vec<GoogleJwk>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GoogleJwk {
    pub kid: String,      // Key ID
    pub n: String,        // RSA modulus (base64url)
    pub e: String,        // RSA exponent (base64url)
    // Other fields like kty, use, alg are ignored by serde
}
