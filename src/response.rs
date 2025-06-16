/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 * SPDX-FileCopyrightText: 2025 Franz Dietrich <dietrich@teilgedanken.de>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 */

use crate::modifications::Modification;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub action: Action,
    #[serde(default)]
    pub response: Option<SmtpResponse>,
    #[serde(default)]
    pub modifications: Vec<Modification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    #[serde(rename = "accept")]
    Accept,
    #[serde(rename = "discard")]
    Discard,
    #[serde(rename = "reject")]
    Reject,
    #[serde(rename = "quarantine")]
    Quarantine,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmtpResponse {
    #[serde(default)]
    pub status: Option<u16>,
    #[serde(default, rename = "enhancedStatus")]
    pub enhanced_status: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub disconnect: bool,
}

impl Default for Response {
    fn default() -> Self {
        Self {
            action: Action::Accept,
            response: None,
            modifications: Vec::new(),
        }
    }
}

impl Response {
    pub fn accept() -> Self {
        Self {
            action: Action::Accept,
            response: None,
            modifications: Vec::new(),
        }
    }

    pub fn reject(status: u16, message: String) -> Self {
        Self {
            action: Action::Reject,
            response: Some(SmtpResponse {
                status: Some(status),
                enhanced_status: None,
                message: Some(message),
                disconnect: false,
            }),
            modifications: Vec::new(),
        }
    }

    pub fn discard() -> Self {
        Self {
            action: Action::Discard,
            response: None,
            modifications: Vec::new(),
        }
    }

    /// Creates a quarantine response with no modifications
    ///
    /// Note: that quarantine is not yet implemented in Stalwart MTA
    /// see https://github.com/stalwartlabs/stalwart/issues/620
    pub fn quarantine() -> Self {
        Self {
            action: Action::Quarantine,
            response: None,
            modifications: Vec::new(),
        }
    }

    pub fn with_modifications(mut self, modifications: Vec<Modification>) -> Self {
        self.modifications = modifications;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifications::Modification;

    #[test]
    fn test_response_serialization() {
        let response = Response::accept();
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: Response = serde_json::from_str(&json).unwrap();

        match deserialized.action {
            Action::Accept => {}
            _ => panic!("Expected Accept action"),
        }
    }

    #[test]
    fn test_parse_docs_example() {
        // JSON example from the documentation (adapted for correct naming)
        let json = r#"{
    "action": "accept",
    "response": {
        "status": 250,
        "enhancedStatus": "2.0.0",
        "message": "Message accepted",
        "disconnect": false
    },
    "modifications": [
        {
            "type": "changeFrom",
            "value": "new@example.com",
            "parameters": {
                "size": "54321"
            }
        },
        {
            "type": "addRecipient",
            "value": "tom@example.com",
            "parameters": null
        },
        {
            "type": "deleteRecipient",
            "value": "jane@foobar.com"
        },
        {
            "type": "replaceContents",
            "value": "This is the new body\r\n"
        },
        {
            "type": "addHeader",
            "name": "X-Spam-Status",
            "value": "No"
        },
        {
            "type": "insertHeader",
            "index": 1,
            "name": "X-Filtered-By",
            "value": "Custom Filter v1.1"
        },
        {
            "type": "changeHeader",
            "index": 4,
            "name": "Subject",
            "value": "This is the new subject"
        },
        {
            "type": "deleteHeader",
            "index": 1,
            "name": "X-Mailer"
        }
    ]
}"#;

        let response: Response = serde_json::from_str(json).expect("Failed to parse JSON");

        // Verify action
        match response.action {
            Action::Accept => {}
            _ => panic!("Expected Accept action"),
        }

        // Verify response
        assert!(response.response.is_some());
        let smtp_response = response.response.unwrap();
        assert_eq!(smtp_response.status, Some(250));
        assert_eq!(smtp_response.enhanced_status, Some("2.0.0".to_string()));
        assert_eq!(smtp_response.message, Some("Message accepted".to_string()));
        assert_eq!(smtp_response.disconnect, false);

        // Verify modifications
        assert_eq!(response.modifications.len(), 8);

        // Test changeFrom modification
        match &response.modifications[0] {
            Modification::ChangeFrom { value, parameters } => {
                assert_eq!(value, "new@example.com");
                assert_eq!(parameters.get("size"), Some(&Some("54321".to_string())));
            }
            _ => panic!("Expected ChangeFrom modification"),
        }

        // Test addRecipient modification
        match &response.modifications[1] {
            Modification::AddRecipient { value, parameters } => {
                assert_eq!(value, "tom@example.com");
                assert!(parameters.is_empty());
            }
            _ => panic!("Expected AddRecipient modification"),
        }

        // Test deleteRecipient modification
        match &response.modifications[2] {
            Modification::DeleteRecipient { value } => {
                assert_eq!(value, "jane@foobar.com");
            }
            _ => panic!("Expected DeleteRecipient modification"),
        }

        // Test replaceContents modification
        match &response.modifications[3] {
            Modification::ReplaceContents { value } => {
                assert_eq!(value, "This is the new body\r\n");
            }
            _ => panic!("Expected ReplaceContents modification"),
        }

        // Test addHeader modification
        match &response.modifications[4] {
            Modification::AddHeader { name, value } => {
                assert_eq!(name, "X-Spam-Status");
                assert_eq!(value, "No");
            }
            _ => panic!("Expected AddHeader modification"),
        }

        // Test insertHeader modification
        match &response.modifications[5] {
            Modification::InsertHeader { index, name, value } => {
                assert_eq!(*index, 1);
                assert_eq!(name, "X-Filtered-By");
                assert_eq!(value, "Custom Filter v1.1");
            }
            _ => panic!("Expected InsertHeader modification"),
        }

        // Test changeHeader modification
        match &response.modifications[6] {
            Modification::ChangeHeader { index, name, value } => {
                assert_eq!(*index, 4);
                assert_eq!(name, "Subject");
                assert_eq!(value, "This is the new subject");
            }
            _ => panic!("Expected ChangeHeader modification"),
        }

        // Test deleteHeader modification
        match &response.modifications[7] {
            Modification::DeleteHeader { index, name } => {
                assert_eq!(*index, 1);
                assert_eq!(name, "X-Mailer");
            }
            _ => panic!("Expected DeleteHeader modification"),
        }
    }

    #[test]
    fn test_parse_docs_example_with_integer_parameters() {
        // JSON example with integer parameters instead of string
        let json = r#"{
    "action": "accept",
    "response": {
        "status": 250,
        "enhancedStatus": "2.0.0",
        "message": "Message accepted",
        "disconnect": false
    },
    "modifications": [
        {
            "type": "changeFrom",
            "value": "new@example.com",
            "parameters": {
                "size": 54321
            }
        }
    ]
}"#;

        let response: Response = serde_json::from_str(json).expect("Failed to parse JSON");

        // Verify action
        match response.action {
            Action::Accept => {}
            _ => panic!("Expected Accept action"),
        }

        // Verify modifications
        assert_eq!(response.modifications.len(), 1);

        // Test changeFrom modification with integer parameter
        match &response.modifications[0] {
            Modification::ChangeFrom { value, parameters } => {
                assert_eq!(value, "new@example.com");
                assert_eq!(parameters.get("size"), Some(&Some("54321".to_string())));
            }
            _ => panic!("Expected ChangeFrom modification"),
        }
    }
}
