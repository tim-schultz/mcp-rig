// src/toolset.rs

//! Functions for registering MCP tools with Rig and creating toolsets.
//!
//! This module exposes two main functions:
//! - `create_mcp_toolset`: Creates a ToolSet from MCP tools for use with RAG
//! - `register_mcp_tools`: Registers MCP tools with a Rig agent builder
//!
//! This module provides the core functionality for integrating MCP tools with
//! Rig agents. It includes functions for registering tools with agent builders
//! and creating toolsets for RAG-enabled dynamic tool retrieval.

use crate::adapter::{McpToolAdapter, McpToolState};
use crate::error::McpRigIntegrationError;
use mcp_client::McpClientTrait;
use rig::agent::AgentBuilder;
use rig::tool::ToolSet;
use std::sync::Arc;

/// Register all available MCP tools with a Rig agent builder.
///
/// This function queries an MCP client for all available tools and registers
/// them with a Rig agent builder. This makes the tools directly available
/// to the agent for use in its operations.
///
/// # Parameters
///
/// - `mcp_client`: The MCP client to query for tools
/// - `agent_builder`: The agent builder to register tools with
///
/// # Returns
///
/// `Ok(())` if registration was successful, or an error if it failed
pub async fn register_mcp_tools(
    mcp_client: Arc<Box<dyn McpClientTrait>>,
    agent_builder: &mut AgentBuilder,
) -> Result<(), McpRigIntegrationError> {
    // List all available tools from the MCP client
    let tools_list = mcp_client
        .list_tools(None)
        .await
        .map_err(|e| McpRigIntegrationError::McpError(e.to_string()))?;

    // For each tool, create an adapter and register it with the Rig agent
    for tool in tools_list.tools {
        let adapter = McpToolAdapter::new(
            Arc::clone(&mcp_client),
            tool.name,
            tool.description,
            tool.input_schema, // Changed from parameters to input_schema
        );

        agent_builder.tool(adapter);
    }

    Ok(())
}

/// Create a ToolSet from all available MCP tools for use with RAG
pub async fn create_mcp_toolset(
    mcp_client: Arc<Box<dyn McpClientTrait>>,
) -> Result<ToolSet<McpToolAdapter>, McpRigIntegrationError> {
    let mut toolset = ToolSet::new();

    // List all available tools from the MCP client
    let tools_list = mcp_client
        .list_tools(None)
        .await
        .map_err(|e| McpRigIntegrationError::McpError(e.to_string()))?;

    // For each tool, create a state and add it to the toolset
    for tool in tools_list.tools {
        let state = McpToolState {
            name: tool.name,
            description: tool.description,
            parameters: tool.input_schema, // Changed from parameters to input_schema
        };

        toolset
            .add(state, Arc::clone(&mcp_client))
            .map_err(|e| McpRigIntegrationError::InitError(e.to_string()))?;
    }

    Ok(toolset)
}

/// Create a filtered ToolSet with only specific tools from an MCP client
pub async fn create_filtered_mcp_toolset(
    mcp_client: Arc<Box<dyn McpClientTrait>>,
    tool_names: &[String],
) -> Result<ToolSet, McpRigIntegrationError> {
    let mut toolset = ToolSet::new();

    // List all available tools from the MCP client
    let tools_list = mcp_client
        .list_tools(None)
        .await
        .map_err(|e| McpRigIntegrationError::McpError(e.to_string()))?;

    // For each tool that matches our filter, create a state and add it to the toolset
    for tool in tools_list.tools {
        if tool_names.contains(&tool.name) {
            let state = McpToolState {
                name: tool.name,
                description: tool.description,
                parameters: tool.input_schema, // Changed from parameters to input_schema
            };

            toolset
                .add(state, Arc::clone(&mcp_client))
                .map_err(|e| McpRigIntegrationError::InitError(e.to_string()))?;
        }
    }

    Ok(toolset)
}
