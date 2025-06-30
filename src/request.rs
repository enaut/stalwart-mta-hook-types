/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 * SPDX-FileCopyrightText: 2025 Franz Dietrich <dietrich@teilgedanken.de>
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

fn deserialize_string_or_int_map<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, String>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, MapAccess, Visitor};
    use std::fmt;

    struct StringOrIntMapVisitor;

    impl<'de> Visitor<'de> for StringOrIntMapVisitor {
        type Value = Option<HashMap<String, String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map with string keys and string or integer values, or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(MapVisitor).map(Some)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    struct MapVisitor;

    impl<'de> Visitor<'de> for MapVisitor {
        type Value = HashMap<String, String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map with string keys and string or integer values")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = HashMap::new();

            while let Some(key) = access.next_key::<String>()? {
                let value: serde_json::Value = access.next_value()?;
                let string_value = match value {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => return Err(de::Error::custom("invalid parameter value type")),
                };
                map.insert(key, string_value);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_option(StringOrIntMapVisitor)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub context: Context,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub envelope: Option<Envelope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub stage: Stage,
    pub client: Client,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sasl: Option<Sasl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<Tls>,
    pub server: Server,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue: Option<Queue>,
    pub protocol: Protocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sasl {
    pub login: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub ip: String,
    pub port: u16,
    pub ptr: Option<String>,
    pub helo: Option<String>,
    #[serde(rename = "activeConnections")]
    pub active_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tls {
    pub version: String,
    pub cipher: String,
    #[serde(rename = "cipherBits")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bits: Option<u16>,
    #[serde(rename = "certIssuer")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    #[serde(rename = "certSubject")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub name: Option<String>,
    pub port: u16,
    pub ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    pub version: u32,
}

#[derive(Debug, Clone)]
pub enum Stage {
    Connect,
    Ehlo,
    Auth,
    Mail,
    Rcpt,
    Data,
}

impl Serialize for Stage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let stage_str = match self {
            Stage::Connect => "connect",
            Stage::Ehlo => "ehlo",
            Stage::Auth => "auth",
            Stage::Mail => "mail",
            Stage::Rcpt => "rcpt",
            Stage::Data => "data",
        };
        serializer.serialize_str(stage_str)
    }
}

impl<'de> Deserialize<'de> for Stage {
    fn deserialize<D>(deserializer: D) -> Result<Stage, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StageVisitor;

        impl<'de> serde::de::Visitor<'de> for StageVisitor {
            type Value = Stage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing a Stage variant (case-insensitive)")
            }

            fn visit_str<E>(self, value: &str) -> Result<Stage, E>
            where
                E: serde::de::Error,
            {
                match value.to_ascii_uppercase().as_str() {
                    "CONNECT" => Ok(Stage::Connect),
                    "EHLO" => Ok(Stage::Ehlo),
                    "AUTH" => Ok(Stage::Auth),
                    "MAIL" => Ok(Stage::Mail),
                    "RCPT" => Ok(Stage::Rcpt),
                    "DATA" => Ok(Stage::Data),
                    _ => Err(E::unknown_variant(
                        value,
                        &["CONNECT", "EHLO", "AUTH", "MAIL", "RCPT", "DATA"],
                    )),
                }
            }
        }

        deserializer.deserialize_str(StageVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_string_or_int_map")]
    #[serde(default)]
    pub parameters: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub from: Address,
    pub to: Vec<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub headers: Vec<(String, String)>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "serverHeaders")]
    #[serde(default)]
    pub server_headers: Vec<(String, String)>,
    pub contents: String,
    pub size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_docs_example() {
        // JSON example from the documentation
        let json = r#"{
    "context": {
        "stage": "DATA",
        "sasl": {
            "login": "user",
            "method": "plain"
        },
        "client": {
            "ip": "192.168.1.1",
            "port": 34567,
            "ptr": "mail.example.com",
            "helo": "mail.example.com",
            "activeConnections": 1
        },
        "tls": {
            "version": "1.3",
            "cipher": "TLS_AES_256_GCM_SHA384",
            "cipherBits": 256,
            "certIssuer": "Let's Encrypt",
            "certSubject": "mail.example.com"
        },
        "server": {
            "name": "Stalwart",
            "port": 25,
            "ip": "192.168.2.2"
        },
        "queue": {
            "id": "1234567890"
        },
        "protocol": {
            "version": 1
        }
    },
    "envelope": {
        "from": {
            "address": "john@example.com",
            "parameters": {
                "size": "12345"
            }
        },
        "to": [
            {
                "address": "bill@foobar.com",
                "parameters": {
                    "orcpt": "rfc822; b@foobar.com"
                }
            },
            {
                "address": "jane@foobar.com",
                "parameters": null
            }
        ]
    },
    "message": {
        "headers": [
            [
                "From",
                "John Doe <john@example.com>"
            ],
            [
                "To",
                "Bill <bill@foobar.com>, Jane <jane@foobar.com>"
            ],
            [
                "Subject",
                "Hello, World!"
            ]
        ],
        "serverHeaders": [
            [
                "Received",
                "from mail.example.com (mail.example.com [192.168.1.1]) by mail.foobar.com (Stalwart) with ESMTPS id 1234567890"
            ]
        ],
        "contents": "Hello, World!\r\n",
        "size": 12345
    }
}"#;

        let request: Request = serde_json::from_str(json).expect("Failed to parse JSON");

        // Verify context
        match request.context.stage {
            Stage::Data => {}
            _ => panic!("Expected Data stage"),
        }

        // Verify SASL
        assert!(request.context.sasl.is_some());
        let sasl = request.context.sasl.unwrap();
        assert_eq!(sasl.login, "user");
        assert_eq!(sasl.method, Some("plain".to_string()));

        // Verify client
        let client = &request.context.client;
        assert_eq!(client.ip, "192.168.1.1");
        assert_eq!(client.port, 34567);
        assert_eq!(client.ptr, Some("mail.example.com".to_string()));
        assert_eq!(client.helo, Some("mail.example.com".to_string()));
        assert_eq!(client.active_connections, 1);

        // Verify TLS
        assert!(request.context.tls.is_some());
        let tls = request.context.tls.unwrap();
        assert_eq!(tls.version, "1.3");
        assert_eq!(tls.cipher, "TLS_AES_256_GCM_SHA384");
        assert_eq!(tls.bits, Some(256));
        assert_eq!(tls.issuer, Some("Let's Encrypt".to_string()));
        assert_eq!(tls.subject, Some("mail.example.com".to_string()));

        // Verify server
        let server = &request.context.server;
        assert_eq!(server.name, Some("Stalwart".to_string()));
        assert_eq!(server.port, 25);
        assert_eq!(server.ip, Some("192.168.2.2".to_string()));

        // Verify queue
        assert!(request.context.queue.is_some());
        let queue = request.context.queue.unwrap();
        assert_eq!(queue.id, "1234567890");

        // Verify protocol
        let protocol = &request.context.protocol;
        assert_eq!(protocol.version, 1);

        // Verify envelope
        assert!(request.envelope.is_some());
        let envelope = request.envelope.unwrap();

        // Verify from address
        assert_eq!(envelope.from.address, "john@example.com");
        assert!(envelope.from.parameters.is_some());
        let from_params = envelope.from.parameters.unwrap();
        assert_eq!(from_params.get("size"), Some(&"12345".to_string()));

        // Verify to addresses
        assert_eq!(envelope.to.len(), 2);

        // First recipient
        assert_eq!(envelope.to[0].address, "bill@foobar.com");
        assert!(envelope.to[0].parameters.is_some());
        let bill_params = envelope.to[0].parameters.as_ref().unwrap();
        assert_eq!(
            bill_params.get("orcpt"),
            Some(&"rfc822; b@foobar.com".to_string())
        );

        // Second recipient
        assert_eq!(envelope.to[1].address, "jane@foobar.com");
        assert!(envelope.to[1].parameters.is_none());

        // Verify message
        assert!(request.message.is_some());
        let message = request.message.unwrap();

        // Verify headers
        assert_eq!(message.headers.len(), 3);
        assert_eq!(
            message.headers[0],
            (
                "From".to_string(),
                "John Doe <john@example.com>".to_string()
            )
        );
        assert_eq!(
            message.headers[1],
            (
                "To".to_string(),
                "Bill <bill@foobar.com>, Jane <jane@foobar.com>".to_string()
            )
        );
        assert_eq!(
            message.headers[2],
            ("Subject".to_string(), "Hello, World!".to_string())
        );

        // Verify server headers
        assert_eq!(message.server_headers.len(), 1);
        assert_eq!(message.server_headers[0].0, "Received");
        assert!(message.server_headers[0]
            .1
            .contains("from mail.example.com"));

        // Verify contents and size
        assert_eq!(message.contents, "Hello, World!\r\n");
        assert_eq!(message.size, 12345);
    }

    #[test]
    fn test_parse_docs_example_with_integer_parameters() {
        // JSON example with integer parameters instead of string
        let json = r#"{
    "context": {
        "stage": "DATA",
        "sasl": {
            "login": "user",
            "method": "plain"
        },
        "client": {
            "ip": "192.168.1.1",
            "port": 34567,
            "ptr": "mail.example.com",
            "ehlo": "mail.example.com",
            "activeConnections": 1
        },
        "tls": {
            "version": "1.3",
            "cipher": "TLS_AES_256_GCM_SHA384",
            "cipherBits": 256,
            "certIssuer": "Let's Encrypt",
            "certSubject": "mail.example.com"
        },
        "server": {
            "name": "Stalwart",
            "port": 25,
            "ip": "192.168.2.2"
        },
        "queue": {
            "id": "1234567890"
        },
        "protocol": {
            "version": 1
        }
    },
    "envelope": {
        "from": {
            "address": "john@example.com",
            "parameters": {
                "size": 12345
            }
        },
        "to": [
            {
                "address": "bill@foobar.com",
                "parameters": {
                    "orcpt": "rfc822; b@foobar.com"
                }
            },
            {
                "address": "jane@foobar.com",
                "parameters": null
            }
        ]
    },
    "message": {
        "headers": [
            [
                "From",
                "John Doe <john@example.com>"
            ],
            [
                "To",
                "Bill <bill@foobar.com>, Jane <jane@foobar.com>"
            ],
            [
                "Subject",
                "Hello, World!"
            ]
        ],
        "serverHeaders": [
            [
                "Received",
                "from mail.example.com (mail.example.com [192.168.1.1]) by mail.foobar.com (Stalwart) with ESMTPS id 1234567890"
            ]
        ],
        "contents": "Hello, World!\r\n",
        "size": 12345
    }
}"#;

        let request: Request = serde_json::from_str(json).expect("Failed to parse JSON");

        // Verify envelope with integer parameter
        assert!(request.envelope.is_some());
        let envelope = request.envelope.unwrap();

        // Verify from address with integer parameter converted to string
        assert_eq!(envelope.from.address, "john@example.com");
        assert!(envelope.from.parameters.is_some());
        let from_params = envelope.from.parameters.unwrap();
        assert_eq!(from_params.get("size"), Some(&"12345".to_string()));
    }

    #[test]
    fn test_request_serialization() {
        let mut from_params = HashMap::new();
        from_params.insert("size".to_string(), "1000".to_string());

        let request = Request {
            context: Context {
                stage: Stage::Mail,
                client: Client {
                    ip: "127.0.0.1".to_string(),
                    port: 12345,
                    ptr: None,
                    helo: Some("localhost".to_string()),
                    active_connections: 1,
                },
                sasl: None,
                tls: None,
                server: Server {
                    name: Some("Test Server".to_string()),
                    port: 25,
                    ip: Some("127.0.0.1".to_string()),
                },
                queue: None,
                protocol: Protocol { version: 1 },
            },
            envelope: Some(Envelope {
                from: Address {
                    address: "test@example.com".to_string(),
                    parameters: Some(from_params),
                },
                to: vec![Address {
                    address: "recipient@example.com".to_string(),
                    parameters: None,
                }],
            }),
            message: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: Request = serde_json::from_str(&json).unwrap();

        match deserialized.context.stage {
            Stage::Mail => {}
            _ => panic!("Expected Mail stage"),
        }

        assert!(deserialized.envelope.is_some());
        let envelope = deserialized.envelope.unwrap();
        assert_eq!(envelope.from.address, "test@example.com");
        assert_eq!(envelope.to.len(), 1);
        assert_eq!(envelope.to[0].address, "recipient@example.com");
    }
}
