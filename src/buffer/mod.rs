//! テキストバッファモジュール
//!
//! ropeyベースのテキストバッファ実装を提供する。
//! egui TextBuffer traitとの統合、Undo/Redo履歴管理を含む。

pub mod text_buffer;
pub mod history;

pub use text_buffer::TextBuffer;
pub use history::History;
