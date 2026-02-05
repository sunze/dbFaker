use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};
use crate::app::{App};
use crate::models::{AppTab, Mode};

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(f.size());

    // 1. 顶部导航
    let titles = vec!["[F1] 数据库", "[F2] 数据集"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" 导航 "))
        .select(match app.current_tab { AppTab::Database => 0, AppTab::Dataset => 1 })
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    // 2. 内容区
    match &app.mode {
        Mode::Preview(rows) => {
            let p = Paragraph::new(rows.join("\n"))
                .block(Block::default().title(" 预览 (ESC退出) ").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
            f.render_widget(p, chunks[1]);
        }
        _ => render_main_content(f, app, chunks[1]),
    }

    // 3. 底部帮助栏
    let hint = match app.mode {
        Mode::Creating => format!(" ➤ [新建] 名称: {}_", app.input),
        Mode::Preview(_) => " ➤ [预览] ESC 返回".into(),
        _ => " Q: 退出 | F1/F2: 切换 | Enter: 确认 | N: 新建 ".into(),
    };
    f.render_widget(Paragraph::new(hint).block(Block::default().borders(Borders::ALL)), chunks[2]);
}

fn render_main_content(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    match app.current_tab {
        AppTab::Database => {
            let items: Vec<ListItem> = app.databases.iter().map(|db| ListItem::new(format!(" 🗄️  {}", db))).collect();
            let list = List::new(items).block(Block::default().title(" 数据库 ").borders(Borders::ALL))
                .highlight_symbol(">> ").highlight_style(Style::default().bg(Color::Blue));
            f.render_stateful_widget(list, area, &mut app.db_state);
        }
        AppTab::Dataset => {
            let items: Vec<ListItem> = app.datasets.iter().map(|d| ListItem::new(format!(" 📄 {}", d.name))).collect();
            let list = List::new(items).block(Block::default().title(" 数据集 ").borders(Borders::ALL))
                .highlight_symbol(">> ").highlight_style(Style::default().bg(Color::DarkGray));
            f.render_stateful_widget(list, area, &mut app.ds_state);
        }
    }
}