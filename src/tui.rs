#[cfg(feature = "tui")]
use anyhow::Result;
#[cfg(feature = "tui")]
use bitcoincore_rpc::{Client, RpcApi};
#[cfg(feature = "tui")]
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame, Terminal,
};
#[cfg(feature = "tui")]
use std::io;

#[cfg(feature = "tui")]
use crate::{ReorgCalculation, format_hashrate};

#[cfg(feature = "tui")]
pub struct TuiApp {
    pub should_quit: bool,
    pub current_tab: usize,
    pub calculations: Vec<ReorgCalculation>,
    pub progress: f64,
    pub status_message: String,
    pub hashrate: f64,
    pub target_days: f64,
    pub current_height: u64,
    pub is_calculating: bool,
}

#[cfg(feature = "tui")]
impl TuiApp {
    pub fn new(hashrate: f64, target_days: f64, current_height: u64) -> Self {
        Self {
            should_quit: false,
            current_tab: 0,
            calculations: Vec::new(),
            progress: 0.0,
            status_message: "Ready to calculate".to_string(),
            hashrate,
            target_days,
            current_height,
            is_calculating: false,
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 3;
    }

    pub fn prev_tab(&mut self) {
        self.current_tab = if self.current_tab == 0 { 2 } else { self.current_tab - 1 };
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

#[cfg(feature = "tui")]
pub fn run_tui(
    client: Client,
    hashrate: f64,
    target_days: f64,
) -> Result<()> {
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let current_height = client.get_block_count()?;
    let mut app = TuiApp::new(hashrate, target_days, current_height);

    // Main loop
    let result = run_app(&mut terminal, &mut app, client);

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

#[cfg(feature = "tui")]
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TuiApp,
    _client: Client,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Char('q') => {
                        app.quit();
                    }
                    crossterm::event::KeyCode::Tab => {
                        app.next_tab();
                    }
                    crossterm::event::KeyCode::BackTab => {
                        app.prev_tab();
                    }
                    crossterm::event::KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        app.quit();
                    }
                    crossterm::event::KeyCode::Char('r') => {
                        if !app.is_calculating {
                            app.is_calculating = true;
                            app.status_message = "Calculating viable heights...".to_string();
                            // TODO: Start calculation in background
                        }
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

#[cfg(feature = "tui")]
fn ui(f: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new("Testnet4 Reorg Calculator - Interactive Mode")
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Main content based on current tab
    match app.current_tab {
        0 => render_parameters_tab(f, chunks[1], app),
        1 => render_calculations_tab(f, chunks[1], app),
        2 => render_progress_tab(f, chunks[1], app),
        _ => {}
    }

    // Status bar
    let status = Paragraph::new(app.status_message.clone())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}

#[cfg(feature = "tui")]
fn render_parameters_tab(f: &mut Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new("Parameters")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let hashrate_text = format!("Hashrate: {}", format_hashrate(app.hashrate));
    let hashrate_para = Paragraph::new(hashrate_text)
        .block(Block::default().borders(Borders::ALL).title("Current Settings"));
    f.render_widget(hashrate_para, chunks[1]);

    let target_text = format!("Target Time: {:.1} days", app.target_days);
    let target_para = Paragraph::new(target_text)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(target_para, chunks[2]);

    let help_text = vec![
        Line::from("Press 'r' to run calculations"),
        Line::from("Press 'Tab' to switch tabs"),
        Line::from("Press 'q' to quit"),
    ];
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, chunks[3]);
}

#[cfg(feature = "tui")]
fn render_calculations_tab(f: &mut Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new("Calculation Results")
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    if app.calculations.is_empty() {
        let empty_text = Paragraph::new("No calculations yet. Press 'r' to start.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(empty_text, chunks[1]);
    } else {
        let items: Vec<ListItem> = app.calculations
            .iter()
            .map(|calc| {
                let text = format!(
                    "Height {}: {:.2} days ({} needed)",
                    calc.fork_height,
                    calc.time_required_days,
                    format_hashrate(calc.hashrate_required)
                );
                ListItem::new(text)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Viable Heights"));
        f.render_widget(list, chunks[1]);
    }
}

#[cfg(feature = "tui")]
fn render_progress_tab(f: &mut Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new("Progress")
        .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let progress_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Calculation Progress"))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(app.progress);
    f.render_widget(progress_gauge, chunks[1]);

    let status_text = if app.is_calculating {
        "Calculating viable heights..."
    } else {
        "Ready"
    };
    let status = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, chunks[2]);
}

// Non-TUI compilation support
#[cfg(not(feature = "tui"))]
pub fn run_tui(_client: Client, _hashrate: f64, _target_days: f64) -> Result<()> {
    Err(anyhow::anyhow!("TUI mode not available. Compile with --features tui"))
}
