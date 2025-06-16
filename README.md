# stalwart_mta_hook_types

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

Rust type definitions for Stalwart Mail Transfer Agent (MTA) hooks. This crate provides strongly-typed request and response structures for implementing external hooks in Stalwart Mail Server.

See also: https://stalw.art/docs/api/mta-hooks/overview

## Overview

This crate defines the data structures used for communication between Stalwart Mail Server and external hook implementations. It provides:

- **Request types**: Structures representing incoming hook requests from Stalwart MTA
- **Response types**: Structures for responding to hook requests with actions and modifications
- **Modification types**: Enums representing possible email modifications (headers, recipients, content)

## Features

- Full serialization/deserialization support with `serde`
- Type-safe handling of SMTP protocol stages
- Support for email envelope and message modifications
- Flexible parameter handling with automatic type conversion
- Comprehensive SMTP response configuration

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
stalwart_mta_hook_types = "0.1"
```

### Basic Example

```rust
use stalwart_mta_hook_types::{Request, Response, Action, Modification};

// Parse an incoming hook request
let request: Request = serde_json::from_str(json_data)?;

// Create a response that accepts the email with modifications
let response = Response {
    action: Action::Accept,
    response: None,
    modifications: vec![
        Modification::add_header("X-Processed-By".to_string(), "My Hook".to_string()),
        Modification::add_recipient("backup@example.com".to_string()),
    ],
};

// Serialize response back to JSON
let response_json = serde_json::to_string(&response)?;
```

### Rejecting Email with Custom Response

```rust
use stalwart_mta_hook_types::{Response, Action, SmtpResponse};

let response = Response {
    action: Action::Reject,
    response: Some(SmtpResponse {
        status: Some(550),
        enhanced_status: Some("5.7.1".to_string()),
        message: Some("Message rejected by policy".to_string()),
        disconnect: false,
    }),
    modifications: vec![],
};
```

## Core Types

### Request

Represents an incoming hook request from Stalwart MTA:

```rust
pub struct Request {
    pub context: Context,
    pub envelope: Option<Envelope>,
    pub message: Option<Message>,
}
```

The `context` contains information about the SMTP session, client connection, and server details.

### Response

Represents the hook's response back to Stalwart MTA:

```rust
pub struct Response {
    pub action: Action,
    pub response: Option<SmtpResponse>,
    pub modifications: Vec<Modification>,
}
```

For the functionality available see the stalwart documentation at: https://stalw.art/docs/api/mta-hooks/overview

## License

This project is licensed under the AGPL-3.0 license.

## Contributing

This crate is part of a larger SoLaWi (Solidarische Landwirtschaft) management system. 

## Credits

Based on Stalwart Mail Server hook types:
- Original work: Copyright 2020 Stalwart Labs LLC
- Modifications: Copyright 2025 Franz Dietrich

Both licensed under AGPL-3.0-only.
