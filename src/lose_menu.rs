use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    prelude::{self, Rect},
    style::{Color, Style, Styled, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Padding, Paragraph, Widget},
};

#[derive(Debug, Clone, Default)]
pub struct LoseMenu {
    pub continue_select: bool,
    pub restart: bool,
    pub coward: bool,
    pub options: bool,
    pub quit: bool,
}

//TODO fix wronog colour for new game
impl Widget for &LoseMenu {
    fn render(self, area: Rect, buf: &mut prelude::Buffer) {
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .spacing(1);
        // Clear.render(area, buf);

        let lose_block = Block::bordered()
            .set_style(
                Style::new()
                    .fg(Color::from_u32(0x00303030))
                    .bg(Color::Reset),
            )
            .border_style(Style::new().fg(Color::White).bg(Color::Reset))
            // .border_type(BorderType::QuadrantInside)
            .title(" Oops! ")
            .title_alignment(Alignment::Center)
            .padding(Padding::proportional(1));
        lose_block.clone().render(area, buf);

        let [msg_area, restart_area, continue_area] = layout.areas(lose_block.inner(area));

        Paragraph::new(vec![
            Line::raw(""),
            Line::from("You exploded").fg(Color::White),
            Line::raw(""),
        ])
        .centered()
        .render(msg_area, buf);

        Line::from(Span::from("New Game").style(if !self.continue_select {
            Style::new().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default().fg(Color::White)
        }))
        .centered()
        .render(restart_area, buf);

        Paragraph::new(vec![
            Span::from("Continue")
                .style(if self.continue_select {
                    Style::new().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default().fg(Color::White)
                })
                .into(),
            Span::from("[ dishonourable ]")
                .set_style(Style::new().fg(Color::Red))
                .into(),
        ])
        .centered()
        .render(continue_area, buf);
    }
}

impl LoseMenu {
    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up | KeyCode::Down => self.continue_select = !self.continue_select,
            KeyCode::Char('q') | KeyCode::Char('Q') => self.quit = true,
            KeyCode::Char('o') | KeyCode::Char('O') => self.options = true,
            KeyCode::Char('r') | KeyCode::Char('R') => self.restart = true,
            KeyCode::Char(' ') | KeyCode::Char('x') | KeyCode::Enter => {
                if self.continue_select {
                    self.coward = true
                } else {
                    self.restart = true
                }
            }
            _ => {}
        }
    }
}
