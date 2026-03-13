//! AIDE - 軽量エディター
//!
//! Rust製の軽量GUIエディター。LSP統合、リアルタイム構文ハイライト、
//! 内蔵ターミナル、ClaudeCode連携機能を持つ。

mod app;
mod buffer;
mod editor;
mod highlight;
mod lsp;
mod terminal;
mod ui;

use app::AideApp;

fn main() -> eframe::Result<()> {
    // ロガーの初期化
    env_logger::init();

    // ウィンドウオプションの設定
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("AIDE - 軽量エディター"),
        ..Default::default()
    };

    // アプリケーションの起動
    eframe::run_native(
        "AIDE",
        options,
        Box::new(|cc| Ok(Box::new(AideApp::new(cc)))),
    )
}
