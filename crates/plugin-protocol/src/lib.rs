use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    Stt,
    Tts,
    Adapter,
    Sync,
    Team,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub protocol_version: String,
    pub command: Vec<String>,
    pub capabilities: Vec<PluginCapability>,
}

impl PluginManifest {
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_rpc_round_trip() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: "ping".into(),
            params: None,
            id: Some(serde_json::json!(1)),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);

        let resp = JsonRpcResponse {
            jsonrpc: "2.0".into(),
            result: Some(serde_json::json!({"ok": true})),
            error: None,
            id: Some(serde_json::json!(1)),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: JsonRpcResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back, resp);
    }

    #[test]
    fn manifest_toml_round_trip() {
        let toml_str = r#"
name = "example"
version = "0.0.1"
protocol_version = "1"
command = ["lira-plugin-example"]
capabilities = ["stt", "adapter"]
"#;
        let manifest = PluginManifest::from_toml_str(toml_str).unwrap();
        assert_eq!(manifest.name, "example");
        assert_eq!(manifest.protocol_version, PROTOCOL_VERSION);
        assert_eq!(manifest.capabilities.len(), 2);
        let again = toml::to_string(&manifest).unwrap();
        let back: PluginManifest = toml::from_str(&again).unwrap();
        assert_eq!(back, manifest);
    }
}
