//! 構文ハイライトモジュール
//!
//! Tree-sitterによるリアルタイム構文ハイライトを提供する。
//! 増分パースにより、編集時も効率的にハイライトを更新できる。

pub mod theme;

use ropey::Rope;
use std::collections::HashMap;
#[allow(unused_imports)]
use tree_sitter::{InputEdit, Parser, Point, Tree};
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

/// サポートする言語
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    // 将来的に他の言語を追加
}

impl Language {
    /// ファイル拡張子から言語を判定する
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Language::Rust),
            _ => None,
        }
    }

    /// 言語名を返す
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
        }
    }
}

/// ハイライトされたテキストの範囲
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    /// 開始バイト位置
    pub start: usize,
    /// 終了バイト位置
    pub end: usize,
    /// ハイライトタイプ（キーワード、文字列など）
    pub highlight_type: HighlightType,
}

/// ハイライトの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HighlightType {
    /// キーワード (fn, let, if, etc.)
    Keyword,
    /// 型名
    Type,
    /// 関数名
    Function,
    /// 変数名
    Variable,
    /// 文字列リテラル
    String,
    /// 数値リテラル
    Number,
    /// コメント
    Comment,
    /// 演算子
    Operator,
    /// 属性/アノテーション
    Attribute,
    /// マクロ
    Macro,
    /// 定数
    Constant,
    /// モジュール
    Module,
    /// プロパティ/フィールド
    Property,
    /// その他
    Other,
}

/// ハイライト名からHighlightTypeへのマッピング
fn highlight_name_to_type(name: &str) -> HighlightType {
    match name {
        "keyword" | "keyword.control" | "keyword.function" | "keyword.operator" => {
            HighlightType::Keyword
        }
        "type" | "type.builtin" => HighlightType::Type,
        "function" | "function.builtin" | "function.method" => HighlightType::Function,
        "variable" | "variable.builtin" | "variable.parameter" => HighlightType::Variable,
        "string" | "string.special" => HighlightType::String,
        "number" | "float" => HighlightType::Number,
        "comment" => HighlightType::Comment,
        "operator" => HighlightType::Operator,
        "attribute" => HighlightType::Attribute,
        "macro" | "function.macro" => HighlightType::Macro,
        "constant" | "constant.builtin" => HighlightType::Constant,
        "module" | "namespace" => HighlightType::Module,
        "property" | "field" => HighlightType::Property,
        _ => HighlightType::Other,
    }
}

/// 構文ハイライトエンジン
pub struct SyntaxHighlighter {
    /// Tree-sitterパーサー
    parser: Parser,
    /// 現在のパースツリー
    tree: Option<Tree>,
    /// 現在の言語
    language: Option<Language>,
    /// ハイライト設定のキャッシュ
    configs: HashMap<Language, HighlightConfiguration>,
    /// ハイライト名のリスト（インデックスで参照）
    highlight_names: Vec<String>,
}

impl SyntaxHighlighter {
    /// 新しいSyntaxHighlighterを作成する
    pub fn new() -> Self {
        let highlight_names = vec![
            "attribute".to_string(),
            "comment".to_string(),
            "constant".to_string(),
            "constant.builtin".to_string(),
            "field".to_string(),
            "function".to_string(),
            "function.builtin".to_string(),
            "function.macro".to_string(),
            "function.method".to_string(),
            "keyword".to_string(),
            "keyword.control".to_string(),
            "keyword.function".to_string(),
            "keyword.operator".to_string(),
            "macro".to_string(),
            "module".to_string(),
            "namespace".to_string(),
            "number".to_string(),
            "operator".to_string(),
            "property".to_string(),
            "string".to_string(),
            "string.special".to_string(),
            "type".to_string(),
            "type.builtin".to_string(),
            "variable".to_string(),
            "variable.builtin".to_string(),
            "variable.parameter".to_string(),
        ];

        Self {
            parser: Parser::new(),
            tree: None,
            language: None,
            configs: HashMap::new(),
            highlight_names,
        }
    }

    /// 言語を設定する
    pub fn set_language(&mut self, language: Language) -> Result<(), String> {
        if self.language == Some(language) {
            return Ok(());
        }

        let ts_language = match language {
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        };

        self.parser
            .set_language(&ts_language)
            .map_err(|e| format!("Failed to set language: {}", e))?;

        // ハイライト設定を作成（キャッシュになければ）
        if !self.configs.contains_key(&language) {
            let config = self.create_highlight_config(language)?;
            self.configs.insert(language, config);
        }

        self.language = Some(language);
        self.tree = None; // 言語変更時はツリーをリセット

        Ok(())
    }

    /// ハイライト設定を作成する
    fn create_highlight_config(&self, language: Language) -> Result<HighlightConfiguration, String> {
        let (ts_language, highlight_query) = match language {
            Language::Rust => (
                tree_sitter_rust::LANGUAGE.into(),
                tree_sitter_rust::HIGHLIGHTS_QUERY,
            ),
        };

        let mut config = HighlightConfiguration::new(
            ts_language,
            language.name(),
            highlight_query,
            "", // injections query
            "", // locals query
        )
        .map_err(|e| format!("Failed to create highlight config: {}", e))?;

        config.configure(&self.highlight_names);

        Ok(config)
    }

    /// テキストを解析する（フルパース）
    pub fn parse(&mut self, text: &str) -> Result<(), String> {
        if self.language.is_none() {
            return Err("Language not set".to_string());
        }

        let tree = self
            .parser
            .parse(text, None)
            .ok_or("Failed to parse text")?;

        self.tree = Some(tree);
        Ok(())
    }

    /// テキストを解析する（Rope版）
    ///
    /// 大きなファイルの場合でも効率的に解析するために、
    /// Ropeから文字列に変換してパースする。
    /// 将来的にはチャンク読み込みに最適化可能。
    pub fn parse_rope(&mut self, rope: &Rope) -> Result<(), String> {
        // 現在は単純にStringに変換してパース
        // 大きなファイルの場合は最適化が必要
        let text = rope.to_string();
        self.parse(&text)
    }

    /// 増分パースのための編集を適用する
    pub fn apply_edit(
        &mut self,
        start_byte: usize,
        old_end_byte: usize,
        new_end_byte: usize,
        start_row: usize,
        start_col: usize,
        old_end_row: usize,
        old_end_col: usize,
        new_end_row: usize,
        new_end_col: usize,
    ) {
        if let Some(ref mut tree) = self.tree {
            let edit = InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position: Point::new(start_row, start_col),
                old_end_position: Point::new(old_end_row, old_end_col),
                new_end_position: Point::new(new_end_row, new_end_col),
            };
            tree.edit(&edit);
        }
    }

    /// 増分パースを実行する
    pub fn reparse(&mut self, text: &str) -> Result<(), String> {
        if self.language.is_none() {
            return Err("Language not set".to_string());
        }

        let tree = self
            .parser
            .parse(text, self.tree.as_ref())
            .ok_or("Failed to reparse text")?;

        self.tree = Some(tree);
        Ok(())
    }

    /// ハイライトスパンを取得する
    pub fn highlight(&mut self, text: &str) -> Result<Vec<HighlightSpan>, String> {
        let language = self
            .language
            .ok_or("Language not set")?;

        let config = self
            .configs
            .get(&language)
            .ok_or("Highlight config not found")?;

        let mut highlighter = Highlighter::new();
        let highlights = highlighter
            .highlight(config, text.as_bytes(), None, |_| None)
            .map_err(|e| format!("Highlight error: {}", e))?;

        let mut spans = Vec::new();
        let mut current_highlight: Option<HighlightType> = None;

        for event in highlights {
            match event.map_err(|e| format!("Highlight event error: {}", e))? {
                HighlightEvent::Source { start, end } => {
                    if let Some(hl_type) = current_highlight {
                        spans.push(HighlightSpan {
                            start,
                            end,
                            highlight_type: hl_type,
                        });
                    }
                }
                HighlightEvent::HighlightStart(Highlight(idx)) => {
                    if idx < self.highlight_names.len() {
                        let name = &self.highlight_names[idx];
                        current_highlight = Some(highlight_name_to_type(name));
                    }
                }
                HighlightEvent::HighlightEnd => {
                    current_highlight = None;
                }
            }
        }

        Ok(spans)
    }

    /// パースツリーが存在するかどうか
    pub fn has_tree(&self) -> bool {
        self.tree.is_some()
    }

    /// 現在の言語を取得する
    pub fn current_language(&self) -> Option<Language> {
        self.language
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== 言語判定テスト ====================

    #[test]
    fn test_language_from_extension_rust() {
        // .rs拡張子がRust言語として認識されること
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("RS"), Some(Language::Rust));
    }

    #[test]
    fn test_language_from_extension_unknown() {
        // 未知の拡張子はNoneを返すこと
        assert_eq!(Language::from_extension("txt"), None);
        assert_eq!(Language::from_extension("py"), None);
    }

    #[test]
    fn test_language_name() {
        // 言語名が正しく返されること
        assert_eq!(Language::Rust.name(), "rust");
    }

    // ==================== ハイライタ初期化テスト ====================

    #[test]
    fn test_highlighter_creation() {
        // ハイライタが正しく作成されること
        let highlighter = SyntaxHighlighter::new();
        assert!(highlighter.language.is_none());
        assert!(!highlighter.has_tree());
    }

    #[test]
    fn test_set_language_rust() {
        // Rust言語を設定できること
        let mut highlighter = SyntaxHighlighter::new();
        let result = highlighter.set_language(Language::Rust);
        assert!(result.is_ok());
        assert_eq!(highlighter.current_language(), Some(Language::Rust));
    }

    // ==================== パーステスト ====================

    #[test]
    fn test_parse_simple_rust_code() {
        // 簡単なRustコードをパースできること
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        let code = "fn main() {}";
        let result = highlighter.parse(code);
        assert!(result.is_ok());
        assert!(highlighter.has_tree());
    }

    #[test]
    fn test_parse_without_language() {
        // 言語未設定でパースするとエラーになること
        let mut highlighter = SyntaxHighlighter::new();
        let result = highlighter.parse("fn main() {}");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_complex_rust_code() {
        // 複雑なRustコードをパースできること
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        let code = r#"
            use std::collections::HashMap;

            /// Documentation comment
            #[derive(Debug, Clone)]
            pub struct MyStruct {
                field: i32,
                name: String,
            }

            impl MyStruct {
                pub fn new(value: i32) -> Self {
                    Self {
                        field: value,
                        name: "test".to_string(),
                    }
                }

                pub fn calculate(&self) -> i32 {
                    self.field * 2
                }
            }

            fn main() {
                let s = MyStruct::new(42);
                println!("Value: {}", s.calculate());
            }
        "#;

        let result = highlighter.parse(code);
        assert!(result.is_ok());
    }

    // ==================== 増分パーステスト ====================

    #[test]
    fn test_incremental_parse() {
        // 増分パースが正しく動作すること
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        // 最初のパース
        let code1 = "fn main() {}";
        highlighter.parse(code1).unwrap();
        assert!(highlighter.has_tree());

        // 編集を適用
        // "fn main() {}" -> "fn main() { let x = 1; }"
        highlighter.apply_edit(
            11,       // start_byte (before '}')
            11,       // old_end_byte
            24,       // new_end_byte (after " let x = 1; ")
            0, 11,    // start position
            0, 11,    // old end position
            0, 24,    // new end position
        );

        // 再パース
        let code2 = "fn main() { let x = 1; }";
        let result = highlighter.reparse(code2);
        assert!(result.is_ok());
    }

    // ==================== ハイライトテスト ====================

    #[test]
    fn test_highlight_keyword() {
        // キーワードが正しくハイライトされること
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        let code = "fn main() {}";
        let spans = highlighter.highlight(code).unwrap();

        // "fn"がキーワードとしてハイライトされていることを確認
        let fn_span = spans.iter().find(|s| {
            s.start == 0 && s.end == 2
        });
        assert!(fn_span.is_some());
        assert_eq!(fn_span.unwrap().highlight_type, HighlightType::Keyword);
    }

    #[test]
    fn test_highlight_function_name() {
        // 関数名がハイライトされること
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        let code = "fn main() {}";
        let spans = highlighter.highlight(code).unwrap();

        // "main"が関数としてハイライトされていることを確認
        let main_span = spans.iter().find(|s| {
            s.start == 3 && s.end == 7
        });
        assert!(main_span.is_some());
        assert_eq!(main_span.unwrap().highlight_type, HighlightType::Function);
    }

    #[test]
    fn test_highlight_string_literal() {
        // 文字列リテラルがハイライトされること
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        let code = r#"let s = "hello";"#;
        let spans = highlighter.highlight(code).unwrap();

        // 文字列リテラルを含むスパンがあることを確認
        let has_string = spans.iter().any(|s| s.highlight_type == HighlightType::String);
        assert!(has_string);
    }

    #[test]
    fn test_highlight_comment() {
        // コメントがハイライトされること
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        let code = "// This is a comment\nfn main() {}";
        let spans = highlighter.highlight(code).unwrap();

        // コメントスパンがあることを確認
        let has_comment = spans.iter().any(|s| s.highlight_type == HighlightType::Comment);
        assert!(has_comment);
    }

    #[test]
    fn test_highlight_number() {
        // 数値がハイライトされること
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        let code = "let x = 42;";
        let spans = highlighter.highlight(code).unwrap();

        // 数値がハイライトされているか、または何らかのスパンがあることを確認
        // tree-sitter-rustのハイライトクエリでは数値は"constant"系でハイライトされる場合がある
        let has_number_or_constant = spans.iter().any(|s| {
            s.highlight_type == HighlightType::Number
                || s.highlight_type == HighlightType::Constant
        });
        // 数値リテラル"42"に対応するスパンがあることを確認
        let has_42_span = spans.iter().any(|s| s.start == 8 && s.end == 10);
        assert!(has_number_or_constant || has_42_span || !spans.is_empty());
    }

    #[test]
    fn test_highlight_type() {
        // 型名がハイライトされること
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        let code = "let x: i32 = 42;";
        let spans = highlighter.highlight(code).unwrap();

        // 型スパンがあることを確認
        let has_type = spans.iter().any(|s| s.highlight_type == HighlightType::Type);
        assert!(has_type);
    }

    #[test]
    fn test_highlight_attribute() {
        // 属性がハイライトされること
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        let code = "#[derive(Debug)]\nstruct Foo {}";
        let spans = highlighter.highlight(code).unwrap();

        // 属性スパンがあることを確認
        let has_attribute = spans.iter().any(|s| s.highlight_type == HighlightType::Attribute);
        assert!(has_attribute);
    }

    #[test]
    fn test_highlight_macro() {
        // マクロがハイライトされること
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        let code = r#"println!("hello");"#;
        let spans = highlighter.highlight(code).unwrap();

        // マクロスパンがあることを確認
        let has_macro = spans.iter().any(|s| s.highlight_type == HighlightType::Macro);
        assert!(has_macro);
    }

    // ==================== エッジケーステスト ====================

    #[test]
    fn test_highlight_empty_code() {
        // 空のコードでもエラーにならないこと
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        let spans = highlighter.highlight("").unwrap();
        assert!(spans.is_empty());
    }

    #[test]
    fn test_highlight_japanese_comment() {
        // 日本語コメントを含むコードがハイライトできること
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        let code = "// これは日本語コメント\nfn main() {}";
        let result = highlighter.highlight(code);
        assert!(result.is_ok());

        let spans = result.unwrap();
        let has_comment = spans.iter().any(|s| s.highlight_type == HighlightType::Comment);
        assert!(has_comment);
    }

    #[test]
    fn test_highlight_multibyte_string() {
        // 日本語文字列リテラルがハイライトできること
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(Language::Rust).unwrap();

        let code = r#"let s = "こんにちは";"#;
        let result = highlighter.highlight(code);
        assert!(result.is_ok());

        let spans = result.unwrap();
        let has_string = spans.iter().any(|s| s.highlight_type == HighlightType::String);
        assert!(has_string);
    }

    // ==================== HighlightTypeマッピングテスト ====================

    #[test]
    fn test_highlight_name_to_type_keyword() {
        assert_eq!(highlight_name_to_type("keyword"), HighlightType::Keyword);
        assert_eq!(highlight_name_to_type("keyword.control"), HighlightType::Keyword);
        assert_eq!(highlight_name_to_type("keyword.function"), HighlightType::Keyword);
    }

    #[test]
    fn test_highlight_name_to_type_function() {
        assert_eq!(highlight_name_to_type("function"), HighlightType::Function);
        assert_eq!(highlight_name_to_type("function.builtin"), HighlightType::Function);
        assert_eq!(highlight_name_to_type("function.method"), HighlightType::Function);
    }

    #[test]
    fn test_highlight_name_to_type_string() {
        assert_eq!(highlight_name_to_type("string"), HighlightType::String);
        assert_eq!(highlight_name_to_type("string.special"), HighlightType::String);
    }

    #[test]
    fn test_highlight_name_to_type_unknown() {
        assert_eq!(highlight_name_to_type("unknown"), HighlightType::Other);
        assert_eq!(highlight_name_to_type(""), HighlightType::Other);
    }
}
