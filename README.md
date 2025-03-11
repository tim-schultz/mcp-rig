// src/error.rs

//! Error types for the MCP-Rig integration.
//!
//! This module defines the error types used throughout the crate and handles
//! conversions between MCP and Rig error types. It provides a comprehensive
//! error handling system that propagates errors correctly between the two systems.

use thiserror::Error;

/// Comprehensive error type for the MCP-Rig integration.
///
/// This enum represents all possible errors that can occur during the integration
/// between MCP clients and Rig tools. It provides specific error variants for
/// different categories of errors, making it easier to handle them appropriately.
///
/// The error type implements `std::error::Error` and provides detailed error messages
/// through the `thiserror` macros. #[derive(Debug, Error)]
pub enum McpRigIntegrationError {
/// Errors originating from the MCP client.
///
/// These errors occur when interacting with MCP clients, such as during
/// tool listing, tool execution, or client initialization. #[error("MCP client error: {0}")]
McpError(String),

    /// Errors originating from the Rig core library.
    ///
    /// These errors occur when interacting with Rig components, such as
    /// during agent creation, tool registration, or agent execution.
    #[error("Rig error: {0}")]
    RigError(#[from] rig_core::Error),

    /// Errors that occur during tool execution.
    ///
    /// These errors are specific to the execution of tools and typically
    /// include errors returned by the tools themselves.
    #[error("Tool execution error: {0}")]
    ToolExecutionError(String),

    /// Errors that occur during initialization.
    ///
    /// These errors happen when setting up components, such as during
    /// tool adapter initialization or client setup.
    #[error("Initialization error: {0}")]
    InitError(String),

    /// Errors related to serialization or deserialization.
    ///
    /// These errors occur when working with JSON data, such as when
    /// parsing tool parameters or results.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Any other errors that don't fit into the above categories.
    ///
    /// This is a catch-all for errors that aren't specifically handled
    /// by the other variants.
    #[error("Other error: {0}")]
    Other(Box<dyn std::error::Error + Send +

// Add conversion from standard error types
impl<E: std::error::Error + Send + Sync + 'static> From<E> for McpRigIntegrationError {
fn from(err: E) -> Self {
// Check if it's a type we handle specifically
if let Some(mcp_err) = err.downcast_ref::<mcp_client::Error>() {
return Self::McpError(mcp_err.to_string());
}

        if let Some(rig_err) = err.downcast_ref::<rig_core::Error>() {
            return Self::RigError(rig_err.clone());
        }

        if let Some(json_err) = err.downcast_ref::<serde_json::Error>() {
            return Self::SerializationError(json_err.clone());
        }

        // Otherwise, wrap it as a generic error
        Self::Other(Box::new(err))
    }

}

// Implementing this manually since we can't derive it
impl PartialEq for McpRigIntegrationError {
fn eq(&self, other: &Self) -> bool {
match (self, other) {
(Self::McpError(a), Self::McpError(b)) => a == b,
(Self::RigError(a), Self::RigError(b)) => a == b,
(Self::ToolExecutionError(a), Self::ToolExecutionError(b)) => a == b,
(Self::InitError(a), Self::InitError(b)) => a == b,
(Self::SerializationError(a), Self::SerializationError(b)) => a.to*string() == b.to_string(),
// Other can't be meaningfully compared
* => false,
}
}
}
