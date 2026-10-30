//! zwipe-tui: ZWIPE in the terminal.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    zwipe_tui::run().await
}
