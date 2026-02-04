use std::io;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Constraint, Direction, Layout},
    Frame,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tb_ipc::client::IpcClient;
use tb_ipc::{Message, DEFAULT_PORT};
use tb_ui_widgets::{tape::QuoteTape, chart::{CandleChart, Candle}};

struct AppState {
    symbol: String,
    price: f64,
    candles: Vec<Candle>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to IPC
    let (ipc, mut rx) = match IpcClient::connect(DEFAULT_PORT).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            // Allow running without daemon for UI testing? No, exit.
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
        symbol: "BTCUSDT".to_string(),
        price: 0.0,
        candles: vec![],
    };

    // Run loop
    let res = run_app(&mut terminal, &mut rx, &mut state).await;

    // Restore terminal
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
                         if topic == "market.quote" {
                             if let Some(p) = payload.get("price").and_then(|v| v.as_f64()) {
                                 state.price = p;
                             }
                             if let Some(s) = payload.get("symbol").and_then(|v| v.as_str()) {
                                 state.symbol = s.to_string();
                             }
                         } else if topic.starts_with("market.bars") {
                             if let Some(arr) = payload.as_array() {
                                 state.candles = arr.iter().filter_map(|v| {
                                     Some(Candle {
                                         open: v.get("o")?.as_f64()?,
                                         high: v.get("h")?.as_f64()?,
                                         low: v.get("l")?.as_f64()?,
                                         close: v.get("c")?.as_f64()?,
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(f.size());

    let tape = QuoteTape {
        symbol: state.symbol.clone(),
        price: state.price,
    };
    f.render_widget(tape, chunks[0]);

    let chart = CandleChart {
        data: &state.candles,
    };
    f.render_widget(chart, chunks[1]);
}
