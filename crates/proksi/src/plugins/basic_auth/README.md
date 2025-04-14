# BasicAuth Plugin for Proksi

This plugin provides HTTP Basic Authentication for Proksi, enabling users to verify the identity of API consumers through username/password credentials.

## Overview

The BasicAuth plugin has been migrated from the old `MiddlewarePlugin` interface to the new `Plugin` interface. This migration maintains all previous functionality while adding new features:

- Configurable `BasicAuthConfig` with sensible defaults
- Standard Basic Authentication process following RFC 7617
- Proper plugin lifecycle methods (creation, start, stop)
- Strong type safety through the `Plugin` trait implementation

## Usage

### Creating an instance with default settings

```rust
use crates::proksi::plugins::basic_auth::BasicAuthPlugin;
use crates::proksi::plugins::Plugin;

let plugin = BasicAuthPlugin::new(BasicAuthConfig::default());
```

### Creating an instance with custom configuration

```rust
use crates::proksi::plugins::basic_auth::{BasicAuthPlugin, BasicAuthConfig};
use crates::proksi::plugins::Plugin;

let config = BasicAuthConfig {
    username: "admin".to_string(),
    password: "secure_password".to_string(),
    realm: "Proksi API".to_string(),
    enabled: true,
};

let plugin = BasicAuthPlugin::new(config);
```

### Using credentials file

```rust
use crates::proksi::plugins::basic_auth::{BasicAuthPlugin, BasicAuthConfig};
use crates::proksi::plugins::Plugin;
use std::path::PathBuf;

let config = BasicAuthConfig {
    credentials_file: Some(PathBuf::from("/path/to/credentials.json")),
    realm: "Proksi API".to_string(),
    enabled: true,
    ..BasicAuthConfig::default()
};

let plugin = BasicAuthPlugin::new(config);
```

## Configuration Options

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `username` | String | The username for authentication | Empty string |
| `password` | String | The password for authentication | Empty string |
| `realm` | String | The authentication realm name | "Proksi Service" |
| `enabled` | bool | Whether the plugin is enabled | true |
| `credentials_file` | Option<PathBuf> | Path to a JSON file containing multiple credentials | None |

### Credentials File Format

```json
{
  "credentials": [
    {
      "username": "user1",
      "password": "password1"
    },
    {
      "username": "user2",
      "password": "password2"
    }
  ]
}
```

## Security Considerations

- **Always use HTTPS**: Basic Authentication sends credentials in base64 encoding, which is easily decoded if intercepted. Always use this plugin with TLS/HTTPS enabled.
- **Password Storage**: Do not store plaintext passwords in production. Consider using environment variables or a secure vault.
- **Rate Limiting**: To prevent brute force attacks, use this plugin alongside a rate-limiting mechanism.

## Future Improvements

- Multi-user authentication with role-based access control
- Dynamic credential updates without restarting
- Support for hashed password storage
- Integration with external authentication providers 