//! ターミナルUI
//!
//! ターミナル出力の描画と入力処理を行う。
//! 入力欄はEnterで改行、Ctrl+Enterでシェルに送信。

use super::pty::{PtyError, PtyManager, TerminalSize};
use eframe::egui;

/// ターミナルの色設定
#[derive(Debug, Clone)]
pub struct TerminalColors {
    /// 背景色
    pub background: egui::Color32,
    /// 前景色（デフォルト）
    pub foreground: egui::Color32,
    /// カーソル色
    pub cursor: egui::Color32,
    /// 選択色
    pub selection: egui::Color32,
}

impl Default for TerminalColors {
    fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(30, 30, 30),
            foreground: egui::Color32::from_rgb(204, 204, 204),
            cursor: egui::Color32::from_rgb(255, 255, 255),
            selection: egui::Color32::from_rgba_unmultiplied(100, 100, 200, 100),
        }
    }
}

/// ターミナルビュー
pub struct TerminalView {
    /// PTYマネージャー
    pty: PtyManager,
    /// 入力バッファ（複数行対応）
    input_buffer: String,
    /// 出力テキスト
    output_text: String,
    /// 色設定
    colors: TerminalColors,
    /// 表示状態
    visible: bool,
    /// エラーメッセージ
    error_message: Option<String>,
    /// 履歴（送信したコマンド）
    history: Vec<String>,
    /// 履歴の現在位置
    history_index: usize,
}

impl TerminalView {
    /// 新しいターミナルビューを作成する
    pub fn new() -> Self {
        Self {
            pty: PtyManager::new(),
            input_buffer: String::new(),
            output_text: String::new(),
            colors: TerminalColors::default(),
            visible: false,
            error_message: None,
            history: Vec::new(),
            history_index: 0,
        }
    }

    /// ターミナルを起動する
    pub fn start(&mut self) -> Result<(), PtyError> {
        self.pty.spawn_shell()?;
        self.visible = true;
        self.error_message = None;
        self.output_text.clear();
        self.output_text.push_str("シェルを起動しました。\n");
        self.output_text.push_str("Ctrl+Enter で入力を送信します。\n\n");
        Ok(())
    }

    /// ターミナルを停止する
    pub fn stop(&mut self) {
        self.pty.kill();
        self.visible = false;
    }

    /// 表示状態を切り替える
    pub fn toggle(&mut self) {
        if self.visible {
            self.visible = false;
        } else {
            if !self.pty.is_running() {
                if let Err(e) = self.start() {
                    self.error_message = Some(format!("ターミナルの起動に失敗しました: {}", e));
                }
            }
            self.visible = true;
        }
    }

    /// 表示状態を取得する
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// 入力を送信する
    fn send_input(&mut self) {
        if self.input_buffer.is_empty() {
            return;
        }

        let input = self.input_buffer.clone();

        // 履歴に追加（空でない場合）
        if !input.trim().is_empty() {
            self.history.push(input.clone());
            self.history_index = self.history.len();
        }

        // 出力にエコー
        self.output_text.push_str(&format!("$ {}\n", input));

        // シェルに送信（改行を追加）
        let command = format!("{}\n", input);
        if let Err(e) = self.pty.write(&command) {
            self.error_message = Some(format!("送信エラー: {}", e));
        }

        self.input_buffer.clear();
    }

    /// 出力を更新する
    fn update_output(&mut self) {
        let output = self.pty.read_output();
        if !output.is_empty() {
            // ANSIエスケープシーケンスを簡易的に除去
            let text = String::from_utf8_lossy(&output);
            let cleaned = self.strip_ansi_codes(&text);
            self.output_text.push_str(&cleaned);
        }
    }

    /// ANSIエスケープシーケンスを除去する（簡易版）
    fn strip_ansi_codes(&self, text: &str) -> String {
        let mut result = String::new();
        let mut in_escape = false;
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\x1b' {
                // エスケープシーケンスの開始
                if let Some(&next) = chars.peek() {
                    if next == '[' {
                        in_escape = true;
                        chars.next(); // '[' をスキップ
                        continue;
                    }
                }
            }

            if in_escape {
                // エスケープシーケンスの終了を検出
                if c.is_ascii_alphabetic() {
                    in_escape = false;
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    /// 履歴を上に移動する
    fn history_up(&mut self) {
        if !self.history.is_empty() && self.history_index > 0 {
            self.history_index -= 1;
            self.input_buffer = self.history[self.history_index].clone();
        }
    }

    /// 履歴を下に移動する
    fn history_down(&mut self) {
        if self.history_index < self.history.len() {
            self.history_index += 1;
            if self.history_index == self.history.len() {
                self.input_buffer.clear();
            } else {
                self.input_buffer = self.history[self.history_index].clone();
            }
        }
    }

    /// UIを描画する
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        if !self.visible {
            return;
        }

        // 出力を更新
        self.update_output();

        // エラーメッセージがあれば表示
        if let Some(ref error) = self.error_message {
            ui.colored_label(egui::Color32::RED, error);
        }

        // ターミナル出力エリア
        let available_height = ui.available_height() - 80.0; // 入力エリア用にスペースを確保

        egui::Frame::default()
            .fill(self.colors.background)
            .inner_margin(8.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.output_text.as_str())
                                .font(egui::TextStyle::Monospace)
                                .text_color(self.colors.foreground)
                                .desired_width(f32::INFINITY)
                                .interactive(false),
                        );
                    });
            });

        ui.separator();

        // 入力エリアラベル
        ui.horizontal(|ui| {
            ui.label("入力 (Enter: 改行, Ctrl+Enter: 送信)");
            if ui.button("送信").clicked() {
                self.send_input();
            }
            if ui.button("クリア").clicked() {
                self.output_text.clear();
            }
        });

        // 入力エリア（複数行対応）
        let response = ui.add(
            egui::TextEdit::multiline(&mut self.input_buffer)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .desired_rows(3)
                .hint_text("コマンドを入力..."),
        );

        // キーボードショートカットの処理
        if response.has_focus() {
            let modifiers = ui.ctx().input(|i| i.modifiers);

            // Ctrl+Enter: 送信
            if modifiers.ctrl && ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)) {
                self.send_input();
            }

            // 履歴ナビゲーション（Ctrl+Up/Down）
            if modifiers.ctrl && ui.ctx().input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                self.history_up();
            }
            if modifiers.ctrl && ui.ctx().input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                self.history_down();
            }
        }
    }

    /// サイズを変更する
    pub fn resize(&mut self, cols: u16, rows: u16) {
        let size = TerminalSize {
            cols,
            rows,
            pixel_width: 0,
            pixel_height: 0,
        };
        if let Err(e) = self.pty.resize(size) {
            self.error_message = Some(format!("リサイズエラー: {}", e));
        }
    }

    /// 色設定を取得する
    pub fn colors(&self) -> &TerminalColors {
        &self.colors
    }

    /// 色設定を変更する
    pub fn set_colors(&mut self, colors: TerminalColors) {
        self.colors = colors;
    }

    /// 入力バッファを取得する
    pub fn input_buffer(&self) -> &str {
        &self.input_buffer
    }

    /// 出力テキストを取得する
    pub fn output_text(&self) -> &str {
        &self.output_text
    }

    /// 履歴を取得する
    pub fn history(&self) -> &[String] {
        &self.history
    }
}

impl Default for TerminalView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_colors_default() {
        let colors = TerminalColors::default();
        assert_eq!(colors.background, egui::Color32::from_rgb(30, 30, 30));
        assert_eq!(colors.foreground, egui::Color32::from_rgb(204, 204, 204));
    }

    #[test]
    fn test_terminal_view_creation() {
        let view = TerminalView::new();
        assert!(!view.is_visible());
        assert!(view.input_buffer().is_empty());
        assert!(view.history().is_empty());
    }

    #[test]
    fn test_strip_ansi_codes() {
        let view = TerminalView::new();

        // 基本的なエスケープシーケンス
        let input = "\x1b[32mHello\x1b[0m";
        let output = view.strip_ansi_codes(input);
        assert_eq!(output, "Hello");

        // 複数のエスケープシーケンス
        let input = "\x1b[1;31mError:\x1b[0m Something went wrong";
        let output = view.strip_ansi_codes(input);
        assert_eq!(output, "Error: Something went wrong");

        // エスケープシーケンスなし
        let input = "Plain text";
        let output = view.strip_ansi_codes(input);
        assert_eq!(output, "Plain text");
    }

    #[test]
    fn test_history_navigation() {
        let mut view = TerminalView::new();

        // 履歴を追加（直接テスト用）
        view.history.push("command1".to_string());
        view.history.push("command2".to_string());
        view.history.push("command3".to_string());
        view.history_index = view.history.len();

        // 上に移動
        view.history_up();
        assert_eq!(view.input_buffer, "command3");

        view.history_up();
        assert_eq!(view.input_buffer, "command2");

        // 下に移動
        view.history_down();
        assert_eq!(view.input_buffer, "command3");

        // 最後まで下に移動
        view.history_down();
        assert!(view.input_buffer.is_empty());
    }

    #[test]
    fn test_toggle_visibility() {
        let mut view = TerminalView::new();
        assert!(!view.is_visible());

        // toggleを呼ぶと、start()が呼ばれるが、PTYが起動できない環境では
        // error_messageが設定される可能性がある
        // ここでは visible フラグが true になることを確認
        view.visible = true;
        assert!(view.is_visible());

        view.toggle();
        assert!(!view.is_visible());
    }

    #[test]
    fn test_set_colors() {
        let mut view = TerminalView::new();
        let new_colors = TerminalColors {
            background: egui::Color32::BLACK,
            foreground: egui::Color32::WHITE,
            cursor: egui::Color32::GREEN,
            selection: egui::Color32::BLUE,
        };

        view.set_colors(new_colors.clone());
        assert_eq!(view.colors().background, egui::Color32::BLACK);
        assert_eq!(view.colors().foreground, egui::Color32::WHITE);
    }
}
