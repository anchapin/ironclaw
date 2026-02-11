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
    ClientCapabilities, ClientInfo, InitializeParams, McpError, McpMethod, McpRequest, McpResponse,
    ServerCapabilities, ServerInfo, Tool,
};
use crate::mcp::retry::RetryConfig;
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

    /// Retry configuration for transient failures
    retry_config: Option<RetryConfig>,
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
            retry_config: None,
        }
    }

    /// Set retry configuration for the client
    ///
    /// # Arguments
    ///
    /// * `config` - Retry configuration
    ///
    /// # Returns
    ///
    /// Returns `self` for chaining
    ///
    /// # Example
    ///
    /// ```ignore
    /// let client = McpClient::new(transport)
    ///     .with_retry(RetryConfig::default().max_attempts(5));
    /// ```
    pub fn with_retry(mut self, config: RetryConfig) -> Self {
        self.retry_config = Some(config);
        self
    }

    /// Send a request and receive a response (with optional retry)
    ///
    /// This is a helper method that wraps the send/recv pattern with retry logic
    /// if a retry config is set.
    ///
    /// # Arguments
    ///
    /// * `request` - The MCP request to send
    ///
    /// # Returns
    ///
    /// Returns the MCP response
    async fn send_request(&mut self, request: &McpRequest) -> Result<McpResponse> {
        if let Some(config) = self.retry_config.clone() {
            // Use retry logic - manually implemented to avoid borrow issues
            let mut last_error = None;

            for attempt in 0..config.max_attempts {
                match self.transport.send(request).await {
                    Ok(()) => match self.transport.recv().await {
                        Ok(response) => {
                            if attempt > 0 {
                                tracing::info!(
                                    "Request succeeded on attempt {} after {} retries",
                                    attempt + 1,
                                    attempt
                                );
                            }
                            return Ok(response);
                        }
                        Err(e) => {
                            last_error = Some(e);
                        }
                    },
                    Err(e) => {
                        last_error = Some(e);
                    }
                }

                // Check if we should retry this error
                if attempt < config.max_attempts - 1 {
                    if let Some(ref error) = last_error {
                        if config.should_retry_error(error) {
                            let delay = config.calculate_delay(attempt);
                            tracing::warn!(
                                "Request attempt {} failed: {}, retrying after {:?}",
                                attempt + 1,
                                error,
                                delay
                            );
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                    }
                }

                // Don't retry
                break;
            }

            Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Request failed")))
        } else {
            // No retry - single attempt
            self.transport.send(request).await?;
            self.transport.recv().await
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
            return Err(anyhow::anyhow!(
                "Cannot initialize: transport is disconnected"
            ));
        }

        self.state = ClientState::Initializing;
        tracing::info!("Initializing MCP connection...");

        // Prepare initialize parameters
        let client_info = ClientInfo {
            name: "ironclaw-orchestrator".to_string(),
            version: env!("CARGO_PKG_VERSION", "0.1.0").to_string(),
        };

        let capabilities = ClientCapabilities {
            sampling: None,
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

        // Send request and receive response (with optional retry)
        let response = self
            .send_request(&request)
            .await
            .context("Failed to complete initialize request")?;

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
            self.server_capabilities
                .as_ref()
                .map(|c| c.server_info.name.as_str())
                .unwrap_or("unknown"),
            self.server_capabilities
                .as_ref()
                .map(|c| c.protocol_version.as_str())
                .unwrap_or("unknown")
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

        // Send request and receive response (with optional retry)
        let response = self
            .send_request(&request)
            .await
            .context("Failed to complete tools/list request")?;

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
    pub async fn call_tool(
        &mut self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
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

        // Send request and receive response (with optional retry)
        let response = self
            .send_request(&request)
            .await
            .context("Failed to complete tools/call request")?;

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
            ClientState::Created => Err(anyhow::anyhow!(
                "Client not initialized. Call initialize() first."
            )),
            ClientState::Initializing => Err(anyhow::anyhow!("Client is currently initializing")),
            ClientState::Ready => Ok(()),
            ClientState::Disconnected => Err(anyhow::anyhow!("Client is disconnected")),
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
mod tests;
