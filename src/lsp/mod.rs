//! LSPクライアントモジュール
//!
//! Language Server Protocolによる言語サポートを提供する。
//! rust-analyzerとの連携により、補完、診断、定義ジャンプなどの機能を実現する。

pub mod client;
pub mod completion;
pub mod diagnostics;

pub use client::{ClientState, LspClient, LspClientError};
pub use completion::{CompletionItem, CompletionKind, CompletionList};
pub use diagnostics::{Diagnostic, DiagnosticCollection, DiagnosticSeverity};


/// LSPがサポートする言語
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspLanguage {
    Rust,
}

impl LspLanguage {
    /// 言語サーバーのコマンドを取得する
    pub fn server_command(&self) -> &'static str {
        match self {
            LspLanguage::Rust => "rust-analyzer",
        }
    }

    /// ファイル拡張子から言語を判定する
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(LspLanguage::Rust),
            _ => None,
        }
    }

    /// 言語IDを取得する（LSPプロトコル用）
    pub fn language_id(&self) -> &'static str {
        match self {
            LspLanguage::Rust => "rust",
        }
    }
}

/// LSPの位置情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// 行番号（0始まり）
    pub line: u32,
    /// 列番号（UTF-16コードユニット、0始まり）
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

impl From<Position> for lsp_types::Position {
    fn from(pos: Position) -> Self {
        lsp_types::Position {
            line: pos.line,
            character: pos.character,
        }
    }
}

impl From<lsp_types::Position> for Position {
    fn from(pos: lsp_types::Position) -> Self {
        Self {
            line: pos.line,
            character: pos.character,
        }
    }
}

/// LSPの範囲情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

impl From<Range> for lsp_types::Range {
    fn from(range: Range) -> Self {
        lsp_types::Range {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

impl From<lsp_types::Range> for Range {
    fn from(range: lsp_types::Range) -> Self {
        Self {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_language_from_extension() {
        assert_eq!(LspLanguage::from_extension("rs"), Some(LspLanguage::Rust));
        assert_eq!(LspLanguage::from_extension("RS"), Some(LspLanguage::Rust));
        assert_eq!(LspLanguage::from_extension("txt"), None);
    }

    #[test]
    fn test_lsp_language_server_command() {
        assert_eq!(LspLanguage::Rust.server_command(), "rust-analyzer");
    }

    #[test]
    fn test_position_conversion() {
        let pos = Position::new(10, 5);
        let lsp_pos: lsp_types::Position = pos.into();
        assert_eq!(lsp_pos.line, 10);
        assert_eq!(lsp_pos.character, 5);

        let back: Position = lsp_pos.into();
        assert_eq!(back, pos);
    }

    #[test]
    fn test_range_conversion() {
        let range = Range::new(Position::new(1, 0), Position::new(1, 10));
        let lsp_range: lsp_types::Range = range.into();
        assert_eq!(lsp_range.start.line, 1);
        assert_eq!(lsp_range.end.character, 10);
    }
}
