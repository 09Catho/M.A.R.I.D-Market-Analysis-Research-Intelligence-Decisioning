use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Widget, Block, Borders},
};

pub struct QuoteTape {
    pub symbol: String,
    pub price: f64,
}

impl Widget for QuoteTape {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title("Tape").borders(Borders::ALL);
        let inner = block.inner(area);
        block.render(area, buf);

        let text = format!("{}  ${:.2}", self.symbol, self.price);
        buf.set_string(inner.x + 1, inner.y, text, Style::default().fg(Color::Green));
    }
}
