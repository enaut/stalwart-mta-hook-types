/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 * SPDX-FileCopyrightText: 2025 Franz Dietrich <dietrich@teilgedanken.de>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 */

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// Custom deserializer to handle null as empty HashMap and convert integers to strings
fn deserialize_null_as_empty_map<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<HashMap<String, Option<Value>>> = Option::deserialize(deserializer)?;

    match opt {
        None => Ok(HashMap::new()),
        Some(map) => {
            let mut result = HashMap::new();
            for (key, value) in map {
                let string_value = match value {
                    Some(Value::String(s)) => Some(s),
                    Some(Value::Number(n)) => Some(n.to_string()),
                    Some(Value::Bool(b)) => Some(b.to_string()),
                    Some(_) => Some(value.unwrap().to_string()),
                    None => None,
                };
                result.insert(key, string_value);
            }
            Ok(result)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Modification {
    #[serde(rename = "changeFrom")]
    ChangeFrom {
        value: String,
        #[serde(default, deserialize_with = "deserialize_null_as_empty_map")]
        parameters: HashMap<String, Option<String>>,
    },
    #[serde(rename = "addRecipient")]
    AddRecipient {
        value: String,
        #[serde(default, deserialize_with = "deserialize_null_as_empty_map")]
        parameters: HashMap<String, Option<String>>,
    },
    #[serde(rename = "deleteRecipient")]
    DeleteRecipient { value: String },
    #[serde(rename = "replaceContents")]
    ReplaceContents { value: String },
    #[serde(rename = "addHeader")]
    AddHeader { name: String, value: String },
    #[serde(rename = "insertHeader")]
    InsertHeader {
        index: u32,
        name: String,
        value: String,
    },
    #[serde(rename = "changeHeader")]
    ChangeHeader {
        index: u32,
        name: String,
        value: String,
    },
    #[serde(rename = "deleteHeader")]
    DeleteHeader { index: u32, name: String },
}

impl Modification {
    pub fn change_from(address: String) -> Self {
        Self::ChangeFrom {
            value: address,
            parameters: HashMap::new(),
        }
    }

    pub fn change_from_with_params(
        address: String,
        parameters: HashMap<String, Option<String>>,
    ) -> Self {
        Self::ChangeFrom {
            value: address,
            parameters,
        }
    }

    pub fn add_recipient(address: String) -> Self {
        Self::AddRecipient {
            value: address,
            parameters: HashMap::new(),
        }
    }

    pub fn add_recipient_with_params(
        address: String,
        parameters: HashMap<String, Option<String>>,
    ) -> Self {
        Self::AddRecipient {
            value: address,
            parameters,
        }
    }

    pub fn delete_recipient(address: String) -> Self {
        Self::DeleteRecipient { value: address }
    }

    pub fn replace_contents(contents: String) -> Self {
        Self::ReplaceContents { value: contents }
    }

    pub fn add_header(name: String, value: String) -> Self {
        Self::AddHeader { name, value }
    }

    pub fn insert_header(index: u32, name: String, value: String) -> Self {
        Self::InsertHeader { index, name, value }
    }

    pub fn change_header(index: u32, name: String, value: String) -> Self {
        Self::ChangeHeader { index, name, value }
    }

    pub fn delete_header(index: u32, name: String) -> Self {
        Self::DeleteHeader { index, name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modification_serialization() {
        let mod_add_header =
            Modification::add_header("X-Test".to_string(), "test-value".to_string());

        let json = serde_json::to_string(&mod_add_header).unwrap();
        let deserialized: Modification = serde_json::from_str(&json).unwrap();

        match deserialized {
            Modification::AddHeader { name, value } => {
                assert_eq!(name, "X-Test");
                assert_eq!(value, "test-value");
            }
            _ => panic!("Expected AddHeader modification"),
        }
    }

    #[test]
    fn test_anonymous_modifications() {
        // Test with anonymized data
        let mod_change_from = Modification::change_from("user@example.org".to_string());
        let mod_add_recipient = Modification::add_recipient("recipient@example.org".to_string());

        // Ensure anonymized data is used in tests
        match mod_change_from {
            Modification::ChangeFrom { value, .. } => {
                assert_eq!(value, "user@example.org");
            }
            _ => panic!("Expected ChangeFrom modification"),
        }

        match mod_add_recipient {
            Modification::AddRecipient { value, .. } => {
                assert_eq!(value, "recipient@example.org");
            }
            _ => panic!("Expected AddRecipient modification"),
        }
    }

    #[test]
    fn test_null_parameters_deserialization() {
        // Test that "parameters": null is deserialized as empty HashMap
        let json = r#"{
            "type": "addRecipient",
            "value": "test@example.com",
            "parameters": null
        }"#;

        let modification: Modification =
            serde_json::from_str(json).expect("Failed to parse JSON with null parameters");

        match modification {
            Modification::AddRecipient { value, parameters } => {
                assert_eq!(value, "test@example.com");
                assert!(
                    parameters.is_empty(),
                    "parameters should be empty when null"
                );
            }
            _ => panic!("Expected AddRecipient modification"),
        }
    }

    #[test]
    fn test_missing_parameters_deserialization() {
        // Test that missing "parameters" field is deserialized as empty HashMap
        let json = r#"{
            "type": "addRecipient",
            "value": "test@example.com"
        }"#;

        let modification: Modification =
            serde_json::from_str(json).expect("Failed to parse JSON with missing parameters");

        match modification {
            Modification::AddRecipient { value, parameters } => {
                assert_eq!(value, "test@example.com");
                assert!(
                    parameters.is_empty(),
                    "parameters should be empty when missing"
                );
            }
            _ => panic!("Expected AddRecipient modification"),
        }
    }

    #[test]
    fn test_integer_parameters_deserialization() {
        // Test that integer parameters are converted to strings
        let json = r#"{
            "type": "changeFrom",
            "value": "new@example.com",
            "parameters": {
                "size": 54321
            }
        }"#;

        let modification: Modification =
            serde_json::from_str(json).expect("Failed to parse JSON with integer parameters");

        match modification {
            Modification::ChangeFrom { value, parameters } => {
                assert_eq!(value, "new@example.com");
                assert_eq!(parameters.get("size"), Some(&Some("54321".to_string())));
            }
            _ => panic!("Expected ChangeFrom modification"),
        }
    }

    #[test]
    fn test_string_parameters_deserialization() {
        // Test that string parameters work as before
        let json = r#"{
            "type": "changeFrom",
            "value": "new@example.com",
            "parameters": {
                "size": "54321"
            }
        }"#;

        let modification: Modification =
            serde_json::from_str(json).expect("Failed to parse JSON with string parameters");

        match modification {
            Modification::ChangeFrom { value, parameters } => {
                assert_eq!(value, "new@example.com");
                assert_eq!(parameters.get("size"), Some(&Some("54321".to_string())));
            }
            _ => panic!("Expected ChangeFrom modification"),
        }
    }

    #[test]
    fn test_mixed_parameters_deserialization() {
        // Test that mixed parameter types work
        let json = r#"{
            "type": "changeFrom",
            "value": "new@example.com",
            "parameters": {
                "size": 54321,
                "priority": "high",
                "enabled": true
            }
        }"#;

        let modification: Modification =
            serde_json::from_str(json).expect("Failed to parse JSON with mixed parameters");

        match modification {
            Modification::ChangeFrom { value, parameters } => {
                assert_eq!(value, "new@example.com");
                assert_eq!(parameters.get("size"), Some(&Some("54321".to_string())));
                assert_eq!(parameters.get("priority"), Some(&Some("high".to_string())));
                assert_eq!(parameters.get("enabled"), Some(&Some("true".to_string())));
            }
            _ => panic!("Expected ChangeFrom modification"),
        }
    }
}
