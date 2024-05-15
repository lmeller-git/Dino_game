use app::App;
use color_eyre::{
    Result
};

pub mod errors;
pub mod tui;
pub mod utils;
pub mod app;

fn main() -> Result<()> {
    errors::install_hooks()?;
    let mut terminal = tui::init()?;
    App::new()
    .run(&mut terminal)?;
    tui::restore()?;
    Ok(())
}
