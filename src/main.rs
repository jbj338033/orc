use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = orc_tui::terminal::init()?;
    let mut app = orc_tui::App::new()?;

    let result = app.run(&mut terminal).await;

    orc_tui::terminal::restore()?;

    result
}
