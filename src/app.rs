//! アプリケーションステート管理
//!
//! AIDEエディタのメインアプリケーションロジックを管理する。

use crate::buffer::{TextBuffer, History, history::EditCommand};
use crate::highlight::{SyntaxHighlighter, Language, theme::Theme};
use crate::terminal::TerminalView;
use crate::ui::{TabBar, TabBarAction, FileExplorer, StatusBar};
use eframe::egui;
use std::path::PathBuf;

/// タブごとのエディター状態
struct EditorTab {
    /// テキストバッファ
    buffer: TextBuffer,
    /// 編集履歴
    history: History,
    /// 構文ハイライタ
    highlighter: SyntaxHighlighter,
    /// 前回のテキスト（差分検出用）
    previous_text: String,
}

impl EditorTab {
    /// 新しいタブを作成する
    fn new() -> Self {
        let mut highlighter = SyntaxHighlighter::new();
        let _ = highlighter.set_language(Language::Rust);
        Self {
            buffer: TextBuffer::new(),
            history: History::with_default_size(),
            highlighter,
            previous_text: String::new(),
        }
    }

    /// ファイルからタブを作成する
    fn from_path(path: &std::path::Path) -> std::io::Result<Self> {
        let buffer = TextBuffer::from_file(path)?;
        let mut highlighter = SyntaxHighlighter::new();

        // 拡張子から言語を設定
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if let Some(lang) = Language::from_extension(ext) {
                let _ = highlighter.set_language(lang);
            }
        }

        Ok(Self {
            previous_text: buffer.to_string(),
            buffer,
            history: History::with_default_size(),
            highlighter,
        })
    }
}

/// アプリケーションのメインステート
pub struct AideApp {
    /// タブごとのエディター状態
    tabs: Vec<EditorTab>,
    /// タブバー
    tab_bar: TabBar,
    /// ファイルエクスプローラー
    file_explorer: FileExplorer,
    /// ステータスバー
    status_bar: StatusBar,
    /// カラーテーマ
    theme: Theme,
    /// ステータスメッセージ
    status_message: String,
    /// ターミナルビュー
    terminal: TerminalView,
}

impl AideApp {
    /// 新しいアプリケーションインスタンスを作成する
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // 初期タブを作成
        let initial_tab = EditorTab::new();

        Self {
            tabs: vec![initial_tab],
            tab_bar: TabBar::new(),
            file_explorer: FileExplorer::new(),
            status_bar: StatusBar::new(),
            theme: Theme::dark(),
            status_message: String::from("AIDE - 準備完了"),
            terminal: TerminalView::new(),
        }
    }

    /// 現在のタブを取得する
    fn current_tab(&self) -> Option<&EditorTab> {
        self.tabs.get(self.tab_bar.active_index())
    }

    /// 現在のタブを可変参照で取得する
    fn current_tab_mut(&mut self) -> Option<&mut EditorTab> {
        let index = self.tab_bar.active_index();
        self.tabs.get_mut(index)
    }

    /// ファイルを開く（新しいタブで）
    pub fn open_file(&mut self, path: &std::path::Path) {
        match EditorTab::from_path(path) {
            Ok(tab) => {
                // タブバーに追加
                self.tab_bar.add_tab_from_path(path.to_path_buf());
                self.tabs.push(tab);

                self.status_message = format!(
                    "開きました: {}",
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("不明")
                );
            }
            Err(e) => {
                self.status_message = format!("エラー: {}", e);
            }
        }
    }

    /// ファイルを保存する
    pub fn save_file(&mut self) {
        if let Some(tab) = self.current_tab_mut() {
            match tab.buffer.save() {
                Ok(()) => {
                    // タブの変更フラグを更新
                    if let Some(ui_tab) = self.tab_bar.active_tab_mut() {
                        ui_tab.modified = false;
                    }
                    self.status_message = String::from("保存しました");
                }
                Err(e) => {
                    self.status_message = format!("保存エラー: {}", e);
                }
            }
        }
    }

    /// ファイルダイアログを開いてファイルを選択する
    fn show_open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Rustファイル", &["rs"])
            .add_filter("すべてのファイル", &["*"])
            .pick_file()
        {
            self.open_file(&path);
        }
    }

    /// ファイルダイアログを開いて保存先を選択する
    fn show_save_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Rustファイル", &["rs"])
            .add_filter("すべてのファイル", &["*"])
            .save_file()
        {
            if let Some(tab) = self.current_tab_mut() {
                match tab.buffer.save_as(&path) {
                    Ok(()) => {
                        // 拡張子から言語を設定
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if let Some(lang) = Language::from_extension(ext) {
                                let _ = tab.highlighter.set_language(lang);
                            }
                        }
                        self.status_message = format!(
                            "保存しました: {}",
                            path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("不明")
                        );
                    }
                    Err(e) => {
                        self.status_message = format!("保存エラー: {}", e);
                    }
                }
            }
        }
    }

    /// 元に戻す
    fn undo(&mut self) {
        if let Some(tab) = self.current_tab_mut() {
            if let Some(inverse_command) = tab.history.undo() {
                Self::apply_command_to_tab(tab, &inverse_command);
                tab.previous_text = tab.buffer.to_string();
                self.status_message = String::from("元に戻しました");
            } else {
                self.status_message = String::from("元に戻す操作がありません");
            }
        }
    }

    /// やり直し
    fn redo(&mut self) {
        if let Some(tab) = self.current_tab_mut() {
            if let Some(command) = tab.history.redo() {
                Self::apply_command_to_tab(tab, &command);
                tab.previous_text = tab.buffer.to_string();
                self.status_message = String::from("やり直しました");
            } else {
                self.status_message = String::from("やり直す操作がありません");
            }
        }
    }

    /// コマンドをタブに適用する
    fn apply_command_to_tab(tab: &mut EditorTab, command: &EditCommand) {
        match command {
            EditCommand::Insert { position, text } => {
                tab.buffer.insert(*position, text);
            }
            EditCommand::Delete { range, .. } => {
                tab.buffer.remove(range.clone());
            }
            EditCommand::Group { commands } => {
                for cmd in commands {
                    Self::apply_command_to_tab(tab, cmd);
                }
            }
        }
    }

    /// テキスト変更を検出して履歴に記録する
    fn detect_and_record_changes(tab: &mut EditorTab, new_text: &str) {
        if new_text == tab.previous_text {
            return;
        }

        // 簡易的な差分検出
        let old_chars: Vec<char> = tab.previous_text.chars().collect();
        let new_chars: Vec<char> = new_text.chars().collect();

        // 先頭から一致する部分を探す
        let common_prefix = old_chars
            .iter()
            .zip(new_chars.iter())
            .take_while(|(a, b)| a == b)
            .count();

        // 末尾から一致する部分を探す
        let common_suffix = old_chars
            .iter()
            .rev()
            .zip(new_chars.iter().rev())
            .take_while(|(a, b)| a == b)
            .take(old_chars.len().saturating_sub(common_prefix))
            .take(new_chars.len().saturating_sub(common_prefix))
            .count();

        let old_end = old_chars.len().saturating_sub(common_suffix);
        let new_end = new_chars.len().saturating_sub(common_suffix);

        // 削除された部分
        if common_prefix < old_end {
            let deleted_text: String = old_chars[common_prefix..old_end].iter().collect();
            tab.history.record(EditCommand::delete(
                common_prefix..old_end,
                deleted_text,
            ));
        }

        // 挿入された部分
        if common_prefix < new_end {
            let inserted_text: String = new_chars[common_prefix..new_end].iter().collect();
            tab.history.record(EditCommand::insert(common_prefix, inserted_text));
        }

        tab.previous_text = new_text.to_string();
    }

    /// タブを閉じる
    fn close_tab(&mut self, index: usize) {
        if self.tabs.len() > 1 && index < self.tabs.len() {
            self.tabs.remove(index);
            self.tab_bar.close_tab(index);
            self.status_message = String::from("タブを閉じました");
        }
    }

    /// 他のタブを閉じる
    fn close_other_tabs(&mut self, keep_index: usize) {
        if keep_index < self.tabs.len() {
            let tab = self.tabs.remove(keep_index);
            self.tabs.clear();
            self.tabs.push(tab);

            // タブバーも更新
            self.tab_bar = TabBar::new();
            if let Some(path) = self.tabs[0].buffer.file_path() {
                self.tab_bar.add_tab_from_path(path.to_path_buf());
            }
            self.status_message = String::from("他のタブを閉じました");
        }
    }

    /// ファイルエクスプローラーでファイルを選択した場合
    fn handle_file_selection(&mut self, path: PathBuf) {
        if path.is_file() {
            self.open_file(&path);
        }
    }
}

impl eframe::App for AideApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // キーボードショートカットの処理
        self.handle_keyboard_shortcuts(ctx);

        // メニューバー
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("ファイル", |ui| {
                    if ui.button("新規タブ (Ctrl+N)").clicked() {
                        self.tabs.push(EditorTab::new());
                        self.tab_bar.add_tab();
                        ui.close();
                    }
                    if ui.button("開く (Ctrl+O)").clicked() {
                        self.show_open_dialog();
                        ui.close();
                    }
                    if ui.button("フォルダを開く").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            let _ = self.file_explorer.set_root(&path);
                        }
                        ui.close();
                    }
                    ui.separator();
                    let has_path = self.current_tab().map_or(false, |t| t.buffer.file_path().is_some());
                    if ui.button("保存 (Ctrl+S)").clicked() {
                        if has_path {
                            self.save_file();
                        } else {
                            self.show_save_dialog();
                        }
                        ui.close();
                    }
                    if ui.button("名前を付けて保存 (Ctrl+Shift+S)").clicked() {
                        self.show_save_dialog();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("終了").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("編集", |ui| {
                    let can_undo = self.current_tab().map_or(false, |t| t.history.can_undo());
                    let can_redo = self.current_tab().map_or(false, |t| t.history.can_redo());

                    if ui.add_enabled(can_undo, egui::Button::new("元に戻す (Ctrl+Z)")).clicked() {
                        self.undo();
                        ui.close();
                    }
                    if ui.add_enabled(can_redo, egui::Button::new("やり直し (Ctrl+Y)")).clicked() {
                        self.redo();
                        ui.close();
                    }
                });
                ui.menu_button("表示", |ui| {
                    ui.menu_button("テーマ", |ui| {
                        if ui.button("ダーク").clicked() {
                            self.theme = Theme::dark();
                            ui.close();
                        }
                        if ui.button("ライト").clicked() {
                            self.theme = Theme::light();
                            ui.close();
                        }
                        if ui.button("Monokai").clicked() {
                            self.theme = Theme::monokai();
                            ui.close();
                        }
                    });
                    ui.separator();
                    let sidebar_label = if self.file_explorer.is_visible() {
                        "サイドバーを隠す (Ctrl+B)"
                    } else {
                        "サイドバーを表示 (Ctrl+B)"
                    };
                    if ui.button(sidebar_label).clicked() {
                        self.file_explorer.toggle();
                        ui.close();
                    }
                    ui.separator();
                    let terminal_label = if self.terminal.is_visible() {
                        "ターミナルを隠す (Ctrl+`)"
                    } else {
                        "ターミナルを表示 (Ctrl+`)"
                    };
                    if ui.button(terminal_label).clicked() {
                        self.terminal.toggle();
                        ui.close();
                    }
                });
                ui.menu_button("ヘルプ", |ui| {
                    if ui.button("AIDEについて").clicked() {
                        self.status_message = String::from(
                            "AIDE - AI Development Editor v0.1.0 - Rust製軽量エディター"
                        );
                        ui.close();
                    }
                });
            });
        });

        // タブバー
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            let action = self.tab_bar.ui(ui);
            match action {
                TabBarAction::Select(index) => {
                    self.tab_bar.select_tab(index);
                }
                TabBarAction::Close(index) => {
                    self.close_tab(index);
                }
                TabBarAction::CloseOthers(index) => {
                    self.close_other_tabs(index);
                }
                TabBarAction::New => {
                    self.tabs.push(EditorTab::new());
                    self.tab_bar.add_tab();
                }
                TabBarAction::None => {}
            }
        });

        // ステータスバー
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(tab) = self.current_tab() {
                        // ファイル情報
                        let lines = tab.buffer.len_lines();
                        let chars = tab.buffer.len_chars();

                        // 変更状態
                        let modified = if tab.buffer.is_modified() { " [変更あり]" } else { "" };

                        // 言語
                        let lang = tab.highlighter.current_language()
                            .map(|l| l.name())
                            .unwrap_or("なし");

                        // ターミナル状態
                        let terminal_status = if self.terminal.is_visible() { " | ターミナル: 表示中" } else { "" };

                        ui.label(format!(
                            "{} | {} 行, {} 文字 | 言語: {}{}{}",
                            self.theme.name,
                            lines,
                            chars,
                            lang,
                            modified,
                            terminal_status
                        ));
                    }
                });
            });
        });

        // ターミナルパネル（下部、表示されている場合）
        if self.terminal.is_visible() {
            egui::TopBottomPanel::bottom("terminal_panel")
                .resizable(true)
                .min_height(100.0)
                .default_height(200.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("ターミナル");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("×").clicked() {
                                self.terminal.toggle();
                            }
                        });
                    });
                    ui.separator();
                    self.terminal.ui(ui);
                });
        }

        // サイドバー（ファイルエクスプローラー）
        if self.file_explorer.is_visible() {
            egui::SidePanel::left("file_explorer")
                .resizable(true)
                .min_width(150.0)
                .default_width(200.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("エクスプローラー");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🔄").on_hover_text("更新").clicked() {
                                let _ = self.file_explorer.refresh();
                            }
                        });
                    });
                    ui.separator();
                    if let Some(path) = self.file_explorer.ui(ui) {
                        self.handle_file_selection(path);
                    }
                });
        }

        // メインエディター領域
        egui::CentralPanel::default().show(ctx, |ui| {
            // 背景色をテーマに合わせる
            let frame = egui::Frame::default()
                .fill(self.theme.background)
                .inner_margin(8.0);

            frame.show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // 現在のタブのテキストを取得
                    let active_index = self.tab_bar.active_index();
                    if active_index < self.tabs.len() {
                        let tab = &mut self.tabs[active_index];

                        // テキストエディット
                        let text = tab.buffer.to_string();
                        let mut editable_text = text.clone();

                        // カスタムスタイルを適用
                        let text_color = self.theme.foreground;

                        let response = ui.add(
                            egui::TextEdit::multiline(&mut editable_text)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(30)
                                .text_color(text_color)
                                .code_editor(),
                        );

                        // 変更があった場合
                        if response.changed() {
                            Self::detect_and_record_changes(tab, &editable_text);
                            tab.buffer = TextBuffer::from_str(&editable_text);

                            // タブの変更フラグを更新
                            if let Some(ui_tab) = self.tab_bar.active_tab_mut() {
                                ui_tab.modified = tab.buffer.is_modified();
                            }
                        }
                    }
                });
            });
        });

        // ターミナルが表示中なら定期的に再描画
        if self.terminal.is_visible() {
            ctx.request_repaint();
        }
    }
}

impl AideApp {
    /// キーボードショートカットを処理する
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        let modifiers = ctx.input(|i| i.modifiers);

        // Ctrl+N: 新規タブ
        if modifiers.ctrl && ctx.input(|i| i.key_pressed(egui::Key::N)) {
            self.tabs.push(EditorTab::new());
            self.tab_bar.add_tab();
        }

        // Ctrl+O: ファイルを開く
        if modifiers.ctrl && ctx.input(|i| i.key_pressed(egui::Key::O)) {
            self.show_open_dialog();
        }

        // Ctrl+S: 保存
        if modifiers.ctrl && !modifiers.shift && ctx.input(|i| i.key_pressed(egui::Key::S)) {
            let has_path = self.current_tab().map_or(false, |t| t.buffer.file_path().is_some());
            if has_path {
                self.save_file();
            } else {
                self.show_save_dialog();
            }
        }

        // Ctrl+Shift+S: 名前を付けて保存
        if modifiers.ctrl && modifiers.shift && ctx.input(|i| i.key_pressed(egui::Key::S)) {
            self.show_save_dialog();
        }

        // Ctrl+W: タブを閉じる
        if modifiers.ctrl && ctx.input(|i| i.key_pressed(egui::Key::W)) {
            let index = self.tab_bar.active_index();
            self.close_tab(index);
        }

        // Ctrl+Tab: 次のタブ
        if modifiers.ctrl && ctx.input(|i| i.key_pressed(egui::Key::Tab)) {
            self.tab_bar.select_next_tab();
        }

        // Ctrl+Shift+Tab: 前のタブ
        if modifiers.ctrl && modifiers.shift && ctx.input(|i| i.key_pressed(egui::Key::Tab)) {
            self.tab_bar.select_prev_tab();
        }

        // Ctrl+Z: 元に戻す
        if modifiers.ctrl && !modifiers.shift && ctx.input(|i| i.key_pressed(egui::Key::Z)) {
            self.undo();
        }

        // Ctrl+Y または Ctrl+Shift+Z: やり直し
        if modifiers.ctrl && ctx.input(|i| i.key_pressed(egui::Key::Y)) {
            self.redo();
        }
        if modifiers.ctrl && modifiers.shift && ctx.input(|i| i.key_pressed(egui::Key::Z)) {
            self.redo();
        }

        // Ctrl+B: サイドバー切り替え
        if modifiers.ctrl && ctx.input(|i| i.key_pressed(egui::Key::B)) {
            self.file_explorer.toggle();
        }

        // Ctrl+`: ターミナル切り替え
        if modifiers.ctrl && ctx.input(|i| i.key_pressed(egui::Key::Backtick)) {
            self.terminal.toggle();
        }
    }
}
