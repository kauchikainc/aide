//! 診断表示
//!
//! LSPからの診断メッセージ（エラー、警告など）を表示する。

use super::Range;

/// 診断の重要度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticSeverity {
    /// エラー
    Error = 1,
    /// 警告
    Warning = 2,
    /// 情報
    Information = 3,
    /// ヒント
    Hint = 4,
}

impl DiagnosticSeverity {
    /// 日本語の表示名を取得する
    pub fn display_name(&self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "エラー",
            DiagnosticSeverity::Warning => "警告",
            DiagnosticSeverity::Information => "情報",
            DiagnosticSeverity::Hint => "ヒント",
        }
    }

    /// アイコン文字を取得する
    pub fn icon(&self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "❌",
            DiagnosticSeverity::Warning => "⚠️",
            DiagnosticSeverity::Information => "ℹ️",
            DiagnosticSeverity::Hint => "💡",
        }
    }
}

impl From<lsp_types::DiagnosticSeverity> for DiagnosticSeverity {
    fn from(severity: lsp_types::DiagnosticSeverity) -> Self {
        match severity {
            lsp_types::DiagnosticSeverity::ERROR => DiagnosticSeverity::Error,
            lsp_types::DiagnosticSeverity::WARNING => DiagnosticSeverity::Warning,
            lsp_types::DiagnosticSeverity::INFORMATION => DiagnosticSeverity::Information,
            lsp_types::DiagnosticSeverity::HINT => DiagnosticSeverity::Hint,
            _ => DiagnosticSeverity::Information,
        }
    }
}

/// 診断メッセージ
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// 診断が適用される範囲
    pub range: Range,
    /// 重要度
    pub severity: DiagnosticSeverity,
    /// メッセージ
    pub message: String,
    /// ソース（例: "rust-analyzer"）
    pub source: Option<String>,
    /// エラーコード
    pub code: Option<String>,
}

impl Diagnostic {
    /// 新しい診断を作成する
    pub fn new(range: Range, severity: DiagnosticSeverity, message: impl Into<String>) -> Self {
        Self {
            range,
            severity,
            message: message.into(),
            source: None,
            code: None,
        }
    }

    /// ソースを設定する
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// エラーコードを設定する
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// フォーマットされたメッセージを取得する
    pub fn formatted_message(&self) -> String {
        let mut parts = vec![self.severity.display_name().to_string()];

        if let Some(ref code) = self.code {
            parts.push(format!("[{}]", code));
        }

        parts.push(self.message.clone());

        if let Some(ref source) = self.source {
            parts.push(format!("({})", source));
        }

        parts.join(" ")
    }
}

impl From<lsp_types::Diagnostic> for Diagnostic {
    fn from(diag: lsp_types::Diagnostic) -> Self {
        let severity = diag
            .severity
            .map(DiagnosticSeverity::from)
            .unwrap_or(DiagnosticSeverity::Information);

        let code = diag.code.map(|c| match c {
            lsp_types::NumberOrString::Number(n) => n.to_string(),
            lsp_types::NumberOrString::String(s) => s,
        });

        Self {
            range: diag.range.into(),
            severity,
            message: diag.message,
            source: diag.source,
            code,
        }
    }
}

/// 診断のコレクション
#[derive(Debug, Clone, Default)]
pub struct DiagnosticCollection {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollection {
    /// 新しいコレクションを作成する
    pub fn new() -> Self {
        Self::default()
    }

    /// 診断を追加する
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// 診断をクリアする
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    /// 診断を設定する（既存の診断を置き換え）
    pub fn set(&mut self, diagnostics: Vec<Diagnostic>) {
        self.diagnostics = diagnostics;
    }

    /// すべての診断を取得する
    pub fn all(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// エラーのみを取得する
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect()
    }

    /// 警告のみを取得する
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .collect()
    }

    /// 指定行の診断を取得する
    pub fn for_line(&self, line: u32) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.range.start.line <= line && line <= d.range.end.line)
            .collect()
    }

    /// 診断の数を取得する
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// 診断が空かどうか
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// エラー数を取得する
    pub fn error_count(&self) -> usize {
        self.errors().len()
    }

    /// 警告数を取得する
    pub fn warning_count(&self) -> usize {
        self.warnings().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::Position;

    #[test]
    fn test_diagnostic_severity_display_name() {
        assert_eq!(DiagnosticSeverity::Error.display_name(), "エラー");
        assert_eq!(DiagnosticSeverity::Warning.display_name(), "警告");
        assert_eq!(DiagnosticSeverity::Information.display_name(), "情報");
        assert_eq!(DiagnosticSeverity::Hint.display_name(), "ヒント");
    }

    #[test]
    fn test_diagnostic_creation() {
        let range = Range::new(Position::new(0, 0), Position::new(0, 10));
        let diag = Diagnostic::new(range, DiagnosticSeverity::Error, "テストエラー")
            .with_source("test")
            .with_code("E001");

        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.message, "テストエラー");
        assert_eq!(diag.source, Some("test".to_string()));
        assert_eq!(diag.code, Some("E001".to_string()));
    }

    #[test]
    fn test_diagnostic_formatted_message() {
        let range = Range::new(Position::new(0, 0), Position::new(0, 10));
        let diag = Diagnostic::new(range, DiagnosticSeverity::Error, "変数が見つかりません")
            .with_source("rust-analyzer")
            .with_code("E0425");

        let formatted = diag.formatted_message();
        assert!(formatted.contains("エラー"));
        assert!(formatted.contains("E0425"));
        assert!(formatted.contains("変数が見つかりません"));
        assert!(formatted.contains("rust-analyzer"));
    }

    #[test]
    fn test_diagnostic_collection() {
        let mut collection = DiagnosticCollection::new();
        assert!(collection.is_empty());

        let range = Range::new(Position::new(0, 0), Position::new(0, 10));
        collection.add(Diagnostic::new(range, DiagnosticSeverity::Error, "エラー1"));
        collection.add(Diagnostic::new(range, DiagnosticSeverity::Warning, "警告1"));
        collection.add(Diagnostic::new(range, DiagnosticSeverity::Error, "エラー2"));

        assert_eq!(collection.len(), 3);
        assert_eq!(collection.error_count(), 2);
        assert_eq!(collection.warning_count(), 1);
    }

    #[test]
    fn test_diagnostic_collection_for_line() {
        let mut collection = DiagnosticCollection::new();

        collection.add(Diagnostic::new(
            Range::new(Position::new(0, 0), Position::new(0, 10)),
            DiagnosticSeverity::Error,
            "1行目のエラー",
        ));
        collection.add(Diagnostic::new(
            Range::new(Position::new(5, 0), Position::new(5, 10)),
            DiagnosticSeverity::Error,
            "6行目のエラー",
        ));

        let line_0_diags = collection.for_line(0);
        assert_eq!(line_0_diags.len(), 1);

        let line_5_diags = collection.for_line(5);
        assert_eq!(line_5_diags.len(), 1);

        let line_10_diags = collection.for_line(10);
        assert_eq!(line_10_diags.len(), 0);
    }
}
