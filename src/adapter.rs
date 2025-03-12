// src/adapter.rs

//! Adapter implementation that bridges MCP tools to Rig's Tool trait.
//!
//! This module contains the core adapter that allows MCP tools to be used
//! with Rig's agent framework. It provides implementations of Rig's `Tool`
//! and `ToolEmbedding` traits for MCP tools.

use crate::error::McpRigIntegrationError;
use mcp_client::McpClientTrait;
use rig::{
    completion::ToolDefinition,
    tool::{Tool, ToolEmbedding},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Adapter that wraps an MCP tool and implements the Rig Tool trait.
///
/// This adapter serves as the bridge between MCP and Rig, allowing MCP tools
/// to be used directly within Rig's agent framework. It handles the conversion
/// between the two systems' tool interfaces, including:
///
/// - Tool definitions and parameters
/// - Tool execution and result handling
/// - Error conversion
/// - Semantic embedding for RAG
#[derive(Clone)]
pub struct McpToolAdapter {
    /// The MCP client used to execute the tool
    mcp_client: Arc<Box<dyn McpClientTrait>>,
    /// The name of the MCP tool
    tool_name: String,
    /// The description of the MCP tool
    tool_description: String,
    /// The JSON Schema parameters of the MCP tool
    parameters: Value,
}

impl McpToolAdapter {
    /// Create a new McpToolAdapter.
    ///
    /// This constructor creates a new adapter that wraps an MCP tool and makes it
    /// compatible with Rig's agent framework.
    ///
    /// # Parameters
    ///
    /// - `mcp_client`: The MCP client used to execute the tool
    /// - `tool_name`: The name of the MCP tool
    /// - `tool_description`: A description of what the tool does
    /// - `parameters`: A JSON Schema definition of the tool's parameters
    ///
    /// # Returns
    ///
    /// A new `McpToolAdapter` instance
    pub fn new(
        mcp_client: Arc<Box<dyn McpClientTrait>>,
        tool_name: String,
        tool_description: String,
        parameters: Value,
    ) -> Self {
        Self {
            mcp_client,
            tool_name,
            tool_description,
            parameters,
        }
    }
}

/// Arguments for an MCP tool call.
///
/// This structure represents the arguments passed to an MCP tool. Since MCP tools
/// can have arbitrary parameter structures, it uses a flattened JSON Value to
/// capture all possible argument combinations.
///
/// The `#[serde(flatten)]` attribute ensures that the JSON properties are directly
/// included in this struct rather than nested under an "args" field.
#[derive(Deserialize)]
pub struct McpToolArgs {
    /// Dynamic arguments structure based on the tool parameters.
    /// This contains all the parameters passed to the tool.
    #[serde(flatten)]
    pub args: Value,
}

/// State required to reconstruct an MCP tool adapter.
///
/// This structure contains all the necessary information to recreate an MCP tool adapter.
/// It's used with the `ToolEmbedding` trait to support RAG-enabled tools.
#[derive(Clone, Serialize, Deserialize)]
pub struct McpToolState {
    /// Name of the tool
    pub name: String,
    /// Description of the tool
    pub description: String,
    /// JSON Schema for the tool parameters
    pub parameters: Value,
}

// Type alias for ClientId to use as Context
// This is serializable, unlike Arc<Box<dyn McpClientTrait>>
#[derive(Clone, Serialize, Deserialize)]
pub struct ClientId(pub String);

/// Implementation of Rig's `Tool` trait for MCP tools.
impl Tool for McpToolAdapter {
    const NAME: &'static str = "mcp_tool";

    /// The error type returned by this tool
    type Error = McpRigIntegrationError;

    /// The argument type accepted by this tool
    type Args = McpToolArgs;

    /// The output type returned by this tool
    type Output = Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        // The key is that we use the actual MCP tool name here
        ToolDefinition {
            name: self.tool_name.clone(), // This is the MCP tool name
            description: self.tool_description.clone(),
            parameters: self.parameters.clone(),
        }
    }

    // Override the name() method to return the dynamic tool name
    fn name(&self) -> String {
        self.tool_name.clone()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Clone everything we need to move into the async block
        let mcp_client = Arc::clone(&self.mcp_client);
        let tool_name = self.tool_name.clone();
        let args_value = args.args.clone();

        // Use tokio::spawn but ensure the future is properly constructed
        // We'll use a pattern that ensures the future is properly constrained
        let result = tokio::task::spawn(async move {
            // This code runs in a separate task
            match mcp_client.call_tool(&tool_name, args_value).await {
                Ok(result) => Ok(result),
                Err(e) => Err(McpRigIntegrationError::McpError(e.to_string())),
            }
        })
        .await // Wait for the spawned task to complete
        .map_err(|e| McpRigIntegrationError::McpError(format!("Task join error: {}", e)))?; // Handle JoinError

        // Now we've properly awaited the spawned task and have our result
        // The `result` here is already a Result<CallToolResult, McpRigIntegrationError>
        let tool_result = result?;

        // Handle errors from the tool execution
        if tool_result.is_error.unwrap_or(false) {
            return Err(McpRigIntegrationError::ToolExecutionError(format!(
                "MCP tool error: {:?}",
                tool_result.content
            )));
        }

        Ok(serde_json::to_value(tool_result.content)?)
    }
}

/// Implementation of Rig's `ToolEmbedding` trait for MCP tools.
impl ToolEmbedding for McpToolAdapter {
    type InitError = McpRigIntegrationError;
    type Context = ClientId;
    type State = McpToolState;
    /// Initializes a new tool instance from state and context.
    fn init(_state: Self::State, _context: Self::Context) -> Result<Self, Self::InitError> {
        // In a real implementation, you would use the ClientId to look up the actual client
        // from a registry or manager. This is a simplified example.
        Err(McpRigIntegrationError::InitError(
            "ClientId-based initialization not implemented".to_string(),
        ))
    }

    /// Provides text documents for embedding this tool in a vector database.
    fn embedding_docs(&self) -> Vec<String> {
        // Create multiple documents to enhance semantic retrieval
        vec![
            // The main description
            self.tool_description.clone(),
            // Include the tool name to help with direct name references
            format!("Tool name: {}", self.tool_name),
            // Restate the capability to give the embedding more context
            format!("Tool capability: {}", self.tool_description),
        ]
    }

    /// Provides the context needed to recreate this tool.
    fn context(&self) -> Self::Context {
        // In a real implementation, this would return the client ID
        // that can be used to look up the actual client.
        ClientId("client-id".to_string())
    }
}
