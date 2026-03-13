//! ステータスバー
//!
//! エディター下部のステータス表示を提供する。

use eframe::egui;

/// ステータスバーの情報
#[derive(Debug, Clone, Default)]
pub struct StatusInfo {
    /// メッセージ
    pub message: String,
    /// ファイル名
    pub file_name: Option<String>,
    /// 行数
    pub line_count: usize,
    /// 文字数
    pub char_count: usize,
    /// カーソル位置（行）
    pub cursor_line: usize,
    /// カーソル位置（列）
    pub cursor_column: usize,
    /// 言語名
    pub language: String,
    /// テーマ名
    pub theme: String,
    /// 変更状態
    pub modified: bool,
    /// エンコーディング
    pub encoding: String,
    /// 改行コード
    pub line_ending: String,
}

impl StatusInfo {
    /// 新しいステータス情報を作成する
    pub fn new() -> Self {
        Self {
            message: String::new(),
            file_name: None,
            line_count: 0,
            char_count: 0,
            cursor_line: 1,
            cursor_column: 1,
            language: "なし".to_string(),
            theme: "Dark".to_string(),
            modified: false,
            encoding: "UTF-8".to_string(),
            line_ending: "LF".to_string(),
        }
    }

    /// メッセージを設定する
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    /// ファイル情報を設定する
    pub fn set_file_info(&mut self, name: Option<String>, lines: usize, chars: usize, modified: bool) {
        self.file_name = name;
        self.line_count = lines;
        self.char_count = chars;
        self.modified = modified;
    }

    /// カーソル位置を設定する
    pub fn set_cursor(&mut self, line: usize, column: usize) {
        self.cursor_line = line;
        self.cursor_column = column;
    }

    /// 言語を設定する
    pub fn set_language(&mut self, language: impl Into<String>) {
        self.language = language.into();
    }

    /// テーマを設定する
    pub fn set_theme(&mut self, theme: impl Into<String>) {
        self.theme = theme.into();
    }
}

/// ステータスバー
pub struct StatusBar {
    /// 情報
    info: StatusInfo,
}

impl StatusBar {
    /// 新しいステータスバーを作成する
    pub fn new() -> Self {
        Self {
            info: StatusInfo::new(),
        }
    }

    /// 情報を取得する
    pub fn info(&self) -> &StatusInfo {
        &self.info
    }

    /// 情報を可変参照で取得する
    pub fn info_mut(&mut self) -> &mut StatusInfo {
        &mut self.info
    }

    /// 情報を設定する
    pub fn set_info(&mut self, info: StatusInfo) {
        self.info = info;
    }

    /// UIを描画する
    pub fn ui(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // 左側：メッセージ
            ui.label(&self.info.message);

            // 右側：ファイル情報
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // エンコーディングと改行コード
                ui.label(format!("{} | {}", self.info.encoding, self.info.line_ending));

                ui.separator();

                // カーソル位置
                ui.label(format!("行 {}, 列 {}", self.info.cursor_line, self.info.cursor_column));

                ui.separator();

                // 言語
                ui.label(format!("言語: {}", self.info.language));

                ui.separator();

                // ファイル情報
                let modified_mark = if self.info.modified { " ●" } else { "" };
                ui.label(format!(
                    "{} 行, {} 文字{}",
                    self.info.line_count, self.info.char_count, modified_mark
                ));

                ui.separator();

                // テーマ
                ui.label(&self.info.theme);
            });
        });
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_info_creation() {
        let info = StatusInfo::new();
        assert!(info.message.is_empty());
        assert!(info.file_name.is_none());
        assert_eq!(info.cursor_line, 1);
        assert_eq!(info.cursor_column, 1);
    }

    #[test]
    fn test_status_info_set_message() {
        let mut info = StatusInfo::new();
        info.set_message("テストメッセージ");
        assert_eq!(info.message, "テストメッセージ");
    }

    #[test]
    fn test_status_info_set_file_info() {
        let mut info = StatusInfo::new();
        info.set_file_info(Some("test.rs".to_string()), 100, 5000, true);

        assert_eq!(info.file_name, Some("test.rs".to_string()));
        assert_eq!(info.line_count, 100);
        assert_eq!(info.char_count, 5000);
        assert!(info.modified);
    }

    #[test]
    fn test_status_info_set_cursor() {
        let mut info = StatusInfo::new();
        info.set_cursor(10, 20);

        assert_eq!(info.cursor_line, 10);
        assert_eq!(info.cursor_column, 20);
    }

    #[test]
    fn test_status_bar_creation() {
        let bar = StatusBar::new();
        assert!(bar.info().message.is_empty());
    }

    #[test]
    fn test_status_bar_set_info() {
        let mut bar = StatusBar::new();
        let mut info = StatusInfo::new();
        info.set_message("新しいメッセージ");
        bar.set_info(info);

        assert_eq!(bar.info().message, "新しいメッセージ");
    }

    #[test]
    fn test_status_info_set_language() {
        let mut info = StatusInfo::new();
        info.set_language("Rust");
        assert_eq!(info.language, "Rust");
    }

    #[test]
    fn test_status_info_set_theme() {
        let mut info = StatusInfo::new();
        info.set_theme("Monokai");
        assert_eq!(info.theme, "Monokai");
    }
}
