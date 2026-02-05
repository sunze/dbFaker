mod app;
mod models;
mod ui;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fake::{Fake, Faker};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::{fs, io};
use crate::app::App;
use crate::models::{AppTab, Dataset, Mode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui::render(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match &mut app.mode {
                Mode::Preview(_) => if key.code == KeyCode::Esc { app.mode = Mode::Normal; },
                Mode::Creating => match key.code {
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            app.datasets.push(Dataset { name: app.input.clone(), fields: vec!["name".into(), "age".into()] });
                            app.save();
                        }
                        app.input.clear();
                        app.mode = Mode::Normal;
                    }
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => { app.input.pop(); }
                    KeyCode::Esc => { app.input.clear(); app.mode = Mode::Normal; }
                    _ => {}
                },
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::F(1) => app.current_tab = AppTab::Database,
                    KeyCode::F(2) => app.current_tab = AppTab::Dataset,
                    KeyCode::Char('n') if app.current_tab == AppTab::Dataset => app.mode = Mode::Creating,
                    KeyCode::Down => app.move_selection(1),
                    KeyCode::Up => app.move_selection(-1),
                    KeyCode::Enter if app.current_tab == AppTab::Dataset => {
                        if let Some(i) = app.ds_state.selected() {
                            let preview = app.generate_preview(&app.datasets[i]);
                            app.mode = Mode::Preview(preview);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // 恢复终端
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

