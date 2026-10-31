//! zwiper-tui: ZWIPE in the terminal.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    zwiper_tui::run().await
}
