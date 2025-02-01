use std::{cmp::min, fmt::Display};

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
    pub restart: Option<()>,
    pub resume: Option<()>,
    pub exit: Option<()>,
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

impl Widget for &OptionMenu {
    // type State = OptionState;

    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .spacing(1);

        // Clear.render(area, buf);

        let option_block = Block::bordered()
            .set_style(Style::new().fg(Color::from_u32(0x00303030)))
            .border_type(BorderType::QuadrantInside)
            .border_style(Style::new().fg(Color::White))
            .title(" Options ")
            .title_alignment(Alignment::Center)
            .padding(Padding::proportional(1));
        option_block.clone().render(area, buf);

        let [size_area, difficulty_area, _, restart_button, continue_button] =
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

        Line::from(Span::from("Restart?").style(if self.cursor_line == 2 {
            Style::default().bg(Color::DarkGray).fg(Color::LightGreen)
        } else {
            Style::default().fg(Color::Gray)
        }))
        .centered()
        .render(restart_button, buf);

        Line::from(Span::from("Continue").style(if self.cursor_line == 3 {
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
    // type State = T;

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
        // let layout = Layout::horizontal([, Constraint::]);
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

impl OptionMenu {
    pub fn new(options: OptionState) -> Self {
        Self {
            cursor_line: 0,
            state: options,
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left => match self.cursor_line {
                0 => {
                    self.state.board_size =
                        FromPrimitive::from_u32((self.state.board_size.clone() as u32 + 3) % 4)
                            .unwrap()
                }
                1 => {
                    self.state.difficulty =
                        FromPrimitive::from_u32((self.state.difficulty.clone() as u32 + 3) % 4)
                            .unwrap()
                }
                _ => (),
            },
            KeyCode::Right => match self.cursor_line {
                0 => {
                    self.state.board_size =
                        FromPrimitive::from_u32((self.state.board_size.clone() as u32 + 1) % 4)
                            .unwrap()
                }
                1 => {
                    self.state.difficulty =
                        FromPrimitive::from_u32((self.state.difficulty.clone() as u32 + 1) % 4)
                            .unwrap()
                }
                _ => (),
            },
            KeyCode::Up => self.cursor_line = self.cursor_line.saturating_sub(1),
            KeyCode::Down => self.cursor_line = min(self.cursor_line + 1, 3),
            KeyCode::Char('q') | KeyCode::Char('Q') => self.state.exit = Some(()),
            KeyCode::Char('o') | KeyCode::Char('O') | KeyCode::Char('c') | KeyCode::Char('C') => {
                self.state.resume = Some(())
            }
            KeyCode::Char('r') | KeyCode::Char('R') => self.state.restart = Some(()),
            KeyCode::Char(' ') | KeyCode::Char('x') | KeyCode::Enter => match self.cursor_line {
                2 => {
                    self.state.restart = Some(());
                }
                3 => {
                    self.state.resume = Some(());
                }
                _ => {}
            },
            _ => {}
        }
    }
}
