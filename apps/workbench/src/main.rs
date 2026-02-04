use std::io;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Constraint, Direction, Layout},
    Frame,
    widgets::{Block, Borders},
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tb_ipc::client::IpcClient;
use tb_ipc::{Message, DEFAULT_PORT};
use tb_ui_widgets::{chart::{CandleChart, Candle}, history::HistoryTable, Bar};

struct AppState {
    bars: Vec<Bar>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to IPC
    let (_ipc, mut rx) = match IpcClient::connect(DEFAULT_PORT).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            return Ok(());
        }
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState {
        bars: vec![],
    };

    // Run loop
    let res = run_app(&mut terminal, &mut rx, &mut state).await;

    // Restore
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    rx: &mut tokio::sync::mpsc::Receiver<Message>,
    state: &mut AppState,
) -> io::Result<()> {
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
    loop {
        terminal.draw(|f| ui(f, state))?;

        tokio::select! {
             _ = interval.tick() => {
                 if crossterm::event::poll(std::time::Duration::from_millis(0))? {
                     if let Event::Key(key) = event::read()? {
                         if let KeyCode::Char('q') = key.code {
                             return Ok(());
                         }
                     }
                 }
             }
             Some(msg) = rx.recv() => {
                 match msg {
                     Message::Event { topic, payload } => {
                         if topic.starts_with("market.bars") {
                             if let Some(arr) = payload.as_array() {
                                 state.bars = arr.iter().filter_map(|v| {
                                     Some(Bar {
                                         ts: v.get("ts")?.as_str()?.to_string(),
                                         open: v.get("o")?.as_f64()?,
                                         high: v.get("h")?.as_f64()?,
                                         low: v.get("l")?.as_f64()?,
                                         close: v.get("c")?.as_f64()?,
                                         volume: v.get("v")?.as_f64()?,
                                     })
                                 }).collect();
                             }
                         }
                     }
                     _ => {}
                 }
             }
        }
    }
}

fn ui(f: &mut Frame, state: &AppState) {
    let outer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(50), Constraint::Percentage(30)])
        .split(f.size());

    // Left: Nav
    f.render_widget(Block::default().title("Nav").borders(Borders::ALL), outer[0]);

    // Center: Chart + Table
    let center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(outer[1]);

    let candles: Vec<Candle> = state.bars.iter().map(|b| Candle {
        open: b.open, high: b.high, low: b.low, close: b.close
    }).collect();

    f.render_widget(CandleChart { data: &candles }, center[0]);
    f.render_widget(HistoryTable { data: &state.bars }, center[1]);

    // Right: AI + Risk
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(outer[2]);

    f.render_widget(Block::default().title("AI Console").borders(Borders::ALL), right[0]);
    f.render_widget(Block::default().title("Risk").borders(Borders::ALL), right[1]);
}
