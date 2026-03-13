//! アプリケーションステート管理
//!
//! AIDEエディタのメインアプリケーションロジックを管理する。

use crate::buffer::{TextBuffer, History, history::EditCommand};
use crate::highlight::{SyntaxHighlighter, Language, theme::Theme};
use eframe::egui;

/// アプリケーションのメインステート
pub struct AideApp {
    /// 現在のテキストバッファ
    buffer: TextBuffer,
    /// 編集履歴
    history: History,
    /// 構文ハイライタ
    highlighter: SyntaxHighlighter,
    /// カラーテーマ
    theme: Theme,
    /// ステータスメッセージ
    status_message: String,
    /// 前回のテキスト（差分検出用）
    previous_text: String,
}

impl AideApp {
    /// 新しいアプリケーションインスタンスを作成する
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut highlighter = SyntaxHighlighter::new();
        // デフォルトでRust言語を設定
        let _ = highlighter.set_language(Language::Rust);

        Self {
            buffer: TextBuffer::new(),
            history: History::with_default_size(),
            highlighter,
            theme: Theme::dark(),
            status_message: String::from("AIDE - 準備完了"),
            previous_text: String::new(),
        }
    }

    /// ファイルを開く
    pub fn open_file(&mut self, path: &std::path::Path) {
        match TextBuffer::from_file(path) {
            Ok(buffer) => {
                self.buffer = buffer;
                self.history.clear();
                self.previous_text = self.buffer.to_string();

                // 拡張子から言語を設定
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if let Some(lang) = Language::from_extension(ext) {
                        let _ = self.highlighter.set_language(lang);
                    }
                }

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
        match self.buffer.save() {
            Ok(()) => {
                self.status_message = String::from("保存しました");
            }
            Err(e) => {
                self.status_message = format!("保存エラー: {}", e);
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
            match self.buffer.save_as(&path) {
                Ok(()) => {
                    // 拡張子から言語を設定
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if let Some(lang) = Language::from_extension(ext) {
                            let _ = self.highlighter.set_language(lang);
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

    /// 元に戻す
    fn undo(&mut self) {
        if let Some(inverse_command) = self.history.undo() {
            self.apply_command(&inverse_command);
            self.previous_text = self.buffer.to_string();
            self.status_message = String::from("元に戻しました");
        } else {
            self.status_message = String::from("元に戻す操作がありません");
        }
    }

    /// やり直し
    fn redo(&mut self) {
        if let Some(command) = self.history.redo() {
            self.apply_command(&command);
            self.previous_text = self.buffer.to_string();
            self.status_message = String::from("やり直しました");
        } else {
            self.status_message = String::from("やり直す操作がありません");
        }
    }

    /// コマンドを適用する
    fn apply_command(&mut self, command: &EditCommand) {
        match command {
            EditCommand::Insert { position, text } => {
                self.buffer.insert(*position, text);
            }
            EditCommand::Delete { range, .. } => {
                self.buffer.remove(range.clone());
            }
            EditCommand::Group { commands } => {
                for cmd in commands {
                    self.apply_command(cmd);
                }
            }
        }
    }

    /// テキスト変更を検出して履歴に記録する
    fn detect_and_record_changes(&mut self, new_text: &str) {
        if new_text == self.previous_text {
            return;
        }

        // 簡易的な差分検出
        // 実際のエディタでは、より効率的なアルゴリズムを使用する
        let old_chars: Vec<char> = self.previous_text.chars().collect();
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
            self.history.record(EditCommand::delete(
                common_prefix..old_end,
                deleted_text,
            ));
        }

        // 挿入された部分
        if common_prefix < new_end {
            let inserted_text: String = new_chars[common_prefix..new_end].iter().collect();
            self.history.record(EditCommand::insert(common_prefix, inserted_text));
        }

        self.previous_text = new_text.to_string();
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
                    if ui.button("開く (Ctrl+O)").clicked() {
                        self.show_open_dialog();
                        ui.close();
                    }
                    if ui.button("保存 (Ctrl+S)").clicked() {
                        if self.buffer.file_path().is_some() {
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
                    let can_undo = self.history.can_undo();
                    let can_redo = self.history.can_redo();

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

        // ステータスバー
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // ファイル情報
                    let lines = self.buffer.len_lines();
                    let chars = self.buffer.len_chars();

                    // 変更状態
                    let modified = if self.buffer.is_modified() { " [変更あり]" } else { "" };

                    // 言語
                    let lang = self.highlighter.current_language()
                        .map(|l| l.name())
                        .unwrap_or("なし");

                    ui.label(format!(
                        "{} | {} 行, {} 文字 | 言語: {}{}",
                        self.theme.name,
                        lines,
                        chars,
                        lang,
                        modified
                    ));
                });
            });
        });

        // メインエディター領域
        egui::CentralPanel::default().show(ctx, |ui| {
            // 背景色をテーマに合わせる
            let frame = egui::Frame::default()
                .fill(self.theme.background)
                .inner_margin(8.0);

            frame.show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // テキストエディット
                    let text = self.buffer.to_string();
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
                        self.detect_and_record_changes(&editable_text);
                        self.buffer = TextBuffer::from_str(&editable_text);
                    }
                });
            });
        });
    }
}

impl AideApp {
    /// キーボードショートカットを処理する
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        let modifiers = ctx.input(|i| i.modifiers);

        // Ctrl+O: ファイルを開く
        if modifiers.ctrl && ctx.input(|i| i.key_pressed(egui::Key::O)) {
            self.show_open_dialog();
        }

        // Ctrl+S: 保存
        if modifiers.ctrl && !modifiers.shift && ctx.input(|i| i.key_pressed(egui::Key::S)) {
            if self.buffer.file_path().is_some() {
                self.save_file();
            } else {
                self.show_save_dialog();
            }
        }

        // Ctrl+Shift+S: 名前を付けて保存
        if modifiers.ctrl && modifiers.shift && ctx.input(|i| i.key_pressed(egui::Key::S)) {
            self.show_save_dialog();
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
    }
}
