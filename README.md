# LoW Access Backend

Rust/Axum API server that validates Google OAuth tokens and manages user authorization for LoW Net access.

## Quick Start

```bash
# 1. Copy example config
cp config.toml.example config.toml

# 2. Edit config.toml with your settings
# REQUIRED:
#   - Set your Google OAuth Client ID
#   - Create a Tailscale OAuth client: https://tailscale.com/kb/1215/oauth-clients
#   - Set the path to your Tailscale OAuth secret file

# 3. Run the server
cargo run
```

## Configuration

The backend supports three configuration methods with the following precedence:

**CLI args > Environment variables > Config file > Built-in defaults**

### Config File (Recommended)

Copy `config.toml.example` to `config.toml` and customize:

```toml
[server]
bind_address = "127.0.0.1:3000"

[google]
client_id = "YOUR_GOOGLE_CLIENT_ID"

[tailscale]
oauth_secret_path = "/run/secrets/tailscale_oauth_secret"
api_url = "https://api.tailscale.com/api/v2"
auth_key_tags = ["tag:low-access"]

[database]
path = "sso.db"
```

### Environment Variables

Prefix with `LOW_ACCESS_` and use `__` (double underscore) for nested keys:

```bash
export LOW_ACCESS_SERVER__BIND_ADDRESS=0.0.0.0:8080
export LOW_ACCESS_GOOGLE__CLIENT_ID=your-client-id
export LOW_ACCESS_TAILSCALE__OAUTH_SECRET_PATH=/path/to/secret
export LOW_ACCESS_TAILSCALE__AUTH_KEY_TAGS='["tag:low-access"]'
export LOW_ACCESS_DATABASE__PATH=./sso.db
```

Then run `low-access-api` to start the server with these settings.

### CLI Arguments

```bash
low-access-api --help

# Examples:
low-access-api --bind-address 0.0.0.0:8080
low-access-api --config /etc/sso/config.toml
low-access-api --database-path /var/lib/sso/db.sqlite
low-access-api --tailscale-auth-key-tag tag:low-access
low-access-api --tailscale-auth-key-tag tag:one --tailscale-auth-key-tag tag:two
```

## API Endpoints

- `GET /` - Health check
- `GET /auth/validate?id_token=...` - Validate Google ID token
- `POST /auth/generate-token` - Generate Tailscale token (approved users only)

## Development

```bash
# Build
cargo build

# Run with debug logs
RUST_LOG=debug cargo run

# Run with CLI arguments (development)
cargo run -- --help
cargo run -- --bind-address 127.0.0.1:8080
cargo run -- --log-level debug

# Run tests (when implemented)
cargo test

# Clean build artifacts
cargo clean
```

## Docker Deployment

```dockerfile
# Example Dockerfile snippet
FROM rust:1.91 as builder
COPY . /app
WORKDIR /app
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/low-access-api /usr/local/bin/
COPY config.toml /etc/sso/config.toml

# Mount secret at runtime
# docker run -v /path/to/secret:/run/secrets/tailscale_oauth_secret ...
CMD ["low-access-api", "--config", "/etc/sso/config.toml"]
```

## Database

SQLite database at path specified in config (default: `sso.db` in working directory).

**Tables:**
- `users` - User records with approval status (pending/approved/denied)
- `user_permissions` - User permission grants

**Migrations:** Run automatically on startup.

**Reset database:**
```bash
rm sso.db sso.db-wal sso.db-shm
```
