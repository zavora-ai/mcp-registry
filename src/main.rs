mod server;
mod store;
mod types;

use rmcp::{ServiceExt, transport::stdio};
use server::RegistryServer;
use store::RegistryStore;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = Arc::new(RegistryStore::new());
    let server = RegistryServer { store };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
