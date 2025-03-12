# MCP-Rig

MCP-Rig is a Rust crate that provides seamless integration between the Model Context Protocol (MCP) and Rig's LLM agent framework. This crate allows you to expose MCP tools to Rig agents, creating a bridge between these two powerful systems.

## Overview

The integration works by adapting MCP tools to implement Rig's `Tool` and `ToolEmbedding` traits, making them directly usable within Rig's agent framework. This allows you to:

- Expose existing MCP tools to LLM agents
- Use tools with different transport mechanisms (stdio, SSE)
- Enable semantic retrieval of tools based on natural language queries
- Manage multiple MCP clients in a single application

## Features

- **MCP Client Management**: Easily create and manage multiple MCP clients
- **Tool Adaptation**: Automatic adaptation of MCP tools to Rig's Tool interface
- **Transport Options**: Support for both stdio and SSE-based transports
- **Error Handling**: Comprehensive error handling for MCP and Rig operations
- **RAG Compatibility**: Tools can be used with Retrieval-Augmented Generation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
mcp-rig = "0.1.0"
rig-core = "0.9.1"
tokio = { version = "1.32", features = ["full"] }
```

## Quick Start

Here's a simple example of how to use MCP-Rig to create a filesystem tool agent:

```rust
use mcp_client::client::ClientInfo;
use mcp_rig::{setup_rig_with_mcp, McpConnectionManager};
use rig::providers::openai::Client as RigClient;
use std::collections::HashMap;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the MCP connection manager
    let mut connection_manager = McpConnectionManager::with_timeout(Duration::from_secs(30));

    // Get current directory for filesystem access
    let current_dir = env::current_dir()?;
    let current_dir_str = current_dir.to_string_lossy().to_string();

    // Add a filesystem client
    connection_manager
        .add_stdio_client(
            "filesystem-client".to_string(),
            "npx",
            vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-filesystem".to_string(),
                current_dir_str,
            ],
            HashMap::new(),
            ClientInfo {
                name: "rig-integration-filesystem".to_string(),
                version: "1.0.0".to_string(),
            },
        )
        .await?;

    // Initialize the Rig client
    let rig_client = RigClient::from_env();

    // Create the agent builder and model
    let agent_builder = rig_client.agent("gpt-4o").preamble(
        "You are a helpful assistant with access to filesystem tools."
    );
    let model = rig_client.completion_model("o3-mini");

    // Get the filesystem client
    let filesystem_client = connection_manager
        .get_client("filesystem-client")
        .ok_or("Filesystem client not found")?;

    // Set up the Rig agent with MCP tools
    let agent = setup_rig_with_mcp(filesystem_client, agent_builder, model).await?;

    // Use cli_chatbot to handle the interaction
    rig::cli_chatbot::cli_chatbot(agent).await?;

    Ok(())
}
```

## Advanced Usage

For more advanced usage, see the `bin/advanced_filesystem_example.rs` example, which demonstrates:

- Setting up a filesystem MCP client
- Configuring a Rig agent with MCP tools
- Using the agent to interact with the filesystem
- File creation and manipulation through the agent

## MCP Connection Manager

The `McpConnectionManager` provides a simplified interface for working with multiple MCP clients:

```rust
// Create a connection manager
let mut manager = McpConnectionManager::new();

// Add a Git client using stdio transport
manager.add_stdio_client(
    "git-client".to_string(),
    "uvx",
    vec!["mcp-server-git".to_string()],
    HashMap::new(),
    ClientInfo {
        name: "integration-git".to_string(),
        version: "1.0.0".to_string(),
    },
).await?;

// Add an Echo client using SSE transport
manager.add_sse_client(
    "echo-client".to_string(),
    "http://localhost:8000/sse",
    HashMap::new(),
    ClientInfo {
        name: "integration-echo".to_string(),
        version: "1.0.0".to_string(),
    },
).await?;

// Retrieve clients by ID
let git_client = manager.get_client("git-client");
let echo_client = manager.get_client("echo-client");
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
