use app::App;
use color_eyre::Result;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::io;

pub mod errors;
pub mod tui;
pub mod utils;
pub mod app;

fn main() -> Result<()> {
    errors::install_hooks()?;
    let mut terminal = tui::init()?;

    let path = Path::new("Highscore.bin");
    let number: u64;
    if !path.exists() {
        File::create(path)?;
        number = 0;
    }
    else {
        number = read(&path)?;
    }

    let mut app = App::new();
    app.highscore = number;
    app.run(&mut terminal)?;
    tui::restore()?;
    
    save(path, app.highscore)?;
    Ok(())
}

fn save(path: &Path, number: u64) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(&number.to_le_bytes())?;
    Ok(())
}

fn read(path: &Path) -> io::Result<u64> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 8];
    file.read_exact(&mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}