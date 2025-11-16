// Domain models organized by module

pub mod user;
pub mod google;
pub mod tailscale;
pub mod handlers;

// Re-export commonly used types at the models root
pub use user::User;
pub use google::{GoogleIdTokenClaims, GoogleJwks, GoogleJwk};
pub use tailscale::{
    CreateAuthKeyRequest, CreateAuthKeyResponse, OAuthTokenResponse, CachedToken,
    Capabilities, DeviceCapabilities, DeviceCreate,
};
pub use handlers::{
    GenerateTokenRequest, ValidateTokenResponse, GenerateTokenResponse,
};
