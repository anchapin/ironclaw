//! MCP Transport Layer
//!
//! This module defines the transport abstraction for communicating with MCP servers.
//! Multiple transports are supported:
//!
//! - **stdio**: Standard input/output (for local MCP servers)
//! - **HTTP**: HTTP/HTTPS (for remote MCP servers) - TODO: Phase 2
//!
//! # Architecture
//!
//! The transport layer is responsible only for sending and receiving messages.
//! Protocol concerns (JSON-RPC formatting) are handled in the protocol layer.

use crate::mcp::protocol::{McpRequest, McpResponse};
use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

/// Transport trait for MCP communication
///
/// All transports must implement this trait, enabling the client
/// to work with different transport mechanisms (stdio, HTTP, etc).
#[allow(async_fn_in_trait)]
pub trait Transport: Send + Sync {
    /// Send a request to the MCP server
    ///
    /// # Arguments
    ///
    /// * `request` - The MCP request to send
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the request was sent successfully
    async fn send(&mut self, request: &McpRequest) -> Result<()>;

    /// Receive a response from the MCP server
    ///
    /// # Returns
    ///
    /// Returns the MCP response, or an error if communication fails
    async fn recv(&mut self) -> Result<McpResponse>;

    /// Check if the transport is still connected
    fn is_connected(&self) -> bool;
}

/// stdio transport for local MCP servers
///
/// This transport spawns an MCP server as a child process and communicates
/// with it via stdin/stdout. Each line is a JSON-RPC message.
///
/// # Example
///
/// ```ignore
/// let transport = StdioTransport::spawn("npx", &["-y", "@modelcontextprotocol/server-filesystem"]);
/// transport.send(&request).await?;
/// let response = transport.recv().await?;
/// ```
pub struct StdioTransport {
    /// Child process handle
    child: Option<Child>,

    /// stdin handle for sending requests
    stdin: ChildStdin,

    /// stdout handle for receiving responses
    stdout: BufReader<ChildStdout>,

    /// Server command (for diagnostics)
    command: String,

    /// Whether the transport is still connected
    connected: bool,

    /// Reusable buffer for reading lines
    line_buffer: String,

    /// Reusable buffer for serializing requests
    write_buffer: Vec<u8>,
}

impl StdioTransport {
    /// Spawn a new MCP server process and create a stdio transport
    ///
    /// # Arguments
    ///
    /// * `command` - The command to spawn (e.g., "npx", "python", "./server")
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    ///
    /// Returns a new `StdioTransport` instance
    ///
    /// # Example
    ///
    /// ```ignore
    /// let transport = StdioTransport::spawn(
    ///     "npx",
    ///     &["-y", "@modelcontextprotocol/server-filesystem", "/path/to/files"]
    /// ).await?;
    /// ```
    pub async fn spawn(command: &str, args: &[&str]) -> Result<Self> {
        tracing::info!("Spawning MCP server: {}", command);
        tracing::debug!("Server arguments: {:?}", args);

        // Spawn the child process with piped stdin/stdout
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Inherit stderr so we can see server logs
            .spawn()
            .context("Failed to spawn MCP server process")?;

        // Get the stdin and stdout handles
        let stdin = child.stdin.take().context("Failed to get child stdin")?;
        let stdout = child.stdout.take().context("Failed to get child stdout")?;

        Ok(Self {
            child: Some(child),
            stdin,
            stdout: BufReader::new(stdout),
            command: format!("{} {}", command, args.join(" ")),
            connected: true,
            line_buffer: String::with_capacity(4096),
            write_buffer: Vec::with_capacity(4096),
        })
    }

    /// Get the server command string (for diagnostics)
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Kill the MCP server process
    ///
    /// This sends a SIGTERM signal to the child process and waits for it to exit.
    pub async fn kill(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            tracing::info!("Killing MCP server: {}", self.command);
            child
                .kill()
                .await
                .context("Failed to kill MCP server process")?;
            self.connected = false;
        }
        Ok(())
    }

    /// Wait for the MCP server process to exit
    ///
    /// This waits for the child process to exit naturally and returns the exit code.
    pub async fn wait(&mut self) -> Result<Option<i32>> {
        if let Some(mut child) = self.child.take() {
            let status = child
                .wait()
                .await
                .context("Failed to wait for MCP server process")?;
            self.connected = false;
            Ok(status.code())
        } else {
            Ok(None)
        }
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        // Try to kill the child process when the transport is dropped
        if let Some(mut child) = self.child.take() {
            tracing::debug!("Dropping StdioTransport, killing MCP server");
            // Note: We can't await in Drop, so we just start the kill
            let _ = child.start_kill();
        }
    }
}

impl Transport for StdioTransport {
    /// Send a JSON-RPC request to the MCP server via stdin
    ///
    /// The request is serialized to JSON and written as a single line to stdin.
    async fn send(&mut self, request: &McpRequest) -> Result<()> {
        if !self.connected {
            return Err(anyhow::anyhow!("Transport is not connected"));
        }

        // Clear buffer for reuse to avoid allocation
        self.write_buffer.clear();

        // Serialize the request to JSON directly into the buffer
        serde_json::to_writer(&mut self.write_buffer, request)
            .context("Failed to serialize MCP request to JSON")?;

        // Append newline (JSON-RPC uses line-based protocol)
        self.write_buffer.push(b'\n');

        // Log the message if debug logging is enabled
        // We do a lossy conversion here which is cheap enough for debug logging
        if tracing::enabled!(tracing::Level::DEBUG) {
            let json_str = String::from_utf8_lossy(&self.write_buffer);
            tracing::debug!("Sending to MCP server: {}", json_str.trim());
        }

        // Write the buffer to stdin in a single call
        self.stdin
            .write_all(&self.write_buffer)
            .await
            .context("Failed to write to MCP server stdin")?;

        // Flush to ensure the message is sent immediately
        self.stdin
            .flush()
            .await
            .context("Failed to flush MCP server stdin")?;

        Ok(())
    }

    /// Receive a JSON-RPC response from the MCP server via stdout
    ///
    /// Reads a single line from stdout and deserializes it as a McpResponse.
    async fn recv(&mut self) -> Result<McpResponse> {
        if !self.connected {
            return Err(anyhow::anyhow!("Transport is not connected"));
        }

        // Clear buffer for reuse to avoid allocation
        self.line_buffer.clear();

        // Read a line from stdout
        let bytes_read = self
            .stdout
            .read_line(&mut self.line_buffer)
            .await
            .context("Failed to read from MCP server stdout")?;

        // Check for EOF
        if bytes_read == 0 {
            self.connected = false;
            return Err(anyhow::anyhow!("MCP server closed connection (EOF)"));
        }

        tracing::debug!("Received from MCP server: {}", self.line_buffer.trim());

        // Deserialize the JSON line
        let response: McpResponse = serde_json::from_str(&self.line_buffer).with_context(|| {
            format!(
                "Failed to deserialize MCP response from JSON: {}",
                self.line_buffer
            )
        })?;

        Ok(response)
    }

    /// Check if the transport is still connected
    fn is_connected(&self) -> bool {
        self.connected && self.child.is_some()
    }
}

#[cfg(test)]
mod tests;
