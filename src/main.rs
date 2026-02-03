use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fake::{Fake, Faker};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::{fs, io};

// --- 数据结构 ---

#[derive(Serialize, Deserialize, Clone)]
struct Dataset {
    name: String,
    fields: Vec<String>,
}

#[derive(PartialEq)]
enum AppTab {
    Database,
    Dataset,
}

enum Mode {
    Normal,
    Creating,
    Preview(Vec<String>),
}

struct App {
    current_tab: AppTab,
    mode: Mode,
    datasets: Vec<Dataset>,
    databases: Vec<String>,
    ds_state: ListState,
    db_state: ListState,
    input: String,
}

impl App {
    fn new() -> Self {
        let datasets = load_datasets();
        let mut ds_state = ListState::default();
        if !datasets.is_empty() { ds_state.select(Some(0)); }

        let databases = vec![
            "PostgreSQL_Prod".to_string(),
            "MySQL_Dev".to_string(),
            "Local_SQLite".to_string(),
        ];
        let mut db_state = ListState::default();
        db_state.select(Some(0));

        App {
            current_tab: AppTab::Dataset,
            mode: Mode::Normal,
            datasets,
            databases,
            ds_state,
            db_state,
            input: String::new(),
        }
    }

    fn save(&self) {
        save_datasets(&self.datasets);
    }
}

// --- 主循环 ---

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match &mut app.mode {
                Mode::Preview(_) => {
                    if key.code == KeyCode::Esc { app.mode = Mode::Normal; }
                }
                Mode::Creating => match key.code {
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            app.datasets.push(Dataset {
                                name: app.input.clone(),
                                fields: vec!["name".into(), "age".into()],
                            });
                            app.save();
                            app.ds_state.select(Some(app.datasets.len() - 1));
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
                    KeyCode::Down => move_selection(&mut app, 1),
                    KeyCode::Up => move_selection(&mut app, -1),
                    KeyCode::Enter if app.current_tab == AppTab::Dataset => {
                        if let Some(i) = app.ds_state.selected() {
                            if let Some(ds) = app.datasets.get(i) {
                                app.mode = Mode::Preview(generate_preview(ds));
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

// --- UI 渲染 ---

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Help
        ])
        .split(size);

    // 1. 顶部 Tab
    let menu_titles = vec!["[F1] 数据库管理", "[F2] 数据集管理"];
    let tabs = Tabs::new(menu_titles)
        .block(Block::default().borders(Borders::ALL).title(" 菜单导航 "))
        .select(if app.current_tab == AppTab::Database { 0 } else { 1 })
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    // 2. 主体内容
    match &app.mode {
        Mode::Preview(rows) => {
            let p = Paragraph::new(rows.join("\n"))
                .block(Block::default().title(" 数据预览 (ESC 返回) ").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
            f.render_widget(p, chunks[1]);
        }
        _ => match app.current_tab {
            AppTab::Database => {
                let items: Vec<ListItem> = app.databases.iter()
                    .map(|db| ListItem::new(format!(" 🗄️  {}", db))).collect();
                let list = List::new(items)
                    .block(Block::default().title(" 数据库列表 ").borders(Borders::ALL))
                    .highlight_style(Style::default().bg(Color::Blue))
                    .highlight_symbol(">> ");
                f.render_stateful_widget(list, chunks[1], &mut app.db_state);
            }
            AppTab::Dataset => {
                let items: Vec<ListItem> = app.datasets.iter()
                    .map(|d| ListItem::new(format!(" 📄 {}", d.name))).collect();
                let list = List::new(items)
                    .block(Block::default().title(" 数据集模型 (按 N 新建) ").borders(Borders::ALL))
                    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                    .highlight_symbol(">> ");
                f.render_stateful_widget(list, chunks[1], &mut app.ds_state);
            }
        },
    }

    // 3. 底部提示
    let hint = match app.mode {
        Mode::Creating => format!(" ➤ [新建模式] 输入名称: {}_", app.input),
        Mode::Preview(_) => " ➤ [预览模式] 按 ESC 返回".to_string(),
        Mode::Normal => match app.current_tab {
            AppTab::Database => " 方向键: 选择 | Enter: 管理 | Q: 退出 ".into(),
            AppTab::Dataset => " 方向键: 选择 | Enter: 预览 | N: 新建 | Q: 退出 ".into(),
        },
    };
    f.render_widget(Paragraph::new(hint).block(Block::default().borders(Borders::ALL)), chunks[2]);
}

// --- 逻辑函数 ---

fn move_selection(app: &mut App, delta: i32) {
    let (state, len) = match app.current_tab {
        AppTab::Database => (&mut app.db_state, app.databases.len()),
        AppTab::Dataset => (&mut app.ds_state, app.datasets.len()),
    };
    if len == 0 { return; }
    let i = match state.selected() {
        Some(i) => {
            let next = i as i32 + delta;
            if next < 0 { 0 } else if next >= len as i32 { len - 1 } else { next as usize }
        }
        None => 0,
    };
    state.select(Some(i));
}

fn generate_preview(dataset: &Dataset) -> Vec<String> {
    (0..10).map(|_| {
        let mut row = String::new();
        for field in &dataset.fields {
            let val: String = match field.as_str() {
                "name" => Faker.fake::<String>(),
                "age" => Faker.fake::<u8>().to_string(),
                _ => "N/A".into(),
            };
            row.push_str(&format!("{}={}  ", field, val));
        }
        row
    }).collect()
}

fn save_datasets(datasets: &[Dataset]) {
    let _ = fs::write("datasets.json", serde_json::to_string_pretty(datasets).unwrap());
}

fn load_datasets() -> Vec<Dataset> {
    fs::read_to_string("datasets.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| vec![Dataset { name: "Default_Users".into(), fields: vec!["name".into(), "age".into()] }])
}