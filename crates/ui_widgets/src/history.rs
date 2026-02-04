use ratatui::{
    buffer::Buffer,
    layout::{Rect, Constraint},
    style::{Color, Style, Modifier},
    widgets::{Widget, Block, Borders, Table, Row, Cell},
};
use crate::Bar;

pub struct HistoryTable<'a> {
    pub data: &'a [Bar],
}

impl<'a> Widget for HistoryTable<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title("History").borders(Borders::ALL);
        let inner = block.inner(area);
        block.render(area, buf);

        let header = Row::new(vec![
            "Time", "Open", "High", "Low", "Close", "Vol"
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        let rows: Vec<Row> = self.data.iter().rev().take(50).map(|bar| {
            let color = if bar.close >= bar.open { Color::Green } else { Color::Red };
            Row::new(vec![
                Cell::from(bar.ts.chars().take(19).collect::<String>()),
                Cell::from(format!("{:.2}", bar.open)),
                Cell::from(format!("{:.2}", bar.high)),
                Cell::from(format!("{:.2}", bar.low)),
                Cell::from(format!("{:.2}", bar.close)),
                Cell::from(format!("{:.0}", bar.volume)),
            ]).style(Style::default().fg(color))
        }).collect();

        Table::new(rows, [
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ])
        .header(header)
        .render(inner, buf);
    }
}
