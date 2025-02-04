use std::{
    io,
    time::{Duration, Instant},
};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use minesweeper::{
    lose_menu::*,
    option_menu::{
        DifficultyOption::{self},
        OptionMenu, OptionState,
        SizeOption::{self, Medium, Small},
    },
    win_menu::WinMenu,
    Board, TileState,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Widget},
    DefaultTerminal, Frame,
};

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let app_result = App::new(OptionState::default(), &terminal).run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug)]
pub struct App {
    board: minesweeper::Board,
    start_time: Instant,
    cursor: (usize, usize),
    state: GameState,
    prev_state: Option<GameState>,
    restart: bool,
    quit: bool,
    option_menu: OptionMenu,
    lose_menu: LoseMenu,
    win_menu: WinMenu,
}

#[derive(Debug, PartialEq, Clone)]
enum GameState {
    Start,
    Play,
    Options,
    Win,
    Lose,
}

// TODO refactor all style using Stylize

impl App {
    pub fn new(options: OptionState, terminal: &DefaultTerminal) -> Self {
        let size = match options.board_size {
            Small => (15, 10),
            Medium => (24, 16),
            SizeOption::Large => (48, 32),
            SizeOption::Max => {
                let ta = terminal.size().unwrap();
                ((ta.width as usize - 6 - 2 + 1) / 2, ta.height as usize - 4)
            }
        };
        Self {
            board: Board::new(
                size.0,
                size.1,
                (match options.difficulty {
                    DifficultyOption::Easy => 0.12,
                    DifficultyOption::Medium => 0.15,
                    DifficultyOption::Hard => 0.17,
                    DifficultyOption::Expert => 0.20,
                } * (size.0 * size.1) as f32) as u32,
            ),
            start_time: Instant::now(),
            cursor: (size.0 / 2, size.1 / 2),
            state: GameState::Start,
            prev_state: None,
            restart: false,
            quit: false,
            option_menu: OptionMenu::new(options),
            lose_menu: Default::default(),
            win_menu: Default::default(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.quit {
            if self.restart {
                *self = Self::new(self.option_menu.state.clone(), terminal);
            }
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let frame_area_centered = center(
            frame.area(),
            Constraint::Length((self.board.width * 2 - 1) as u16 + 8),
            Constraint::Length(self.board.height as u16 + 4),
        );

        let area = frame.area();

        self.option_menu
            .state
            .theme_data
            .tile_bg
            .inspect(|c| frame.buffer_mut().set_style(area, Style::new().bg(*c)));

        frame.render_widget(self, frame_area_centered);

        if self.state == GameState::Options {
            frame.render_widget(
                &self.option_menu,
                center(frame.area(), Constraint::Length(59), Constraint::Length(15)),
            );
        } else if self.state == GameState::Lose {
            frame.render_widget(
                &self.lose_menu,
                center(frame.area(), Constraint::Length(31), Constraint::Length(12)),
            );
        } else if self.state == GameState::Win {
            frame.render_widget(
                &self.win_menu,
                center(frame.area(), Constraint::Length(25), Constraint::Length(7)),
            );
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        while event::poll(Duration::from_millis(1))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match self.state {
                        GameState::Play | GameState::Start => self.handle_key_event(key_event),
                        GameState::Options => self.option_menu.handle_key_event(key_event),
                        GameState::Lose => self.lose_menu.handle_key_event(key_event),
                        GameState::Win => self.win_menu.handle_key_event(key_event),
                    }

                    if self.option_menu.state.quit || self.lose_menu.quit || self.win_menu.quit {
                        self.quit();
                    } else if self.option_menu.state.restart
                        || self.lose_menu.restart
                        || self.win_menu.restart
                    {
                        self.option_menu.state.restart = false;
                        self.restart = true;
                    } else if self.option_menu.state.resume {
                        self.state = self.prev_state.clone().unwrap_or(GameState::Play);
                        self.option_menu.state.resume = false;
                    } else if self.lose_menu.options || self.win_menu.options {
                        self.lose_menu.options = false;
                        self.win_menu.options = false;
                        // TODO maybe change this
                        self.handle_key_event(KeyEvent::new(
                            KeyCode::Char('o'),
                            KeyModifiers::empty(),
                        ));
                    } else if self.lose_menu.coward {
                        self.lose_menu.coward = false;
                        self.board.undo(self.cursor.0, self.cursor.1);
                        self.state = GameState::Play;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left => self.move_cursor(-1, 0),
            KeyCode::Right => self.move_cursor(1, 0),
            KeyCode::Up => self.move_cursor(0, -1),
            KeyCode::Down => self.move_cursor(0, 1),
            KeyCode::Char('q') | KeyCode::Char('Q') => self.quit(),
            KeyCode::Char('x') | KeyCode::Char('X') => {
                if self.state == GameState::Start {
                    self.state = GameState::Play;
                    self.start_time = Instant::now();
                    self.board.first_dig(self.cursor.0, self.cursor.1);
                } else {
                    if let Err(()) = self.board.dig(self.cursor.0, self.cursor.1) {
                        self.state = GameState::Lose;
                    } else if self.board.game_won() {
                        self.state = GameState::Win;
                        self.win_menu.time = self.start_time.elapsed().as_secs();
                    }
                }
            }
            KeyCode::Char('z') | KeyCode::Char('Z') => {
                if self.state == GameState::Play {
                    self.board.flag(self.cursor.0, self.cursor.1)
                }
                if self.board.game_won() {
                    self.state = GameState::Win;
                    self.win_menu.time = self.start_time.elapsed().as_secs();
                }
            }
            KeyCode::Char('o') | KeyCode::Char('O') => {
                self.prev_state = Some(self.state.clone());
                self.state = GameState::Options;
            }
            _ => {}
        }
    }

    fn move_cursor(&mut self, dx: i8, dy: i8) {
        if self.cursor.0 as i8 + dx >= 0
            && self.cursor.0 as i8 + dx < self.board.width as i8
            && self.cursor.1 as i8 + dy >= 0
            && self.cursor.1 as i8 + dy < self.board.height as i8
        {
            self.cursor = (
                (self.cursor.0 as i8 + dx) as usize,
                (self.cursor.1 as i8 + dy) as usize,
            );
        }
    }

    fn quit(&mut self) {
        self.quit = true;
    }

    fn styled_board(&self) -> Vec<Line<'_>> {
        let mut lines: Vec<Line<'_>> = vec![];

        for y in 0..self.board.height {
            let mut span_vec: Vec<Span> = vec![];
            for x in 0..self.board.width {
                if (x, y) == self.cursor
                    && (self.state == GameState::Start || self.state == GameState::Play)
                {
                    span_vec.push(self.ts_span_cursor(self.board.check(x, y)));
                } else {
                    span_vec.push(self.ts_span(self.board.check(x, y)));
                }

                if x < self.board.width - 1 {
                    span_vec.push(Span::raw(" "));
                }
            }
            if self.option_menu.state.theme_data.tile_bg.is_some() {
                lines.push(
                    Line::from(span_vec).bg(self.option_menu.state.theme_data.tile_bg.unwrap()),
                );
            } else {
                lines.push(Line::from(span_vec));
            }
        }

        lines
    }

    fn ts_span_cursor(&self, ts: TileState) -> Span<'static> {
        match ts {
            minesweeper::TileState::Hidden => Span::styled(
                "◼",
                Style::default()
                    .fg(Color::Black)
                    .bg(self.option_menu.state.theme_data.cursor),
            ),
            minesweeper::TileState::Flagged => Span::styled(
                "◄",
                Style::default()
                    .fg(Color::Black)
                    .bg(self.option_menu.state.theme_data.cursor),
            ),
            minesweeper::TileState::Empty => Span::styled(
                "·",
                Style::default()
                    .fg(Color::Black)
                    .bg(self.option_menu.state.theme_data.cursor),
            ),
            minesweeper::TileState::Mine => {
                Span::styled("◉", Style::default().fg(Color::Black).bg(Color::Red))
            }
            minesweeper::TileState::Count(n) => Span::styled(
                n.to_string(),
                Style::default()
                    .fg(Color::Black)
                    .bg(self.option_menu.state.theme_data.cursor),
            ),
        }
    }

    fn ts_span(&self, ts: TileState) -> Span<'static> {
        match ts {
            minesweeper::TileState::Hidden => Span::styled(
                "◼",
                Style::default().fg(self.option_menu.state.theme_data.tile_fg),
            ),
            minesweeper::TileState::Flagged => Span::styled(
                "◄",
                Style::default().fg(self.option_menu.state.theme_data.flag),
            ),
            minesweeper::TileState::Empty => Span::styled(
                "·",
                Style::default().fg(self.option_menu.state.theme_data.tile_fg),
            ),
            minesweeper::TileState::Mine => Span::styled("◉", Style::default().fg(Color::Red)),
            minesweeper::TileState::Count(n) => {
                Span::styled(n.to_string(), Style::default().fg(number_colors(n)))
            }
        }
    }
}

fn number_colors(n: u8) -> Color {
    // this is a fn because color does not implement Sized
    match n {
        1 => Color::Indexed(39),
        2 => Color::Indexed(48),
        3 => Color::Indexed(175),
        4 => Color::Indexed(105),
        5 => Color::Indexed(162),
        6 => Color::Indexed(31),
        7 => Color::Indexed(255),
        _ => Color::Indexed(244),
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner_block = Block::bordered()
            .border_set(border::THICK)
            .title_top(Span::styled(
                " Minesweeper ",
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .title(
                Line::from(format!(
                    " {}/{} ",
                    self.board.mines_left, self.board.mine_count
                ))
                .centered(),
            )
            .title(
                Line::from(match self.state {
                    GameState::Play => self.start_time.elapsed().as_secs().to_string(),
                    _ => String::from("0"),
                })
                .right_aligned(),
            )
            .title_bottom(Line::from(" Quit [q] ").left_aligned())
            .title_bottom(Line::from(" Options [o] ").right_aligned())
            .border_style(Style::new().fg(Color::White))
            .padding(Padding::symmetric(3, 1));

        Paragraph::new(self.styled_board())
            .block(inner_block)
            .render(area, buf);
    }
}
