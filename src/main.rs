// src/main.rs
//
// This file contains an example application that demonstrates
// the integration of MCP clients with Rig tools.

use mcp_client::client::ClientInfo;
use mcp_rig_integration::{setup_rig_with_mcp_rag, McpConnectionManager, McpRigIntegrationError};
use rig_core::RigClient;
use std::collections::HashMap;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("mcp_client=debug".parse().unwrap())
                .add_directive("rig_core=debug".parse().unwrap())
                .add_directive("mcp_rig_integration=debug".parse().unwrap()),
        )
        .init();

    // Create and configure the MCP connection manager
    let mut connection_manager = McpConnectionManager::with_timeout(Duration::from_secs(30));

    // Add a Git client using StdioTransport
    connection_manager
        .add_stdio_client(
            "git-client".to_string(),
            "uvx",
            vec!["mcp-server-git".to_string()],
            HashMap::new(),
            ClientInfo {
                name: "rig-integration-git".to_string(),
                version: "1.0.0".to_string(),
            },
        )
        .await?;

    // Add an Echo client using SseTransport
    connection_manager
        .add_sse_client(
            "echo-client".to_string(),
            "http://localhost:8000/sse",
            HashMap::new(),
            ClientInfo {
                name: "rig-integration-echo".to_string(),
                version: "1.0.0".to_string(),
            },
        )
        .await?;

    // Initialize the Rig client
    let rig_client = RigClient::new()?;

    // Create a Rig agent with MCP tools from the git client
    let git_client = connection_manager
        .get_client("git-client")
        .ok_or("Git client not found")?;

    let agent = setup_rig_with_mcp_rag(
        git_client,
        &rig_client,
        "gpt-4-turbo",             // Model
        "text-embedding-ada-002",  // Embedding model
        "You are a helpful assistant with access to Git tools. You can help users manage their repositories.",
        5,                         // Max dynamic tools
    ).await?;

    // Run a conversation with the agent
    println!("Starting conversation with the agent...");

    let response = agent
        .chat("Can you check the git status of the current repository?")
        .await?;
    println!("Agent response: {}", response);

    let response = agent
        .chat("What branches are available and which one am I on?")
        .await?;
    println!("Agent response: {}", response);

    // Now try with the echo client
    let echo_client = connection_manager
        .get_client("echo-client")
        .ok_or("Echo client not found")?;

    let echo_agent = setup_rig_with_mcp_rag(
        echo_client,
        &rig_client,
        "gpt-4-turbo",             // Model
        "text-embedding-ada-002",  // Embedding model
        "You are a helpful assistant with access to Echo tools. You can help users test message passing.",
        3,                         // Max dynamic tools
    ).await?;

    // Run a conversation with the echo agent
    let response = echo_agent
        .chat("Can you echo a message back to me?")
        .await?;
    println!("Echo agent response: {}", response);

    Ok(())
}
