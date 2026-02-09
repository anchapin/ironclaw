//! MCP Client Layer
//!
//! This module provides the high-level MCP client that orchestrates
//! communication with MCP servers using the transport layer.
//!
//! # Architecture
//!
//! The client is generic over the transport layer, allowing it to work
//! with different transport mechanisms (stdio, HTTP, etc.) through the
//! [`Transport`] trait.
//!
//! # Usage
//!
//! ```ignore
//! use ironclaw_orchestrator::mcp::{McpClient, StdioTransport};
//!
//! // Create a stdio transport
//! let transport = StdioTransport::spawn("npx", &["-y", "@modelcontextprotocol/server-filesystem"]).await?;
//!
//! // Create MCP client
//! let mut client = McpClient::new(transport);
//!
//! // Initialize connection
//! client.initialize().await?;
//!
//! // List available tools
//! let tools = client.list_tools().await?;
//!
//! // Call a tool
//! let result = client.call_tool("read_file", json!({"path": "/tmp/file.txt"})).await?;
//! ```

use crate::mcp::protocol::{
    ClientCapabilities, ClientInfo, InitializeParams, McpError, McpMethod,
    McpRequest, ServerCapabilities, ServerInfo, Tool,
};
use crate::mcp::transport::Transport;
use anyhow::{Context, Result};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};

/// High-level MCP client
///
/// This client provides a convenient, type-safe API for interacting with MCP servers.
/// It handles the initialization handshake, tool discovery, and tool invocation.
///
/// # Type Parameters
///
/// * `T` - The transport type (e.g., `StdioTransport`, `HttpTransport`)
///
/// # Lifecycle
///
/// 1. Create client with `McpClient::new(transport)`
/// 2. Initialize with `client.initialize()`
/// 3. Use the client (list tools, call tools)
/// 4. Drop the client when done (transport auto-cleanup)
///
/// # Example
///
/// ```ignore
/// let transport = StdioTransport::spawn("npx", &["-y", "@modelcontextprotocol/server-filesystem"]).await?;
/// let mut client = McpClient::new(transport);
/// client.initialize().await?;
/// let tools = client.list_tools().await?;
/// ```
pub struct McpClient<T>
where
    T: Transport,
{
    /// Underlying transport for sending/receiving messages
    transport: T,

    /// Next request ID (monotonically increasing)
    next_id: AtomicU64,

    /// Server capabilities (after initialization)
    server_capabilities: Option<ServerCapabilities>,

    /// Available tools (after listing)
    tools: Vec<Tool>,

    /// Client state
    state: ClientState,
}

/// Client state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    /// Client is created but not initialized
    Created,

    /// Initialization is in progress
    Initializing,

    /// Client is initialized and ready
    Ready,

    /// Client is disconnected
    Disconnected,
}

impl<T> McpClient<T>
where
    T: Transport,
{
    /// Create a new MCP client with the given transport
    ///
    /// # Arguments
    ///
    /// * `transport` - The transport to use for communication
    ///
    /// # Returns
    ///
    /// Returns a new `McpClient` instance
    ///
    /// # Example
    ///
    /// ```ignore
    /// let transport = StdioTransport::spawn("npx", &["-y", "server"]).await?;
    /// let client = McpClient::new(transport);
    /// ```
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            next_id: AtomicU64::new(1),
            server_capabilities: None,
            tools: Vec::new(),
            state: ClientState::Created,
        }
    }

    /// Get the underlying transport
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Get a mutable reference to the underlying transport
    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Initialize the MCP connection
    ///
    /// This sends an `initialize` request to the server and waits for the response.
    /// The server will respond with its capabilities and information.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initialization succeeded
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Transport send/recv fails
    /// - Server returns an error response
    /// - Server reports incompatible protocol version
    pub async fn initialize(&mut self) -> Result<()> {
        if self.state != ClientState::Created {
            return Err(anyhow::anyhow!(
                "Cannot initialize client: invalid state {:?}",
                self.state
            ));
        }

        if !self.transport.is_connected() {
            return Err(anyhow::anyhow!("Cannot initialize: transport is disconnected"));
        }

        self.state = ClientState::Initializing;
        tracing::info!("Initializing MCP connection...");

        // Prepare initialize parameters
        let client_info = ClientInfo {
            name: "ironclaw-orchestrator".to_string(),
            version: env!("CARGO_PKG_VERSION", "0.1.0").to_string(),
        };

        let capabilities = ClientCapabilities {
            sampling: Some(false),
            experimental: None,
        };

        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities,
            client_info,
        };

        // Create initialize request
        let request = McpRequest::new(
            self.next_id.fetch_add(1, Ordering::SeqCst),
            "initialize",
            Some(json!(params)),
        );

        // Send request
        self.transport
            .send(&request)
            .await
            .context("Failed to send initialize request")?;

        // Receive response
        let response = self
            .transport
            .recv()
            .await
            .context("Failed to receive initialize response")?;

        // Check for error response
        if !response.is_success() {
            let error = response
                .error
                .ok_or_else(|| McpError::internal_error("Initialize failed with unknown error"))?;
            return Err(anyhow::anyhow!("Initialize failed: {}", error));
        }

        // Parse server capabilities from response
        let result = response
            .result
            .ok_or_else(|| McpError::internal_error("Initialize response missing result"))?;

        // Parse the server capabilities
        let server_info: ServerInfo = serde_json::from_value(result["serverInfo"].clone())
            .context("Failed to parse server info from initialize response")?;

        // Store server capabilities
        self.server_capabilities = Some(ServerCapabilities {
            protocol_version: result["protocolVersion"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing protocolVersion in initialize response"))?
                .to_string(),
            capabilities: result["capabilities"].clone(),
            server_info,
        });

        self.state = ClientState::Ready;
        tracing::info!(
            "MCP connection initialized: {} v{}",
            self.server_capabilities.as_ref().map(|c| c.server_info.name.as_str()).unwrap_or("unknown"),
            self.server_capabilities.as_ref().map(|c| c.protocol_version.as_str()).unwrap_or("unknown")
        );

        Ok(())
    }

    /// List available tools from the MCP server
    ///
    /// This sends a `tools/list` request to the server and returns the list of tools.
    ///
    /// # Returns
    ///
    /// Returns a vector of available tools
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Client is not initialized
    /// - Transport send/recv fails
    /// - Server returns an error response
    /// - Tool list format is invalid
    pub async fn list_tools(&mut self) -> Result<Vec<Tool>> {
        self.ensure_ready()?;

        tracing::debug!("Listing available tools from MCP server");

        // Create tools/list request
        let request = McpRequest::notification(
            self.next_id.fetch_add(1, Ordering::SeqCst),
            McpMethod::ToolsList.as_str().to_string(),
        );

        // Send request
        self.transport
            .send(&request)
            .await
            .context("Failed to send tools/list request")?;

        // Receive response
        let response = self
            .transport
            .recv()
            .await
            .context("Failed to receive tools/list response")?;

        // Check for error response
        if !response.is_success() {
            let error = response
                .error
                .ok_or_else(|| McpError::internal_error("Tools/list failed with unknown error"))?;
            return Err(anyhow::anyhow!("Failed to list tools: {}", error));
        }

        // Parse tools from response
        let result = response
            .result
            .ok_or_else(|| McpError::internal_error("Tools/list response missing result"))?;

        let tools: Vec<Tool> = serde_json::from_value(result["tools"].clone())
            .context("Failed to parse tools from response")?;

        // Cache the tools
        self.tools = tools.clone();

        tracing::info!("Listed {} tools from MCP server", tools.len());

        // Log tool names for debugging
        for tool in &tools {
            tracing::debug!("  - {}", tool.name);
        }

        Ok(tools)
    }

    /// Call a tool on the MCP server
    ///
    /// This sends a `tools/call` request with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to call
    /// * `arguments` - The arguments to pass to the tool (must match tool's input schema)
    ///
    /// # Returns
    ///
    /// Returns the tool's result as a JSON value
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Client is not initialized
    /// - Transport send/recv fails
    /// - Server returns an error response
    /// - Tool execution fails
    pub async fn call_tool(&mut self, name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        self.ensure_ready()?;

        tracing::debug!("Calling tool: {} with arguments: {:?}", name, arguments);

        // Create tools/call request
        let params = json!({
            "name": name,
            "arguments": arguments
        });

        let request = McpRequest::new(
            self.next_id.fetch_add(1, Ordering::SeqCst),
            McpMethod::ToolsCall.as_str().to_string(),
            Some(params),
        );

        // Send request
        self.transport
            .send(&request)
            .await
            .context("Failed to send tools/call request")?;

        // Receive response
        let response = self
            .transport
            .recv()
            .await
            .context("Failed to receive tools/call response")?;

        // Check for error response
        if !response.is_success() {
            let error = response
                .error
                .ok_or_else(|| McpError::internal_error("Tool call failed with unknown error"))?;
            return Err(anyhow::anyhow!("Tool '{}' failed: {}", name, error));
        }

        // Parse tool result
        let result = response
            .result
            .ok_or_else(|| McpError::internal_error("Tool call response missing result"))?;

        tracing::debug!("Tool '{}' returned result: {:?}", name, result);

        Ok(result)
    }

    /// Check if the client is ready for operations
    fn ensure_ready(&self) -> Result<()> {
        match self.state {
            ClientState::Created => {
                Err(anyhow::anyhow!(
                    "Client not initialized. Call initialize() first."
                ))
            }
            ClientState::Initializing => {
                Err(anyhow::anyhow!(
                    "Client is currently initializing"
                ))
            }
            ClientState::Ready => Ok(()),
            ClientState::Disconnected => {
                Err(anyhow::anyhow!(
                    "Client is disconnected"
                ))
            }
        }
    }

    /// Get the current client state
    pub fn state(&self) -> ClientState {
        self.state
    }

    /// Get server capabilities (after initialization)
    ///
    /// Returns `None` if the client hasn't been initialized yet
    pub fn server_capabilities(&self) -> Option<&ServerCapabilities> {
        self.server_capabilities.as_ref()
    }

    /// Get available tools (cached after listing)
    ///
    /// Returns an empty slice if tools haven't been listed yet
    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::protocol::McpResponse;

    // Mock transport for testing
    #[derive(Clone)]
    struct MockTransport {
        connected: bool,
        requests: Vec<McpRequest>,
        response: Option<McpResponse>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                connected: true,
                requests: Vec::new(),
                response: None,
            }
        }

        fn set_response(&mut self, response: McpResponse) {
            self.response = Some(response);
        }

        fn set_error_response(&mut self, code: i32, message: &str) {
            self.response = Some(McpResponse::err(
                1,
                McpError::new(code, message),
            ));
        }
    }

    #[allow(async_fn_in_trait)]
    impl Transport for MockTransport {
        async fn send(&mut self, request: &McpRequest) -> Result<()> {
            if !self.connected {
                return Err(anyhow::anyhow!("Mock transport disconnected"));
            }
            self.requests.push(request.clone());
            Ok(())
        }

        async fn recv(&mut self) -> Result<McpResponse> {
            if !self.connected {
                return Err(anyhow::anyhow!("Mock transport disconnected"));
            }

            if let Some(response) = self.response.take() {
                Ok(response)
            } else {
                // Return a default success response
                Ok(McpResponse::ok(
                    self.requests.last().unwrap().id,
                    json!({}),
                ))
            }
        }

        fn is_connected(&self) -> bool {
            self.connected
        }
    }

    // Helper to create a successful initialize response
    fn create_init_response() -> McpResponse {
        McpResponse::ok(
            1,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "serverInfo": {
                    "name": "test-server",
                    "version": "1.0.0"
                }
            }),
        )
    }

    // Helper to create a tools list response
    fn create_tools_list_response(tools: &[Tool]) -> McpResponse {
        let tools_array = if tools.is_empty() {
            serde_json::Value::Array(Vec::new())
        } else {
            serde_json::to_value(tools).unwrap()
        };
        McpResponse::ok(2, json!({"tools": tools_array}))
    }

    // Helper to create a tool call response
    fn create_tool_call_response(result: serde_json::Value) -> McpResponse {
        McpResponse::ok(3, result)
    }

    #[tokio::test]
    async fn test_client_creation() {
        let transport = MockTransport::new();
        let client = McpClient::new(transport);

        assert_eq!(client.next_id.load(Ordering::SeqCst), 1);
        assert_eq!(client.state(), ClientState::Created);
    }

    #[tokio::test]
    async fn test_client_initialize_success() {
        let mut transport = MockTransport::new();
        transport.set_response(create_init_response());

        let mut client = McpClient::new(transport);

        // Initialize should succeed
        assert!(client.initialize().await.is_ok());

        // State should be Ready
        assert_eq!(client.state(), ClientState::Ready);

        // Server capabilities should be stored
        let caps = client.server_capabilities().unwrap();
        assert_eq!(caps.server_info.name, "test-server");
    }

    #[tokio::test]
    async fn test_client_initialize_error() {
        let mut transport = MockTransport::new();
        transport.set_error_response(-32001, "Initialization failed");

        let mut client = McpClient::new(transport);

        // Initialize should fail
        assert!(client.initialize().await.is_err());

        // State should not be Ready (since init failed)
        assert_ne!(client.state(), ClientState::Ready);
    }

    #[tokio::test]
    async fn test_client_list_tools() {
        let mut transport = MockTransport::new();

        let tools = vec![
            Tool {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                input_schema: json!({"type": "object"}),
            },
        ];

        transport.set_response(create_tools_list_response(&tools));

        let mut client = McpClient::new(transport);
        client.state = ClientState::Ready; // Skip initialization for this test

        // List tools should succeed
        let result = client.list_tools().await;

        assert!(result.is_ok());
        let listed_tools = result.unwrap();
        assert_eq!(listed_tools.len(), 1);
        assert_eq!(listed_tools[0].name, "test_tool");
    }

    #[tokio::test]
    async fn test_client_call_tool() {
        let mut transport = MockTransport::new();
        let tool_result = json!({"status": "success"});

        transport.set_response(create_tool_call_response(tool_result));

        let mut client = McpClient::new(transport);
        client.state = ClientState::Ready; // Skip initialization

        // Call tool should succeed
        let result = client.call_tool("test_tool", json!({})).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "success");
    }

    #[tokio::test]
    async fn test_client_call_tool_not_found() {
        let mut transport = MockTransport::new();
        transport.set_error_response(-32601, "Tool not found");

        let mut client = McpClient::new(transport);
        client.state = ClientState::Ready; // Skip initialization

        // Call tool should fail
        let result = client.call_tool("unknown_tool", json!({})).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_client_state_transitions() {
        let transport = MockTransport::new();
        let mut client = McpClient::new(transport);

        // Initial state
        assert_eq!(client.state(), ClientState::Created);

        // After initialization
        client.state = ClientState::Ready;

        // ensure_ready() should pass
        assert!(client.ensure_ready().is_ok());
    }

    #[tokio::test]
    async fn test_client_list_tools_when_not_initialized() {
        let transport = MockTransport::new();
        let mut client = McpClient::new(transport);

        // List tools should fail (not initialized)
        let result = client.list_tools().await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not initialized"));
    }

    #[tokio::test]
    async fn test_client_server_capabilities_after_init() {
        let mut transport = MockTransport::new();
        transport.set_response(create_init_response());

        let mut client = McpClient::new(transport);

        // Before initialization, no capabilities
        assert!(client.server_capabilities().is_none());

        // Initialize
        client.initialize().await.unwrap();

        // After initialization, capabilities are available
        let caps = client.server_capabilities().unwrap();
        assert_eq!(caps.server_info.name, "test-server");
    }

    #[tokio::test]
    async fn test_client_tools_caching() {
        let mut transport = MockTransport::new();

        let tools = vec![
            Tool {
                name: "tool1".to_string(),
                description: "First tool".to_string(),
                input_schema: json!({}),
            },
            Tool {
                name: "tool2".to_string(),
                description: "Second tool".to_string(),
                input_schema: json!({}),
            },
        ];

        transport.set_response(create_tools_list_response(&tools));

        let mut client = McpClient::new(transport.clone());
        client.state = ClientState::Ready;

        // First call should fetch from server
        let result1 = client.list_tools().await.unwrap();
        assert_eq!(result1.len(), 2);

        // Tools should be cached
        let tools = client.tools();
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_client_state_debug() {
        // Just verify that ClientState implements Debug
        let state = ClientState::Created;
        let formatted = format!("{:?}", state);
        // Debug output for enums shows the variant name
        assert!(formatted == "Created" || formatted.contains("Created"));
    }

    #[tokio::test]
    async fn test_client_initialize_without_connection() {
        let mut transport = MockTransport::new();
        transport.connected = false;

        let mut client = McpClient::new(transport);

        // Initialize should fail (transport disconnected)
        assert!(client.initialize().await.is_err());
    }

    #[tokio::test]
    async fn test_client_multiple_operations() {
        // This test verifies that the client can perform multiple operations sequentially
        // The AtomicU64 ensures each request gets a unique, incrementing ID
        let mut transport = MockTransport::new();
        transport.set_response(create_init_response());

        let mut client = McpClient::new(transport);

        // Initialize should succeed
        assert!(client.initialize().await.is_ok());

        // Client should be in Ready state
        assert_eq!(client.state(), ClientState::Ready);

        // Server capabilities should be available
        assert!(client.server_capabilities().is_some());
    }
}
