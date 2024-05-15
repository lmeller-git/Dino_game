use crate::tui;
use crate::errors;

use color_eyre::owo_colors::OwoColorize;
use crossterm::event::poll;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use num::ToPrimitive;
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::canvas::MapResolution;
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, canvas, *},
    widgets::Paragraph,
    style::Color,
};
use std::collections::hash_map::Keys;
use std::time::Duration;
use std::{io, sync::Arc};

use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};

use rand::prelude::*;

#[derive(Debug, Default)]
pub struct App {
    score: u64,
    exit: bool,
    y: f64,
    in_air: bool,
    rising: bool,
    ducking: bool,
    height: f64,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        let title = Title::from(" Dinosaur Game ".bold());
        let instructions = Title::from(Line::from(vec![
            " Jump ".into(),
            "<Up> ".bold(),
            " Quit ".into(),
            "<Q> ".bold(),
            " Duck ".into(),
            "<Down> ".bold()
        ]));

        let block = Block::default()
                    .title(title.alignment(Alignment::Center)
                        .position(Position::Top))
                    .title(instructions
                        .alignment(Alignment::Center)
                        .position(Position::Bottom))
                    .borders(Borders::ALL);
        
        let counter_text = Text::from(vec![Line::from(vec![
            "Score ".into(),
            self.score.to_string().into(),
        ])]);

        Paragraph::new(counter_text)
        .block(block.clone())
        .right_aligned()
        .render(area, buf);

        let player = Canvas::default()
            .block(block)
            .x_bounds([-90.0, 90.0])
            .y_bounds([-45.0, 45.0])
            .paint(|ctx|{
                ctx.draw(&canvas::Rectangle {
                    x: (area.width / 128).to_f64().unwrap(),
                    y: self.y,
                    width: 10.0,
                    height: self.height,
                    color: Color::White,
                });
            });
        player.render(area, buf); 
            
    }   
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render_frame(frame))?;
            if event::poll(Duration::from_millis(5))? {
                self.handle_events().wrap_err("handle events failed")?;
            }
            self.update_position()?;
            if self.exit {
                break;
            }
            self.score += 1;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event).wrap_err_with(|| {
                    format!("handling key event failed: \n{key_event:#?}")
                })
            }
           _ => Ok(())
        }
    }

    fn update_position(&mut self) -> Result<()> {
        if self.in_air {
            if self.rising {
                if self.y < 10.0 {
                    self.y += 1.0;
                }
                else {
                    self.rising = false;
                }
            }
            else {
                if self.y > -20.0 {
                    self.y -= 1.0;
                }
                else {
                    self.in_air = false;
                }
            }
        }
        if self.ducking {
            self.height = 5.0;
        }
        else {
            self.height = 10.0;
        }
        Ok(())
    }

    pub fn new() -> App {
        App {
            score: 0,
            exit: false, 
            y: -20.0,
            in_air: false,
            rising: false,
            ducking: false,
            height: 10.0
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement()?,
            KeyCode::Right => self.increment()?,
            KeyCode::Down => self.duck()?,
            KeyCode::Up => self.jump()?,
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn decrement(&mut self) -> Result<()> {
        self.score -= 1;
        Ok(())
    }

    fn increment(&mut self) -> Result<()> {
        self.score += 1;
        Ok(())
    }

    fn jump(&mut self) -> Result<()> {
        if self.in_air {
            self.in_air;
        }
        else {
            self.in_air = true;
            self.rising = true;
            self.ducking = false;
        }
        Ok(())
    }

    fn duck(&mut self) -> Result<()> {
        if self.ducking {
            self.ducking = false;
        }
        else {
            self.ducking = true;
        }
        Ok(())
    }

}

#[derive(Debug, Default)]
pub struct Player {
    x: u8,
    y: u8,
    in_air: bool
}

impl Widget for &Player {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
                let block = Block::bordered();
    }
}

impl Player {

    fn handle_events(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        Ok(())
    }

    fn jump(&mut self) -> Result<()> {
        Ok(())
    }

    fn duck(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Cactus {
    x: u8
}

impl Widget for &Cactus {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        
    }
}

#[derive(Debug, Default)]
pub struct Bird {
    x: u8,
    y: u8
}

impl Widget for &Bird {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
        
    }
}