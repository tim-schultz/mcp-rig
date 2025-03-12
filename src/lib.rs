//! # MCP-Rig Integration
//!
//! This crate provides seamless integration between MCP (Machine Comprehension Protocol) clients
//! and Rig's LLM agent framework. It allows you to expose MCP tools to Rig agents, creating a
//! bridge between these two powerful systems.
//!
//! ## Overview
//!
//! The integration works by adapting MCP tools to implement Rig's `Tool` and `ToolEmbedding` traits,
//! making them directly usable within Rig's agent framework. This allows you to:
//!
//! - Expose existing MCP tools to LLM agents
//! - Use tools with different transport mechanisms (stdio, SSE)
//! - Enable semantic retrieval of tools based on natural language queries
//! - Manage multiple MCP clients in a single application

use rig::{agent::Agent, completion::CompletionModel, providers::openai::Client as RigClient};

mod adapter;
mod connection;
mod error;
mod toolset;

pub use adapter::{McpToolAdapter, McpToolArgs, McpToolState};
pub use connection::McpConnectionManager;
pub use error::McpRigIntegrationError;
pub use toolset::{create_mcp_toolset, register_mcp_tools};

// Re-export relevant dependencies for ease of use
pub use mcp_client;

// High-level integration function that sets up a Rig agent with MCP tools
pub async fn setup_rig_with_mcp(
    mcp_client: std::sync::Arc<Box<dyn mcp_client::McpClientTrait>>,
    rig_client: &RigClient,
    model: &str,
    preamble: &str,
) -> Result<Agent<impl CompletionModel>, error::McpRigIntegrationError> {
    // Create the model and agent builder
    let mut agent_builder = rig_client.agent(model).preamble(preamble);
    let model = rig_client.completion_model(model);
    register_mcp_tools(
        std::sync::Arc::clone(&mcp_client),
        &mut agent_builder,
        model,
    )
    .await?;

    // Build the agent
    let agent = agent_builder.build();

    Ok(agent)
}

// /// Variant that also adds dynamic RAG-enabled tools
// pub async fn setup_rig_with_mcp_rag(
//     mcp_client: std::sync::Arc<Box<dyn mcp_client::McpClientTrait>>,
//     rig_client: &RigClient,
//     model: &str,
//     embedding_model: &str,
//     preamble: &str,
//     max_dynamic_tools: usize,
// ) -> Result<Agent<impl CompletionModel>, error::McpRigIntegrationError> {
//     // Create the model and agent builder
//     let mut agent_builder = rig_client.agent(model).preamble(preamble);
//     let model = rig_client.completion_model(model);

//     // For RAG-enabled dynamic tools
//     let toolset = create_mcp_toolset(std::sync::Arc::clone(&mcp_client)).await?;

//     // Create embedding store for tool embeddings
//     let embedding_model = rig_client.embedding_model(embedding_model);
//     let mut index = EmbeddingStore::new(embedding_model);

//     // TODO: ToolSet doesn't expose a way to iterate over tools yet
//     // We'll need to add tools directly with the builder for now
//     register_mcp_tools(
//         std::sync::Arc::clone(&mcp_client),
//         &mut agent_builder,
//         model,
//     )
//     .await?;

//     // Add dynamic tool retrieval to the agent
//     let agent = agent_builder.build();

//     Ok(agent)
// }
