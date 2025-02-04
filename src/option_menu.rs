use std::{cmp::min, fmt::Display};

use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Direction, Layout, Rect},
    prelude::Constraint,
    style::{Color, Modifier, Style, Styled},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Padding, Paragraph, Widget},
};
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug)]
pub struct OptionMenu {
    cursor_line: u32,
    pub state: OptionState,
}

#[derive(Debug, Clone, Default)]
pub struct OptionState {
    pub board_size: SizeOption,
    pub difficulty: DifficultyOption,
    pub theme: Theme,
    pub theme_data: ThemeData,
    pub restart: bool,
    pub resume: bool,
    pub quit: bool,
}

#[derive(EnumIter, PartialEq, Display, Debug, Clone, Default, FromPrimitive)]
pub enum SizeOption {
    Small = 0,
    #[default]
    Medium,
    Large,
    Max,
}

#[derive(EnumIter, PartialEq, Display, Debug, Clone, Default, FromPrimitive)]
pub enum DifficultyOption {
    Easy = 0,
    #[default]
    Medium,
    Hard,
    Expert,
}

#[derive(Debug, Clone)]
pub struct ThemeData {
    pub cursor: Color,
    pub flag: Color,
    pub tile_fg: Color,
    pub tile_bg: Option<Color>,
}

// TODO add flag, dot
impl ThemeData {
    fn new(theme: &Theme) -> Self {
        match theme {
            Theme::Default => Self {
                cursor: Color::Indexed(190),
                flag: Color::Rgb(227, 85, 85),
                tile_fg: Color::Gray,
                tile_bg: None,
            },
            Theme::Light => Self {
                cursor: Color::Indexed(190),
                flag: Color::Rgb(227, 85, 85),
                tile_fg: Color::Rgb(233, 233, 233),
                tile_bg: Some(Color::Rgb(205, 205, 205)),
            },
            Theme::Dark => Self {
                cursor: Color::Rgb(56, 132, 133),
                flag: Color::Rgb(227, 123, 87),
                tile_fg: Color::Rgb(70, 70, 70),
                tile_bg: Some(Color::Rgb(44, 44, 44)),
            },
        }
    }
}

impl Default for ThemeData {
    fn default() -> Self {
        Self::new(&Theme::Default)
    }
}

#[derive(EnumIter, Debug, PartialEq, Display, FromPrimitive, Default, Clone)]
pub enum Theme {
    #[default]
    Default = 0,
    Light,
    Dark,
}

impl OptionMenu {
    pub fn new(options: OptionState) -> Self {
        Self {
            cursor_line: 0,
            state: options,
        }
    }

    fn update_theme(&mut self) {
        self.state.theme_data = ThemeData::new(&self.state.theme);
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left => match self.cursor_line {
                0 => {
                    self.state.board_size =
                        FromPrimitive::from_u32((self.state.board_size.clone() as u32 + 4) % 5)
                            .unwrap()
                }
                1 => {
                    self.state.difficulty =
                        FromPrimitive::from_u32((self.state.difficulty.clone() as u32 + 4) % 5)
                            .unwrap()
                }
                2 => {
                    self.state.theme =
                        FromPrimitive::from_u32((self.state.theme.clone() as u32 + 2) % 3).unwrap();
                    self.update_theme();
                }
                _ => (),
            },
            KeyCode::Right => match self.cursor_line {
                0 => {
                    self.state.board_size =
                        FromPrimitive::from_u32((self.state.board_size.clone() as u32 + 1) % 5)
                            .unwrap()
                }
                1 => {
                    self.state.difficulty =
                        FromPrimitive::from_u32((self.state.difficulty.clone() as u32 + 1) % 5)
                            .unwrap()
                }
                2 => {
                    self.state.theme =
                        FromPrimitive::from_u32((self.state.theme.clone() as u32 + 1) % 3).unwrap();
                    self.update_theme();
                }
                _ => (),
            },
            KeyCode::Up => self.cursor_line = self.cursor_line.saturating_sub(1),
            KeyCode::Down => self.cursor_line = min(self.cursor_line + 1, 4),
            KeyCode::Char('q') | KeyCode::Char('Q') => self.state.quit = true,
            KeyCode::Char('o') | KeyCode::Char('O') | KeyCode::Char('c') | KeyCode::Char('C') => {
                self.state.resume = true
            }
            KeyCode::Char('r') | KeyCode::Char('R') => self.state.restart = true,
            KeyCode::Char(' ') | KeyCode::Char('x') | KeyCode::Enter => match self.cursor_line {
                3 => {
                    self.state.restart = true;
                }
                4 => {
                    self.state.resume = true;
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl Widget for &OptionMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .spacing(1);

        // Clear.render(area, buf);

        let option_block = Block::bordered()
            .set_style(
                Style::new().fg(Color::Rgb(48, 48, 48)).bg(Color::Reset), //
                                                                          // Style::new().fg(self.state.theme_data.tile_fg).bg(self
                                                                          //     .state
                                                                          //     .theme_data
                                                                          //     .tile_bg
                                                                          //     .unwrap_or(Color::Reset)),
            )
            // .border_type(BorderType::QuadrantInside)
            .border_style(Style::new().fg(Color::White).bg(Color::Reset))
            .title(" Options ")
            .title_alignment(Alignment::Center)
            .padding(Padding::proportional(1));
        option_block.clone().render(area, buf);

        let [size_area, difficulty_area, theme_area, _, restart_button, continue_button] =
            layout.areas(option_block.inner(area));

        SingleSelector::<SizeOption>::new(
            String::from("Board size:"),
            self.state.board_size.clone(),
            self.cursor_line == 0,
        )
        .render(size_area, buf);

        SingleSelector::<DifficultyOption>::new(
            String::from("Difficulty:"),
            self.state.difficulty.clone(),
            self.cursor_line == 1,
        )
        .render(difficulty_area, buf);

        SingleSelector::<Theme>::new(
            String::from("Theme:"),
            self.state.theme.clone(),
            self.cursor_line == 2,
        )
        .render(theme_area, buf);

        Line::from(Span::from("Restart?").style(if self.cursor_line == 3 {
            Style::default().bg(Color::DarkGray).fg(Color::LightGreen)
        } else {
            Style::default().fg(Color::Gray)
        }))
        .centered()
        .render(restart_button, buf);

        Line::from(Span::from("Continue").style(if self.cursor_line == 4 {
            Style::default().bg(Color::DarkGray).fg(Color::LightGreen)
        } else {
            Style::default().fg(Color::Gray)
        }))
        .centered()
        .render(continue_button, buf);
    }
}

#[derive(PartialEq, Default)]
struct SingleSelector<T> {
    label: String,
    state: T,
    pub highlight: bool,
}

impl<T: Default> SingleSelector<T> {
    fn new(label: String, state: T, highlight: bool) -> Self {
        Self {
            label,
            state,
            highlight,
        }
    }
}

impl<T: IntoEnumIterator + Display + PartialEq + FromPrimitive> Widget for SingleSelector<T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.highlight {
            Clear.render(area, buf);
            buf.set_style(area, Style::new().bg(Color::DarkGray));
        }

        let style_base = Style::default().fg(Color::White);

        let mut constraints = vec![
            Constraint::Length(self.label.len() as u16),
            Constraint::Fill(1),
        ];

        for i in T::iter() {
            constraints.push(Constraint::Length(i.to_string().len() as u16 + 2));
        }

        let areas = Layout::new(Direction::Horizontal, constraints)
            .spacing(1)
            .split(area);

        Paragraph::new(self.label)
            .style(style_base)
            .render(areas[0], buf);

        let mut count = 2;
        for i in T::iter() {
            Paragraph::new(format!("<{}>", i.to_string()))
                .style(if self.state == i {
                    style_base
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::LightGreen)
                } else {
                    style_base
                })
                .render(areas[count], buf);
            count += 1;
        }
    }
}
