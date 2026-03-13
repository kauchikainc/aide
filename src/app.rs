//! アプリケーションステート管理
//!
//! AIDEエディタのメインアプリケーションロジックを管理する。

use crate::buffer::TextBuffer;
use eframe::egui;

/// アプリケーションのメインステート
pub struct AideApp {
    /// 現在のテキストバッファ
    buffer: TextBuffer,
    /// ステータスメッセージ
    status_message: String,
}

impl AideApp {
    /// 新しいアプリケーションインスタンスを作成する
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            buffer: TextBuffer::new(),
            status_message: String::from("AIDE - 準備完了"),
        }
    }

    /// ファイルを開く
    pub fn open_file(&mut self, path: &std::path::Path) {
        match TextBuffer::from_file(path) {
            Ok(buffer) => {
                self.buffer = buffer;
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
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            self.open_file(&path);
        }
    }

    /// ファイルダイアログを開いて保存先を選択する
    fn show_save_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().save_file() {
            match self.buffer.save_as(&path) {
                Ok(()) => {
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
                    if ui.button("元に戻す (Ctrl+Z)").clicked() {
                        // TODO: Undo実装
                        ui.close();
                    }
                    if ui.button("やり直し (Ctrl+Y)").clicked() {
                        // TODO: Redo実装
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
                    // カーソル位置などの表示（将来実装）
                    let lines = self.buffer.len_lines();
                    let chars = self.buffer.len_chars();
                    ui.label(format!("{} 行, {} 文字", lines, chars));
                });
            });
        });

        // メインエディター領域
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // テキストエディット
                // NOTE: eguiの標準TextEditはRopeと直接連携できないため、
                // 本番では自前のエディタコンポーネントを使用する
                let text = self.buffer.to_string();
                let mut editable_text = text;
                let response = ui.add(
                    egui::TextEdit::multiline(&mut editable_text)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(30)
                        .code_editor(),
                );

                // 変更があった場合、バッファを更新
                if response.changed() {
                    // 簡易実装: 全置換（本番では差分計算を行う）
                    self.buffer = TextBuffer::from_str(&editable_text);
                }
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
    }
}
