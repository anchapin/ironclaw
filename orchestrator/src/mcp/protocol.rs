//! MCP Protocol Types (JSON-RPC 2.0)
//!
//! This module defines the core protocol types for the Model Context Protocol (MCP).
//! MCP is built on top of JSON-RPC 2.0, which is a simple stateless RPC protocol.
//!
//! # Protocol Specification
//!
//! - JSON-RPC 2.0: <https://www.jsonrpc.org/specification>
//! - MCP Spec: <https://modelcontextprotocol.io/specification/2025-03-26>
//!
//! # Architecture
//!
//! The protocol layer is responsible only for serialization/deserialization of MCP messages.
//! Transport concerns (stdio, HTTP) are handled in the transport layer.

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 version constant
pub const JSONRPC_VERSION: &str = "2.0";

/// A JSON-RPC 2.0 request message
///
/// Requests are sent from the client to the MCP server to invoke methods.
/// Each request has a unique ID (monotonically increasing) to match responses.
///
/// # Example
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "id": 1,
///   "method": "tools/list",
///   "params": {}
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpRequest {
    /// JSON-RPC version (always "2.0")
    #[serde(rename = "jsonrpc")]
    pub jsonrpc: String,

    /// Request identifier (used to match responses)
    pub id: u64,

    /// Method name to invoke
    pub method: String,

    /// Method parameters (optional, depends on method)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl McpRequest {
    /// Create a new MCP request
    ///
    /// # Arguments
    ///
    /// * `id` - Unique request identifier
    /// * `method` - Method name to invoke
    /// * `params` - Optional method parameters
    pub fn new(id: u64, method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method: method.into(),
            params,
        }
    }

    /// Create a request without parameters
    pub fn notification(id: u64, method: impl Into<String>) -> Self {
        Self::new(id, method, None)
    }
}

/// A JSON-RPC 2.0 response message
///
/// Responses are sent from the MCP server back to the client.
/// A response either contains a `result` or an `error`, but never both.
///
/// # Example (Success)
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "id": 1,
///   "result": {"tools": [...]}
/// }
/// ```
///
/// # Example (Error)
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "id": 1,
///   "error": {"code": -32601, "message": "Method not found"}
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpResponse {
    /// JSON-RPC version (always "2.0")
    #[serde(rename = "jsonrpc")]
    pub jsonrpc: String,

    /// Request identifier (must match the request's ID)
    pub id: u64,

    /// Result payload (present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// Error information (present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

impl McpResponse {
    /// Create a successful response
    pub fn ok(id: u64, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn err(id: u64, error: McpError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }

    /// Check if the response is successful
    pub fn is_success(&self) -> bool {
        self.result.is_some() && self.error.is_none()
    }

    /// Get the result, or the error if unsuccessful
    pub fn into_result(self) -> Result<serde_json::Value, McpError> {
        match (self.result, self.error) {
            (Some(result), None) => Ok(result),
            (None, Some(error)) => Err(error),
            _ => Err(McpError::internal_error(
                "Invalid response: both result and error present",
            )),
        }
    }
}

/// A JSON-RPC 2.0 error object
///
/// Errors follow the JSON-RPC 2.0 specification with MCP-specific extensions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpError {
    /// Error code (JSON-RPC defined or MCP-specific)
    pub code: i32,

    /// Human-readable error message
    pub message: String,

    /// Additional error data (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl McpError {
    /// Create a new error
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create an error with additional data
    pub fn with_data(code: i32, message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    // JSON-RPC standard errors
    /// Parse error (-32700): Invalid JSON was received
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(-32700, message)
    }

    /// Invalid request (-32600): The JSON sent is not a valid Request object
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(-32600, message)
    }

    /// Method not found (-32601): The method does not exist / is not available
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self::new(-32601, format!("Method not found: {}", method.into()))
    }

    /// Invalid params (-32602): Invalid method parameter(s)
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(-32602, message)
    }

    /// Internal error (-32603): Internal JSON-RPC error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(-32603, message)
    }

    // MCP-specific errors (negative numbers beyond JSON-RPC range)
    /// Server error (-32000): MCP server error
    pub fn server_error(message: impl Into<String>) -> Self {
        Self::new(-32000, message)
    }

    /// Initialization error (-32001): Failed to initialize connection
    pub fn initialization_error(message: impl Into<String>) -> Self {
        Self::new(-32001, message)
    }
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Error {}] {}", self.code, self.message)
    }
}

impl std::error::Error for McpError {}

/// MCP method identifiers
///
/// MCP defines a set of standard methods that all servers must support.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum McpMethod {
    /// Initialize the connection (must be called first)
    Initialize,

    /// List available tools
    ToolsList,

    /// Call a specific tool
    ToolsCall,

    /// List available resources
    ResourcesList,

    /// Read a resource
    ResourcesRead,

    /// List available prompts
    PromptsList,

    /// Get a prompt
    PromptsGet,

    /// Custom method (for extensibility)
    Custom(String),
}

impl McpMethod {
    /// Convert to string for JSON-RPC method field
    pub fn as_str(&self) -> &str {
        match self {
            Self::Initialize => "initialize",
            Self::ToolsList => "tools/list",
            Self::ToolsCall => "tools/call",
            Self::ResourcesList => "resources/list",
            Self::ResourcesRead => "resources/read",
            Self::PromptsList => "prompts/list",
            Self::PromptsGet => "prompts/get",
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl From<String> for McpMethod {
    fn from(s: String) -> Self {
        match s.as_str() {
            "initialize" => Self::Initialize,
            "tools/list" => Self::ToolsList,
            "tools/call" => Self::ToolsCall,
            "resources/list" => Self::ResourcesList,
            "resources/read" => Self::ResourcesRead,
            "prompts/list" => Self::PromptsList,
            "prompts/get" => Self::PromptsGet,
            _ => Self::Custom(s),
        }
    }
}

impl From<&str> for McpMethod {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

/// Initialization parameters
///
/// Sent during the initialize handshake to negotiate capabilities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitializeParams {
    /// Client protocol version
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,

    /// Client capabilities
    pub capabilities: ClientCapabilities,

    /// Client information
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Client capabilities advertised during initialization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientCapabilities {
    /// Sampling capability (object or null)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<serde_json::Value>,

    /// Experimental features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<serde_json::Value>,
}

/// Client identification information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientInfo {
    /// Client name
    pub name: String,

    /// Client version
    pub version: String,
}

/// Server capabilities (returned during initialization)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerCapabilities {
    /// Server protocol version
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,

    /// Server capabilities
    pub capabilities: serde_json::Value,

    /// Server information
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// Server identification information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerInfo {
    /// Server name
    pub name: String,

    /// Server version
    pub version: String,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tool {
    /// Tool name (unique identifier)
    pub name: String,

    /// Tool description
    pub description: String,

    /// Tool input schema (JSON Schema)
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Tool call parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolCallParams {
    /// Name of the tool to call
    pub name: String,

    /// Tool arguments (must match input schema)
    pub arguments: serde_json::Value,
}

#[cfg(test)]
mod tests;
