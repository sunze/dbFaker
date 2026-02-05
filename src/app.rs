use std::fs;
use ratatui::widgets::ListState;
use fake::{Fake, Faker};
use crate::models::{AppTab, Dataset, Mode};

pub struct App {
    pub current_tab: AppTab,
    pub mode: Mode,
    pub datasets: Vec<Dataset>,
    pub databases: Vec<String>,
    pub ds_state: ListState,
    pub db_state: ListState,
    pub input: String,
}


impl App {
    pub fn new() -> Self {
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

    pub fn move_selection(&mut self, delta: i32) {
        let (state, len) = match self.current_tab {
            AppTab::Database => (&mut self.db_state, self.databases.len()),
            AppTab::Dataset => (&mut self.ds_state, self.datasets.len()),
        };
        if len == 0 { return; }
        let i = match state.selected() {
            Some(i) => {
                let next = i as i32 + delta;
                next.rem_euclid(len as i32) as usize
            }
            None => 0,
        };
        state.select(Some(i));
    }

    pub fn generate_preview(&self, dataset: &Dataset) -> Vec<String> {
        (0..10).map(|_| {
            dataset.fields.iter().map(|f| {
                let val = match f.as_str() {
                    "name" => Faker.fake::<String>(),
                    "age" => Faker.fake::<u8>().to_string(),
                    _ => "N/A".into(),
                };
                format!("{}={}", f, val)
            }).collect::<Vec<_>>().join("  ")
        }).collect()
    }

    pub fn save(&self) {
        save_datasets(&self.datasets);
    }


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