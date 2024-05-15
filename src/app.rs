use crate::tui;

use color_eyre::{
    eyre::WrapErr,
    Result,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use num::ToPrimitive;

use ratatui::widgets::canvas::Canvas;
use ratatui::{
    prelude::*,
    widgets::{block::*, canvas, *},
    widgets::Paragraph,
    style::Color,
};

use std::time::Duration;

use rand::prelude::*;

#[derive(Debug, Default)]
pub struct App {
    pub score: u64,
    pub highscore: u64,
    exit: bool,
    y: f64,
    in_air: bool,
    rising: bool,
    ducking: bool,
    height: f64,
    enemies: Vec<f64>,
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

        let best_counter_text = Text::from(vec![Line::from(vec![
            "Highscore ".into(),
            self.highscore.to_string().into(),
        ])]);

        Paragraph::new(counter_text)
        .block(block.clone())
        .right_aligned()
        .render(area, buf);

        Paragraph::new(best_counter_text)
        .block(block.clone())
        .left_aligned()
        .render(area, buf);

        let player = Canvas::default()
            .block(block)
            .x_bounds([-90.0, 90.0])
            .y_bounds([-45.0, 45.0])
            .paint(|ctx|{
                ctx.draw(&canvas::Rectangle {
                    x: -5.0,
                    y: self.y,
                    width: 10.0,
                    height: self.height,
                    color: Color::White,
                });
                ctx.layer();
                ctx.draw(&canvas::Line {
                    x1: -90.0,
                    y1: -20.0,
                    x2: 90.0,
                    y2: -20.0,
                    color: Color::Green,
                });
                ctx.layer();
                if self.enemies.len() > 0 {
                    for enemy in self.enemies.iter(){
                        ctx.draw(&canvas::Rectangle {
                            x: *enemy,
                            y: -20.0,
                            width: 2.0,
                            height: 5.0,
                            color: Color::Red,
                        })
                    }
                }
            });
        player.render(area, buf); 

            
    }   
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render_frame(frame))?;
            let time = self.increase_spead();
            if event::poll(Duration::from_micros(time))? {
                self.handle_events().wrap_err("handle events failed")?;
            }
            self.update_position()?;
            self.update_enemies()?;
            if self.collision_check() {
                break;
            }
            if self.exit {
                break;
            }
            self.score += 1;
            self.highscore();
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn highscore(&mut self) {
        if self.score > self.highscore {
            self.highscore = self.score;
        }
    }

    fn increase_spead(&self) -> u64 {
        if (self.score / 10000).to_u64().unwrap() >= 5000 {
            return 1;
        }
        else {
            return 5000 - (self.score / 10000).to_u64().unwrap();
        }
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

    fn collision_check(&mut self) -> bool {
        if self.y < -15.0 {
            for enemy in self.enemies.iter() {
                if *enemy < 5.0 && *enemy > -5.0 {
                    return true;
                }
            }
        }
        false
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

    fn update_enemies(&mut self) -> Result<()> {
        let mut rng = thread_rng();
        let mut last_in_range: bool = false;
        if self.enemies.len() > 0 {
            let last_one = self.enemies[self.enemies.len() - 1];
            if last_one < 55.0 || last_one > 84.0 {
                last_in_range = true;
            }
        }
        else {
            last_in_range = true;
        }
        if rng.gen_range(0.0..1.0) < 0.008 && last_in_range {
            self.enemies.push(88.0);
        }
        let mut count = 0;
        for  enemy in self.enemies.iter_mut() {
            if *enemy > - 90.0 {
                *enemy -= 1.0;
            }
            else {
                count += 1;
            }
        }
        for _ in 0..count {
            self.enemies.remove(0);
        }

        Ok(())
    }

    pub fn new() -> App {
        App {
            score: 0,
            highscore: 0,
            exit: false, 
            y: -20.0,
            in_air: false,
            rising: false,
            ducking: false,
            height: 10.0,
            enemies: vec![]
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Down => self.duck()?,
            KeyCode::Up => self.jump()?,
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
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


/*
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

*/