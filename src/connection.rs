// src/connection.rs

//! Connection manager for MCP clients.
//!
//! This module provides a connection manager for MCP clients with different transport
//! mechanisms. It simplifies the creation, storage, and retrieval of MCP clients,
//! supporting various transport options such as stdio and SSE.

use crate::error::McpRigIntegrationError;
use mcp_client::{
    client::{ClientCapabilities, ClientInfo, McpClient, McpClientTrait},
    transport::{SseTransport, StdioTransport, Transport},
    McpService,
};
use std::{collections::HashMap, sync::Arc, time::Duration};

/// Manager for MCP client connections.
///
/// The `McpConnectionManager` simplifies working with multiple MCP clients by:
///
/// 1. Providing a unified interface for different transport mechanisms
/// 2. Handling client initialization and setup
/// 3. Storing clients for later retrieval by ID
/// 4. Managing client lifecycle and configuration
///
/// It supports both stdio-based and SSE-based transports, and can be extended
/// to support additional transport types.
///
/// # Example
///
/// ```rust,no_run
/// use mcp_rig::McpConnectionManager;
/// use mcp_client::client::ClientInfo;
/// use std::collections::HashMap;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut manager = McpConnectionManager::new();
///
/// // Add a Git client using stdio transport
/// manager.add_stdio_client(
///     "git-client".to_string(),
///     "uvx",
///     vec!["mcp-server-git".to_string()],
///     HashMap::new(),
///     ClientInfo {
///         name: "rig-integration-git".to_string(),
///         version: "1.0.0".to_string(),
///     },
/// ).await?;
///
/// // Retrieve the client by ID
/// let git_client = manager.get_client("git-client")
///     .ok_or("Git client not found")?;
///
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct McpConnectionManager {
    /// Map of client ID to client instance
    clients: HashMap<String, Arc<Box<dyn McpClientTrait>>>,
    /// Default timeout for MCP services
    timeout: Duration,
}

impl McpConnectionManager {
    /// Create a new connection manager with default timeout of 30 seconds.
    ///
    /// This constructor creates a connection manager with a default timeout
    /// suitable for most MCP client operations.
    ///
    /// # Returns
    ///
    /// A new `McpConnectionManager` instance with default settings
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Create a new connection manager with specified timeout.
    ///
    /// This constructor allows customizing the timeout used for MCP service operations.
    /// Use this when you need different timeout behavior than the default.
    ///
    /// # Parameters
    ///
    /// - `timeout`: The duration to use for timeouts in MCP service operations
    ///
    /// # Returns
    ///
    /// A new `McpConnectionManager` instance with the specified timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            clients: HashMap::new(),
            timeout,
        }
    }

    /// Add a client using a StdioTransport
    pub async fn add_stdio_client(
        &mut self,
        id: String,
        program: &str,
        args: Vec<String>,
        env: HashMap<String, String>,
        client_info: ClientInfo,
    ) -> Result<(), McpRigIntegrationError> {
        let transport = StdioTransport::new(program, args, env);
        self.add_client(id, transport, client_info).await
    }

    /// Add a client using an SseTransport
    pub async fn add_sse_client(
        &mut self,
        id: String,
        url: &str,
        headers: HashMap<String, String>,
        client_info: ClientInfo,
    ) -> Result<(), McpRigIntegrationError> {
        let transport = SseTransport::new(url, headers);
        self.add_client(id, transport, client_info).await
    }

    /// Generic method to add a client with any transport
    pub async fn add_client(
        &mut self,
        id: String,
        transport: impl Transport,
        client_info: ClientInfo,
    ) -> Result<(), McpRigIntegrationError> {
        let handle = transport
            .start()
            .await
            .map_err(|e| McpRigIntegrationError::McpError(e.to_string()))?;

        let service = McpService::with_timeout(handle, self.timeout);
        let mut client = McpClient::new(service);

        // Initialize the client
        let capabilities = ClientCapabilities::default();
        client
            .initialize(client_info, capabilities)
            .await
            .map_err(|e| McpRigIntegrationError::McpError(e.to_string()))?;

        self.clients.insert(id, Arc::new(Box::new(client)));
        Ok(())
    }

    /// Get a client by ID
    pub fn get_client(&self, id: &str) -> Option<Arc<Box<dyn McpClientTrait>>> {
        self.clients.get(id).cloned()
    }

    /// Remove a client by ID
    pub fn remove_client(&mut self, id: &str) -> bool {
        self.clients.remove(id).is_some()
    }

    /// Check if a client exists
    pub fn has_client(&self, id: &str) -> bool {
        self.clients.contains_key(id)
    }

    /// Get all client IDs
    pub fn client_ids(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }

    /// Get the number of clients
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
}
