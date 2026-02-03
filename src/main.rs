use std::fs;
use std::io;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fake::{Fake, Faker};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct Dataset {
    name: String,
    fields: Vec<String>,
}

enum Mode {
    Normal,
    Creating,
    Preview(Vec<String>),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ===== terminal init =====
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ===== app state =====
    let mut datasets = load_datasets();
    let mut list_state = ListState::default();
    if !datasets.is_empty() {
        list_state.select(Some(0));
    }

    let mut mode = Mode::Normal;
    let mut input = String::new();

    // ===== event loop =====
    loop {
        terminal.draw(|f| {
            let size = f.size();

            match &mode {
                Mode::Preview(rows) => {
                    let text = rows.join("\n");
                    let p = Paragraph::new(text)
                        .block(
                            Block::default()
                                .title("Preview (ESC 返回)")
                                .borders(Borders::ALL),
                        );
                    f.render_widget(p, size);
                }

                _ => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
                        .split(size);

                    let items: Vec<ListItem> = datasets
                        .iter()
                        .map(|d| ListItem::new(d.name.clone()))
                        .collect();

                    let list = List::new(items)
                        .block(Block::default().title("Datasets").borders(Borders::ALL))
                        .highlight_style(
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol(">> ");

                    f.render_stateful_widget(list, chunks[0], &mut list_state);

                    let hint = match mode {
                        Mode::Creating => format!("输入数据集名称: {}", input),
                        _ => "↑↓ 选择 | Enter 预览 | n 新建 | q 退出".to_string(),
                    };

                    let p = Paragraph::new(hint)
                        .block(Block::default().borders(Borders::ALL));
                    f.render_widget(p, chunks[1]);
                }
            }
        })?;

        if let Event::Key(key) = event::read()? {
            match &mut mode {
                Mode::Preview(_) => {
                    if key.code == KeyCode::Esc {
                        mode = Mode::Normal;
                    }
                }

                Mode::Creating => match key.code {
                    KeyCode::Enter => {
                        if !input.is_empty() {
                            datasets.push(Dataset {
                                name: input.clone(),
                                fields: vec!["name".into(), "age".into()],
                            });
                            save_datasets(&datasets);
                            list_state.select(Some(datasets.len() - 1));
                        }
                        input.clear();
                        mode = Mode::Normal;
                    }
                    KeyCode::Char(c) => input.push(c),
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Esc => {
                        input.clear();
                        mode = Mode::Normal;
                    }
                    _ => {}
                },

                Mode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('n') => mode = Mode::Creating,

                    KeyCode::Down => {
                        if let Some(i) = list_state.selected() {
                            if i + 1 < datasets.len() {
                                list_state.select(Some(i + 1));
                            }
                        }
                    }

                    KeyCode::Up => {
                        if let Some(i) = list_state.selected() {
                            if i > 0 {
                                list_state.select(Some(i - 1));
                            }
                        }
                    }

                    KeyCode::Enter => {
                        if let Some(i) = list_state.selected() {
                            if let Some(ds) = datasets.get(i) {
                                mode = Mode::Preview(generate_preview(ds));
                            }
                        }
                    }

                    _ => {}
                },
            }
        }
    }

    // ===== restore terminal =====
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

// ===== fake preview =====
fn generate_preview(dataset: &Dataset) -> Vec<String> {
    let mut rows = Vec::new();

    for _ in 0..5 {
        let mut row = String::new();
        for field in &dataset.fields {
            let value = match field.as_str() {
                "name" => Faker.fake::<String>(),
                "age" => Faker.fake::<u8>().to_string(),
                _ => "N/A".into(),
            };
            row.push_str(&format!("{}={}  ", field, value));
        }
        rows.push(row);
    }
    rows
}

// ===== storage =====
fn save_datasets(datasets: &Vec<Dataset>) {
    let _ = fs::write(
        "datasets.json",
        serde_json::to_string_pretty(datasets).unwrap(),
    );
}

fn load_datasets() -> Vec<Dataset> {
    fs::read_to_string("datasets.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
