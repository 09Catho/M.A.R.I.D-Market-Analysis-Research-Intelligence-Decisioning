use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color},
    widgets::{Widget, Block, Borders},
};
use ratatui::widgets::canvas::{Canvas, Line, Rectangle};

pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub struct CandleChart<'a> {
    pub data: &'a [Candle],
}

impl<'a> Widget for CandleChart<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
         let block = Block::default().title("Chart").borders(Borders::ALL);
         let inner = block.inner(area);
         block.render(area, buf);

         if self.data.is_empty() { return; }

         // Find min/max
         let min = self.data.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
         let max = self.data.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
         let range = max - min;
         let padding = range * 0.1;

         Canvas::default()
             .block(Block::default().borders(Borders::NONE))
             .x_bounds([0.0, self.data.len() as f64])
             .y_bounds([min - padding, max + padding])
             .paint(|ctx| {
                 for (i, candle) in self.data.iter().enumerate() {
                     let color = if candle.close >= candle.open { Color::Green } else { Color::Red };

                     // Wick
                     ctx.draw(&Line {
                         x1: i as f64 + 0.5,
                         y1: candle.low,
                         x2: i as f64 + 0.5,
                         y2: candle.high,
                         color,
                     });

                     // Body
                     let (y1, y2) = if candle.close >= candle.open {
                         (candle.open, candle.close)
                     } else {
                         (candle.close, candle.open)
                     };

                     ctx.draw(&Rectangle {
                         x: i as f64 + 0.2,
                         y: y1,
                         width: 0.6,
                         height: (y2 - y1).max(0.0001), // Ensure at least visible
                         color,
                     });
                 }
             })
             .render(inner, buf);
    }
}
