use crate::tui;

use color_eyre::owo_colors::OwoColorize;
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

use std::path::Path;

use std::time::Duration;

use rand::prelude::*;

use crate::read_write::*;

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
    enemies: Vec<Vec<f64>>,
    speedy: bool,
    on_puase: bool,
    dead: bool,
    auto: bool,
    black: bool,
    color_switch: bool,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {

        if self.dead {
            let title = Title::from(" Dinosaur Game ".bold());
            let instructions = Title::from(Line::from(vec![
                " Restart ".into(),
                "<Enter> ".bold(),
                " Quit ".into(),
                "<Q> ".bold(),
            ]));
        
            
            let block = Block::default()
                    .title(title.alignment(Alignment::Center)
                        .position(Position::Top))
                    .title(instructions
                        .alignment(Alignment::Center)
                        .position(Position::Bottom))
                    .borders(Borders::ALL);
            
            let best_counter_text = Text::from(vec![Line::from(vec![
                "Highscore ".into(),
                self.highscore.to_string().into(),
            ])]);

            Paragraph::new(best_counter_text)
            .block(block.clone())
            .left_aligned()
            .render(area, buf);
            
            let info_text = Text::from(vec![Line::from(vec![
                "You died with score ".into(),
                self.score.to_string().into(),
            ])]);

            Paragraph::new(info_text)
            .block(block)
            .centered()
            .bold()
            .red()
            .render(area, buf);
        }        
        else {
            let title = Title::from(" Dinosaur Game ".bold());
            let instructions = Title::from(Line::from(vec![
                " Jump ".into(),
                "<Up> ".bold(),
                " Speed ".into(),
                "<Right> ".bold(),
                " Duck ".into(),
                "<Down> ".bold(),
                " Pause ".into(),
                "<Esc> ".bold(),
                " Quit ".into(),
                "<Q> ".bold(),
                " Auto ".into(),
                "<Tab> ".bold(),
            ]));
            
            let color: Color;
            let player_color: Color;

            if self.black {
                color = Color::Black;
                player_color = Color::White;
            }
            else {
                color = Color::White;
                player_color = Color::Black;
            }
            
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
                
            if self.on_puase {
                let pause_text = Text::from("Paused");

                Paragraph::new(pause_text)
                .centered()
                .block(block.clone())
                .style(Style::default().bg(player_color))
                .render(area, buf);
            }

            let player = Canvas::default()
                .block(block)
                .x_bounds([-90.0, 90.0])
                .y_bounds([-45.0, 45.0])
                .background_color(color)
                .paint(|ctx|{
                    ctx.draw(&canvas::Rectangle {
                        x: -5.0,
                        y: self.y,
                        width: 10.0,
                        height: self.height,
                        color: player_color,
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
                                x: enemy[0],
                                y: enemy[2],
                                width: 2.0,
                                height: enemy[1],
                                color: Color::Red,
                            })
                        }
                    }
                });
            player.render(area, buf);  
        }   
    }   
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        loop {
            self.handle_mode()?;
            terminal.draw(|frame| self.render_frame(frame))?;
            let time = self.increase_spead();
            if event::poll(Duration::from_micros(time))? {
                self.handle_events().wrap_err("handle events failed")?;
            }
            if self.exit {
                break;
            }
            if self.on_puase || self.dead {
                continue;
            }
            if self.auto {
                autorun(self)?;
            }
            self.update_position()?;
            self.update_enemies()?;
            if self.collision_check() {
                self.dead = true;
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

    fn handle_mode(&mut self) -> Result<()> {
        if self.score < 2000 || !self.color_switch {
            return Ok(());
        }
        let mut rng = thread_rng();
        let random_num = rng.gen_range(0.0..1.0);
        if self.black && (self.score % 1000 == 0) && random_num < 0.3{
            self.black = false;
        }
        else if self.score % 500 == 0 && random_num < 0.7{
            self.black = true;
        }
        Ok(())
    }

    fn increase_spead(&self) -> u64 {
        if !self.speedy {
            if (self.score / 100).to_u64().unwrap() >= 5000 {
                return 1;
            }
            else {
                return 5000 - (self.score / 100).to_u64().unwrap();
            }
        }
        else {
            if (self.score / 100).to_u64().unwrap() >= 5000 {
                return 1;
            }
            else {
                return (5000 - (self.score / 100).to_u64().unwrap()) / 1000;
            }
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
        for enemy in self.enemies.iter() {
            if enemy[0] < 5.0 && enemy[0] > -5.0 {
                if (self.y >= enemy[2] && self.y <= enemy[2] + enemy[1]) || (self.y + self.height >= (enemy[2] - enemy[1] ) && (self.y + self.height <= enemy[2] + enemy[1])){
                    return true;
                }
            }
        }

        false
    }

    fn update_position(&mut self) -> Result<()> {
        if self.in_air {
            if self.rising {
                if self.y < 15.0 {
                    self.y += 1.25;
                }
                else {
                    self.rising = false;
                }
            }
            else {
                if self.y > -20.0 {
                    self.y -= 1.25;
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
        let mut last_is_flying: bool = false;
        let mut last_one = 0.0;
        if self.enemies.len() > 0 {
            last_one = self.enemies[self.enemies.len() - 1][0];
            if last_one < 50.0 {
                last_in_range = true;
                if self.enemies[self.enemies.len() - 1][2] > -20.0 {
                    last_is_flying = true;
                }
            }
        }
        else {
            last_in_range = true;
        }
        
        if rng.gen_range(0.0..1.0) < 0.015 && last_in_range {
            let mut height = rng.gen_range(5.0..8.0);
            let flying = rng.gen_range(0.0..1.0);
            let mut y = -20.0;
            if flying > 0.75 && flying < 0.82 {
                y = rng.gen_range(-12.0..-8.0);
                height = 1.0;
            }
            else if flying > 0.82 {
                y = rng.gen_range(0.0..5.0);
                height = 1.0;
            }
            self.enemies.push(vec![88.0, height, y]);
        }
        let mut count = 0;
        for  enemy in self.enemies.iter_mut() {
            if enemy[0] > - 90.0 {
                enemy[0] -= 1.0;
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
            enemies: vec![],
            speedy: false,
            on_puase: false,
            dead: false,
            auto: false,
            black: true,
            color_switch: true,
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Down => self.duck()?,
            KeyCode::Up => self.jump()?,
            KeyCode::Right => self.speed()?,
            KeyCode::Esc => self.pause()?,
            KeyCode::Enter => self.restart()?,
            KeyCode::Tab => self.auto()?,
            KeyCode::Char('c') => self.disable_color_switch()?,
            _ => {}
        }
        Ok(())
    }

    fn auto(&mut self) -> Result<()> {
        if self.auto {
            self.auto = false;
        }
        else {
            self.auto = true;
        }
        Ok(())
    }

    fn disable_color_switch(&mut self) -> Result<()> {
        if self.color_switch {
            self.color_switch = false;
        }
        else {
            self.color_switch = true;
        }
        Ok(())
    }

    fn restart(&mut self) -> Result<()> {

        if self.dead {
            let path = Path::new("Highscore.bin");
            save(path, self.highscore)?;
            
            let num = read(path)?;

            self.dead = false;
            self.on_puase = false;
            self.ducking = false;
            self.in_air = false;
            self.y = -20.0;
            self.speedy = false;
            self.enemies = vec![];
            self.height = 10.0;
            self.score = 0;
            self.highscore = num;
            self.auto = false;
            self.black = true;
        }

        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn speed(&mut self) -> Result<()> {
        if self.speedy {
            self.speedy = false;
        }
        else {
            self.speedy = true;
        }
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
        if self.in_air {
            self.rising = false;
        }
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        if self.on_puase {
            self.on_puase = false;
        }
        else {
            self.on_puase = true;
        }
        Ok(())
    }

}

fn autorun(app: &mut App) -> Result<()> {
    let mut enemies_in_front = vec![];

    if app.enemies.len() > 0 {
        for enemy in app.enemies.iter() {
            if enemy[0] > 5.0 {
                enemies_in_front.push(enemy);
            }
        }
        
        if enemies_in_front.len() > 0 {
            let closest_enemy: &Vec<f64> = enemies_in_front[0];

            if !(closest_enemy[0] > 45.0)  {
                if closest_enemy[2] > -20.0 && !app.ducking {
                    app.duck()?;
                }
                else if closest_enemy[2] == -20.0 {
                    app.jump()?;
                }
            }
        }
    }

    Ok(())
}


/*
//TODO: refactor app class and extract player and enemies into seperate classes
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