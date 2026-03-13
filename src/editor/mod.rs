//! エディターコンポーネント
//!
//! メインエディタービュー、行番号表示、カーソル管理を提供する。

pub mod view;
pub mod gutter;
pub mod cursor;

pub use view::EditorView;
pub use cursor::Cursor;
