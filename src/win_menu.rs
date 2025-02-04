use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Alignment::Center,
    prelude::{Buffer, Rect},
    style::{Color, Style, Styled, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType::Double, Padding, Paragraph, Widget},
};

#[derive(Debug, Clone, Default)]
pub struct WinMenu {
    pub time: u64,
    pub restart: bool,
    pub options: bool,
    pub quit: bool,
}

impl WinMenu {
    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.quit = true,
            KeyCode::Char('o') | KeyCode::Char('O') => self.options = true,
            _ => self.restart = true,
        }
    }
}

impl Widget for &WinMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear.render(area, buf);
        let win_block = Block::bordered()
            .set_style(
                Style::new()
                    .fg(Color::from_u32(0x00303030))
                    .bg(Color::Reset),
            )
            .border_style(Style::new().fg(Color::White).bg(Color::Reset))
            .border_type(Double)
            .padding(Padding::proportional(1))
            .title(" Congratulations! ")
            .title_alignment(Center);
        win_block.clone().render(area, buf);
        Paragraph::new(vec![
            Line::raw(""),
            Line::from(Span::from(format!("Time: {}s", self.time).fg(Color::White))),
            Line::raw(""),
        ])
        .centered()
        .render(win_block.inner(area), buf);
    }
}
