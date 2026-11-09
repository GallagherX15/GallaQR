use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QrHistoryItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Local>,
    pub ecc_level: String,
    pub color_hex: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct QrHistory {
    pub items: Vec<QrHistoryItem>,
}

impl QrHistory {
    pub fn get_storage_path() -> Option<PathBuf> {
        // Modo portable: si existe un marcador .portable o history.json junto al ejecutable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let portable_marker = exe_dir.join(".portable");
                let portable_history = exe_dir.join("history.json");
                if portable_marker.exists() || portable_history.exists() {
                    return Some(portable_history);
                }
            }
        }

        let base_dir = dirs::data_local_dir()
            .or_else(dirs::config_dir)
            .unwrap_or_else(|| PathBuf::from("."));
        let app_dir = base_dir.join("gallaqr");
        if !app_dir.exists() {
            let _ = fs::create_dir_all(&app_dir);
        }
        Some(app_dir.join("history.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::get_storage_path() {
            if path.exists() {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(history) = serde_json::from_str::<QrHistory>(&data) {
                        return history;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::get_storage_path() {
            if let Ok(data) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, data);
            }
        }
    }

    pub fn add(
        &mut self,
        title: String,
        content: String,
        ecc_level: String,
        color_hex: String,
    ) -> QrHistoryItem {
        let item = QrHistoryItem {
            id: format!("{}-{}", Local::now().timestamp_millis(), self.items.len()),
            title,
            content,
            created_at: Local::now(),
            ecc_level,
            color_hex,
        };
        self.items.insert(0, item.clone());
        self.save();
        item
    }

    pub fn remove(&mut self, id: &str) {
        self.items.retain(|item| item.id != id);
        self.save();
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.save();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_in_memory_operations() {
        let mut history = QrHistory::default();
        assert_eq!(history.items.len(), 0);

        let item1 = history.add(
            "Mi Portafolio".to_string(),
            "https://miportafolio.com".to_string(),
            "M".to_string(),
            "#000000".to_string(),
        );
        assert_eq!(history.items.len(), 1);
        assert_eq!(history.items[0].title, "Mi Portafolio");

        let _item2 = history.add(
            "Instagram".to_string(),
            "https://instagram.com/usuario".to_string(),
            "H".to_string(),
            "#4f46e5".to_string(),
        );
        assert_eq!(history.items.len(), 2);
        // Newest should be at index 0
        assert_eq!(history.items[0].title, "Instagram");

        // Remove item1
        history.remove(&item1.id);
        assert_eq!(history.items.len(), 1);
        assert_eq!(history.items[0].title, "Instagram");

        // Clear
        history.clear();
        assert_eq!(history.items.len(), 0);
    }

    #[test]
    fn test_history_serialization() {
        let mut history = QrHistory::default();
        history.add(
            "Test QR".to_string(),
            "https://test.com".to_string(),
            "Q".to_string(),
            "#000000".to_string(),
        );

        let json = serde_json::to_string(&history).unwrap();
        let decoded: QrHistory = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.items.len(), 1);
        assert_eq!(decoded.items[0].title, "Test QR");
        assert_eq!(decoded.items[0].ecc_level, "Q");
    }
}
