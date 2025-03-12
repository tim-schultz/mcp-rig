// advanced_filesystem_example.rs
//
// This example demonstrates a more advanced integration with the filesystem MCP server,
// using cli_chatbot to manage the message loop.

use mcp_client::client::ClientInfo;
use mcp_rig::{setup_rig_with_mcp, McpConnectionManager};
use rig::providers::openai::Client as RigClient;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::io::{self, AsyncBufReadExt};
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

    println!("Starting Advanced MCP-Rig Filesystem Example");

    // Get the current directory
    let current_dir = env::current_dir()?;
    let current_dir_str = current_dir.to_string_lossy().to_string();
    println!("Current directory: {}", current_dir_str);

    // Create a test file for demonstrations
    let test_file_path = current_dir.join("test_file.txt");
    if !test_file_path.exists() {
        println!("Creating a test file for demonstration purposes...");
        fs::write(
            &test_file_path,
            "This is a test file.\nIt contains multiple lines.\nWe will use it to demonstrate file operations.\n\nFeel free to modify this content.\n",
        )?;
        println!("Created test file at: {}", test_file_path.display());
    }

    // Create the MCP connection manager
    let mut connection_manager = McpConnectionManager::with_timeout(Duration::from_secs(30));

    // Add a filesystem client
    println!(
        "Setting up filesystem client with access to: {}",
        current_dir_str
    );

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

    println!("Filesystem client connected successfully");

    // Initialize the Rig client
    let rig_client = RigClient::from_env();

    // Create the model and agent builder
    let agent_builder = rig_client.agent("gpt-4o").preamble("You are a helpful assistant with access to filesystem tools. You can help users explore and manipulate files and directories.

When editing files, first use read_file to show the current content, then use edit_file with dryRun:true to preview changes before applying them.

For file paths, use relative paths from the current working directory. For example, 'test_file.txt' not '/path/to/test_file.txt'.

Always be cautious when modifying files and confirm important operations with the user.");
    let model = rig_client.completion_model("o3-mini");
    println!("Rig client initialized");

    // Get the filesystem client
    let filesystem_client = connection_manager
        .get_client("filesystem-client")
        .ok_or("Filesystem client not found")?;

    // Set up the Rig agent with detailed instructions
    println!("Setting up Rig agent with filesystem MCP tools");
    let agent = setup_rig_with_mcp(filesystem_client, agent_builder, model).await?;

    println!("Rig agent setup complete. Starting interactive session...");
    println!("\n--- Interactive Session ---\n");
    println!("Type your questions or commands about the filesystem. Type 'exit' to quit.");

    // Use cli_chatbot to manage the message loop
    rig::cli_chatbot::cli_chatbot(agent).await?;

    println!("\n--- Session ended ---");

    // Clean up the test file if the user wants
    println!("Would you like to delete the test file? (y/n)");
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin).lines();

    let response = match reader.next_line().await {
        Ok(Some(line)) => line,
        _ => String::from("n"),
    };

    if response.trim().to_lowercase() == "y" {
        if Path::new(&test_file_path).exists() {
            fs::remove_file(&test_file_path)?;
            println!("Test file deleted.");
        }
    } else {
        println!("Test file kept at: {}", test_file_path.display());
    }

    println!("Example completed successfully!");
    Ok(())
}
