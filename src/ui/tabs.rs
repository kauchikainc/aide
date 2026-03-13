//! タブバー
//!
//! 複数ファイルを切り替えるためのタブバーを提供する。

use eframe::egui;
use std::path::PathBuf;

/// タブの情報
#[derive(Debug, Clone)]
pub struct Tab {
    /// タブID
    pub id: usize,
    /// ファイルパス（未保存の場合はNone）
    pub path: Option<PathBuf>,
    /// 表示名
    pub title: String,
    /// 変更フラグ
    pub modified: bool,
}

impl Tab {
    /// 新しいタブを作成する
    pub fn new(id: usize) -> Self {
        Self {
            id,
            path: None,
            title: format!("新規 {}", id),
            modified: false,
        }
    }

    /// ファイルパスからタブを作成する
    pub fn from_path(id: usize, path: PathBuf) -> Self {
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("不明")
            .to_string();

        Self {
            id,
            path: Some(path),
            title,
            modified: false,
        }
    }

    /// 表示用のタイトルを取得する（変更マーク付き）
    pub fn display_title(&self) -> String {
        if self.modified {
            format!("● {}", self.title)
        } else {
            self.title.clone()
        }
    }
}

/// タブバーの状態
pub struct TabBar {
    /// タブリスト
    tabs: Vec<Tab>,
    /// アクティブなタブのインデックス
    active_index: usize,
    /// 次のタブID
    next_id: usize,
}

impl TabBar {
    /// 新しいタブバーを作成する
    pub fn new() -> Self {
        let mut bar = Self {
            tabs: Vec::new(),
            active_index: 0,
            next_id: 1,
        };
        // 初期タブを追加
        bar.add_tab();
        bar
    }

    /// タブを追加する
    pub fn add_tab(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let tab = Tab::new(id);
        self.tabs.push(tab);
        self.active_index = self.tabs.len() - 1;
        id
    }

    /// ファイルからタブを追加する
    pub fn add_tab_from_path(&mut self, path: PathBuf) -> usize {
        // 既存のタブを確認
        if let Some(index) = self.find_tab_by_path(&path) {
            self.active_index = index;
            return self.tabs[index].id;
        }

        let id = self.next_id;
        self.next_id += 1;
        let tab = Tab::from_path(id, path);
        self.tabs.push(tab);
        self.active_index = self.tabs.len() - 1;
        id
    }

    /// パスからタブを検索する
    fn find_tab_by_path(&self, path: &PathBuf) -> Option<usize> {
        self.tabs.iter().position(|t| t.path.as_ref() == Some(path))
    }

    /// タブを閉じる
    pub fn close_tab(&mut self, index: usize) -> bool {
        if self.tabs.len() <= 1 {
            return false; // 最後のタブは閉じない
        }

        if index < self.tabs.len() {
            self.tabs.remove(index);
            if self.active_index >= self.tabs.len() {
                self.active_index = self.tabs.len() - 1;
            } else if self.active_index > index {
                self.active_index -= 1;
            }
            true
        } else {
            false
        }
    }

    /// アクティブなタブを閉じる
    pub fn close_active_tab(&mut self) -> bool {
        self.close_tab(self.active_index)
    }

    /// アクティブなタブのインデックスを取得する
    pub fn active_index(&self) -> usize {
        self.active_index
    }

    /// アクティブなタブを取得する
    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_index)
    }

    /// アクティブなタブを可変参照で取得する
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_index)
    }

    /// タブを選択する
    pub fn select_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_index = index;
        }
    }

    /// 次のタブを選択する
    pub fn select_next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_index = (self.active_index + 1) % self.tabs.len();
        }
    }

    /// 前のタブを選択する
    pub fn select_prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            if self.active_index == 0 {
                self.active_index = self.tabs.len() - 1;
            } else {
                self.active_index -= 1;
            }
        }
    }

    /// タブ数を取得する
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// タブが空かどうか
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// すべてのタブを取得する
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// 変更されたタブがあるかどうか
    pub fn has_modified_tabs(&self) -> bool {
        self.tabs.iter().any(|t| t.modified)
    }

    /// UIを描画する（アクションを返す）
    pub fn ui(&mut self, ui: &mut egui::Ui) -> TabBarAction {
        let mut action = TabBarAction::None;

        ui.horizontal(|ui| {
            // タブを描画
            let mut close_index = None;

            for (index, tab) in self.tabs.iter().enumerate() {
                let is_active = index == self.active_index;
                let title = tab.display_title();

                // タブボタン
                let button = if is_active {
                    egui::Button::new(&title).fill(egui::Color32::from_rgb(60, 60, 70))
                } else {
                    egui::Button::new(&title)
                };

                let response = ui.add(button);

                if response.clicked() {
                    action = TabBarAction::Select(index);
                }

                // 右クリックでコンテキストメニュー
                response.context_menu(|ui| {
                    if ui.button("閉じる").clicked() {
                        close_index = Some(index);
                        ui.close();
                    }
                    if ui.button("他を閉じる").clicked() {
                        action = TabBarAction::CloseOthers(index);
                        ui.close();
                    }
                });

                // 中クリックで閉じる
                if response.middle_clicked() {
                    close_index = Some(index);
                }
            }

            // 閉じるアクション
            if let Some(index) = close_index {
                action = TabBarAction::Close(index);
            }

            // 新規タブボタン
            if ui.button("+").clicked() {
                action = TabBarAction::New;
            }
        });

        action
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}

/// タブバーのアクション
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabBarAction {
    /// アクションなし
    None,
    /// タブを選択
    Select(usize),
    /// タブを閉じる
    Close(usize),
    /// 他のタブを閉じる
    CloseOthers(usize),
    /// 新規タブ
    New,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_creation() {
        let tab = Tab::new(1);
        assert_eq!(tab.id, 1);
        assert!(tab.path.is_none());
        assert!(!tab.modified);
    }

    #[test]
    fn test_tab_from_path() {
        let path = PathBuf::from("/test/file.rs");
        let tab = Tab::from_path(1, path.clone());
        assert_eq!(tab.id, 1);
        assert_eq!(tab.path, Some(path));
        assert_eq!(tab.title, "file.rs");
    }

    #[test]
    fn test_tab_display_title() {
        let mut tab = Tab::new(1);
        tab.title = "test.rs".to_string();
        assert_eq!(tab.display_title(), "test.rs");

        tab.modified = true;
        assert_eq!(tab.display_title(), "● test.rs");
    }

    #[test]
    fn test_tab_bar_creation() {
        let bar = TabBar::new();
        assert_eq!(bar.len(), 1);
        assert_eq!(bar.active_index(), 0);
    }

    #[test]
    fn test_tab_bar_add_tab() {
        let mut bar = TabBar::new();
        assert_eq!(bar.len(), 1);

        bar.add_tab();
        assert_eq!(bar.len(), 2);
        assert_eq!(bar.active_index(), 1);
    }

    #[test]
    fn test_tab_bar_close_tab() {
        let mut bar = TabBar::new();
        bar.add_tab();
        bar.add_tab();
        assert_eq!(bar.len(), 3);

        bar.close_tab(1);
        assert_eq!(bar.len(), 2);
    }

    #[test]
    fn test_tab_bar_cannot_close_last_tab() {
        let mut bar = TabBar::new();
        assert_eq!(bar.len(), 1);

        let result = bar.close_tab(0);
        assert!(!result);
        assert_eq!(bar.len(), 1);
    }

    #[test]
    fn test_tab_bar_select_tab() {
        let mut bar = TabBar::new();
        bar.add_tab();
        bar.add_tab();

        bar.select_tab(0);
        assert_eq!(bar.active_index(), 0);

        bar.select_tab(2);
        assert_eq!(bar.active_index(), 2);
    }

    #[test]
    fn test_tab_bar_select_next_prev() {
        let mut bar = TabBar::new();
        bar.add_tab();
        bar.add_tab();
        bar.select_tab(0);

        bar.select_next_tab();
        assert_eq!(bar.active_index(), 1);

        bar.select_next_tab();
        assert_eq!(bar.active_index(), 2);

        bar.select_next_tab();
        assert_eq!(bar.active_index(), 0); // ラップアラウンド

        bar.select_prev_tab();
        assert_eq!(bar.active_index(), 2); // ラップアラウンド
    }

    #[test]
    fn test_tab_bar_has_modified_tabs() {
        let mut bar = TabBar::new();
        assert!(!bar.has_modified_tabs());

        if let Some(tab) = bar.active_tab_mut() {
            tab.modified = true;
        }
        assert!(bar.has_modified_tabs());
    }

    #[test]
    fn test_tab_bar_add_tab_from_path() {
        let mut bar = TabBar::new();
        let path = PathBuf::from("/test/file.rs");

        let id1 = bar.add_tab_from_path(path.clone());
        assert_eq!(bar.len(), 2);

        // 同じパスで追加しても重複しない
        let id2 = bar.add_tab_from_path(path);
        assert_eq!(bar.len(), 2);
        assert_eq!(id1, id2);
    }
}
