//! ファイルエクスプローラー
//!
//! サイドバーのファイルツリー表示を提供する。

use eframe::egui;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// ファイルエントリ
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// ファイルパス
    pub path: PathBuf,
    /// ファイル名
    pub name: String,
    /// ディレクトリかどうか
    pub is_dir: bool,
    /// 子エントリ（ディレクトリの場合）
    pub children: Option<Vec<FileEntry>>,
}

impl FileEntry {
    /// パスからエントリを作成する
    pub fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_str()?.to_string();
        let is_dir = path.is_dir();

        Some(Self {
            path: path.to_path_buf(),
            name,
            is_dir,
            children: None,
        })
    }

    /// 子エントリを読み込む
    pub fn load_children(&mut self) -> std::io::Result<()> {
        if !self.is_dir {
            return Ok(());
        }

        let mut children = Vec::new();
        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            let path = entry.path();

            // 隠しファイルをスキップ
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }

            if let Some(file_entry) = FileEntry::from_path(&path) {
                children.push(file_entry);
            }
        }

        // ディレクトリを先に、その後ファイルをアルファベット順にソート
        children.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        self.children = Some(children);
        Ok(())
    }

    /// アイコンを取得する
    pub fn icon(&self) -> &'static str {
        if self.is_dir {
            "📁"
        } else {
            // 拡張子に応じたアイコン
            match self.path.extension().and_then(|e| e.to_str()) {
                Some("rs") => "🦀",
                Some("toml") => "📋",
                Some("md") => "📝",
                Some("txt") => "📄",
                Some("json") => "📊",
                Some("yml") | Some("yaml") => "⚙️",
                Some("sh") | Some("bash") => "🐚",
                Some("py") => "🐍",
                Some("js") | Some("ts") => "📜",
                _ => "📄",
            }
        }
    }
}

/// ファイルエクスプローラー
pub struct FileExplorer {
    /// ルートディレクトリ
    root: Option<FileEntry>,
    /// 展開されたディレクトリ
    expanded: HashSet<PathBuf>,
    /// 選択されたファイル
    selected: Option<PathBuf>,
    /// 表示状態
    visible: bool,
    /// 幅
    width: f32,
}

impl FileExplorer {
    /// 新しいファイルエクスプローラーを作成する
    pub fn new() -> Self {
        Self {
            root: None,
            expanded: HashSet::new(),
            selected: None,
            visible: true,
            width: 200.0,
        }
    }

    /// ルートディレクトリを設定する
    pub fn set_root(&mut self, path: &Path) -> std::io::Result<()> {
        let mut root = FileEntry::from_path(path)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "パスが見つかりません"))?;
        root.load_children()?;
        self.expanded.insert(path.to_path_buf());
        self.root = Some(root);
        Ok(())
    }

    /// 表示状態を切り替える
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// 表示状態を取得する
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// 選択されたファイルを取得する
    pub fn selected(&self) -> Option<&PathBuf> {
        self.selected.as_ref()
    }

    /// リフレッシュする
    pub fn refresh(&mut self) -> std::io::Result<()> {
        if let Some(ref mut root) = self.root {
            root.load_children()?;
            // 展開されたディレクトリも再読み込み
            Self::reload_expanded_recursive(&self.expanded, root)?;
        }
        Ok(())
    }

    /// 展開されたディレクトリを再読み込みする（再帰ヘルパー）
    fn reload_expanded_recursive(expanded: &HashSet<PathBuf>, entry: &mut FileEntry) -> std::io::Result<()> {
        if let Some(ref mut children) = entry.children {
            for child in children.iter_mut() {
                if child.is_dir && expanded.contains(&child.path) {
                    child.load_children()?;
                    Self::reload_expanded_recursive(expanded, child)?;
                }
            }
        }
        Ok(())
    }

    /// UIを描画する（選択されたファイルパスを返す）
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<PathBuf> {
        if !self.visible {
            return None;
        }

        let mut selected_file = None;
        let mut expand_changes: Vec<(PathBuf, bool)> = Vec::new();
        let mut new_selection: Option<PathBuf> = None;

        if let Some(ref mut root) = self.root {
            egui::ScrollArea::vertical().show(ui, |ui| {
                selected_file = Self::render_entry_recursive(
                    ui,
                    root,
                    &self.expanded,
                    &self.selected,
                    &mut expand_changes,
                    &mut new_selection,
                );
            });
        } else {
            ui.label("フォルダを開いてください");
        }

        // 展開状態を更新
        for (path, expand) in expand_changes {
            if expand {
                self.expanded.insert(path);
            } else {
                self.expanded.remove(&path);
            }
        }

        // 選択状態を更新
        if let Some(path) = new_selection {
            self.selected = Some(path);
        }

        selected_file
    }

    /// エントリを描画する（再帰ヘルパー）
    fn render_entry_recursive(
        ui: &mut egui::Ui,
        entry: &mut FileEntry,
        expanded: &HashSet<PathBuf>,
        selected: &Option<PathBuf>,
        expand_changes: &mut Vec<(PathBuf, bool)>,
        new_selection: &mut Option<PathBuf>,
    ) -> Option<PathBuf> {
        let mut selected_file = None;

        if entry.is_dir {
            let is_expanded = expanded.contains(&entry.path);
            let icon = if is_expanded { "📂" } else { "📁" };

            let header = egui::CollapsingHeader::new(format!("{} {}", icon, entry.name))
                .default_open(is_expanded)
                .show(ui, |ui| {
                    // 子エントリがまだ読み込まれていない場合は読み込む
                    if entry.children.is_none() {
                        let _ = entry.load_children();
                    }

                    if let Some(ref mut children) = entry.children {
                        for child in children.iter_mut() {
                            if let Some(path) = Self::render_entry_recursive(
                                ui,
                                child,
                                expanded,
                                selected,
                                expand_changes,
                                new_selection,
                            ) {
                                selected_file = Some(path);
                            }
                        }
                    }
                });

            // 展開状態の変更を記録
            if header.header_response.clicked() {
                expand_changes.push((entry.path.clone(), !is_expanded));
            }
        } else {
            let icon = entry.icon();
            let is_selected = selected.as_ref() == Some(&entry.path);

            let label = if is_selected {
                egui::Label::new(
                    egui::RichText::new(format!("{} {}", icon, entry.name))
                        .background_color(egui::Color32::from_rgb(60, 60, 100)),
                )
            } else {
                egui::Label::new(format!("{} {}", icon, entry.name))
            };

            let response = ui.add(label.sense(egui::Sense::click()));

            if response.clicked() {
                *new_selection = Some(entry.path.clone());
                selected_file = Some(entry.path.clone());
            }

            // ダブルクリックでも選択
            if response.double_clicked() {
                selected_file = Some(entry.path.clone());
            }
        }

        selected_file
    }
}

impl Default for FileExplorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_file_entry_from_path() {
        let current_dir = env::current_dir().unwrap();
        let entry = FileEntry::from_path(&current_dir);
        assert!(entry.is_some());

        let entry = entry.unwrap();
        assert!(entry.is_dir);
    }

    #[test]
    fn test_file_entry_icon() {
        let rs_file = FileEntry {
            path: PathBuf::from("/test/file.rs"),
            name: "file.rs".to_string(),
            is_dir: false,
            children: None,
        };
        assert_eq!(rs_file.icon(), "🦀");

        let dir = FileEntry {
            path: PathBuf::from("/test/dir"),
            name: "dir".to_string(),
            is_dir: true,
            children: None,
        };
        assert_eq!(dir.icon(), "📁");
    }

    #[test]
    fn test_file_explorer_creation() {
        let explorer = FileExplorer::new();
        assert!(explorer.is_visible());
        assert!(explorer.root.is_none());
    }

    #[test]
    fn test_file_explorer_toggle() {
        let mut explorer = FileExplorer::new();
        assert!(explorer.is_visible());

        explorer.toggle();
        assert!(!explorer.is_visible());

        explorer.toggle();
        assert!(explorer.is_visible());
    }

    #[test]
    fn test_file_explorer_set_root() {
        let mut explorer = FileExplorer::new();
        let current_dir = env::current_dir().unwrap();

        let result = explorer.set_root(&current_dir);
        assert!(result.is_ok());
        assert!(explorer.root.is_some());
    }
}
