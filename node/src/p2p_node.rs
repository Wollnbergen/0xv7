use sultan_coordinator::P2PNode;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut node = P2PNode::new();
    node.run().await
}
