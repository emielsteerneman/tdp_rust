use turbomcp::prelude::*;

#[derive(Clone)]
struct HelloServer;

#[server(name = "hello", version = "1.0.0")]
impl HelloServer {
    #[tool("Say hello to someone")]
    async fn hello(&self, name: String) -> McpResult<String> {
        Ok(format!("Hello, {}!", name))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    HelloServer.().await?;
    Ok(())
}

// That's it! Just 25 lines for a complete MCP server.
// JSON schemas automatically generated from function signatures.
