//! 補完機能
//!
//! LSPによるコード補完を提供する。

/// 補完アイテムの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    /// テキスト
    Text,
    /// メソッド
    Method,
    /// 関数
    Function,
    /// コンストラクタ
    Constructor,
    /// フィールド
    Field,
    /// 変数
    Variable,
    /// クラス
    Class,
    /// インターフェース
    Interface,
    /// モジュール
    Module,
    /// プロパティ
    Property,
    /// ユニット
    Unit,
    /// 値
    Value,
    /// 列挙型
    Enum,
    /// キーワード
    Keyword,
    /// スニペット
    Snippet,
    /// 色
    Color,
    /// ファイル
    File,
    /// 参照
    Reference,
    /// フォルダ
    Folder,
    /// 列挙型メンバ
    EnumMember,
    /// 定数
    Constant,
    /// 構造体
    Struct,
    /// イベント
    Event,
    /// 演算子
    Operator,
    /// 型パラメータ
    TypeParameter,
    /// マクロ
    Macro,
}

impl CompletionKind {
    /// 日本語の表示名を取得する
    pub fn display_name(&self) -> &'static str {
        match self {
            CompletionKind::Text => "テキスト",
            CompletionKind::Method => "メソッド",
            CompletionKind::Function => "関数",
            CompletionKind::Constructor => "コンストラクタ",
            CompletionKind::Field => "フィールド",
            CompletionKind::Variable => "変数",
            CompletionKind::Class => "クラス",
            CompletionKind::Interface => "インターフェース",
            CompletionKind::Module => "モジュール",
            CompletionKind::Property => "プロパティ",
            CompletionKind::Unit => "ユニット",
            CompletionKind::Value => "値",
            CompletionKind::Enum => "列挙型",
            CompletionKind::Keyword => "キーワード",
            CompletionKind::Snippet => "スニペット",
            CompletionKind::Color => "色",
            CompletionKind::File => "ファイル",
            CompletionKind::Reference => "参照",
            CompletionKind::Folder => "フォルダ",
            CompletionKind::EnumMember => "列挙型メンバ",
            CompletionKind::Constant => "定数",
            CompletionKind::Struct => "構造体",
            CompletionKind::Event => "イベント",
            CompletionKind::Operator => "演算子",
            CompletionKind::TypeParameter => "型パラメータ",
            CompletionKind::Macro => "マクロ",
        }
    }

    /// アイコン文字を取得する
    pub fn icon(&self) -> &'static str {
        match self {
            CompletionKind::Text => "📝",
            CompletionKind::Method => "🔧",
            CompletionKind::Function => "ƒ",
            CompletionKind::Constructor => "🏗️",
            CompletionKind::Field => "📦",
            CompletionKind::Variable => "𝑥",
            CompletionKind::Class => "📘",
            CompletionKind::Interface => "📗",
            CompletionKind::Module => "📁",
            CompletionKind::Property => "🔑",
            CompletionKind::Unit => "📐",
            CompletionKind::Value => "💎",
            CompletionKind::Enum => "📋",
            CompletionKind::Keyword => "🔤",
            CompletionKind::Snippet => "✂️",
            CompletionKind::Color => "🎨",
            CompletionKind::File => "📄",
            CompletionKind::Reference => "🔗",
            CompletionKind::Folder => "📂",
            CompletionKind::EnumMember => "📌",
            CompletionKind::Constant => "🔒",
            CompletionKind::Struct => "🧱",
            CompletionKind::Event => "⚡",
            CompletionKind::Operator => "➕",
            CompletionKind::TypeParameter => "🅃",
            CompletionKind::Macro => "🔮",
        }
    }
}

impl From<lsp_types::CompletionItemKind> for CompletionKind {
    fn from(kind: lsp_types::CompletionItemKind) -> Self {
        match kind {
            lsp_types::CompletionItemKind::TEXT => CompletionKind::Text,
            lsp_types::CompletionItemKind::METHOD => CompletionKind::Method,
            lsp_types::CompletionItemKind::FUNCTION => CompletionKind::Function,
            lsp_types::CompletionItemKind::CONSTRUCTOR => CompletionKind::Constructor,
            lsp_types::CompletionItemKind::FIELD => CompletionKind::Field,
            lsp_types::CompletionItemKind::VARIABLE => CompletionKind::Variable,
            lsp_types::CompletionItemKind::CLASS => CompletionKind::Class,
            lsp_types::CompletionItemKind::INTERFACE => CompletionKind::Interface,
            lsp_types::CompletionItemKind::MODULE => CompletionKind::Module,
            lsp_types::CompletionItemKind::PROPERTY => CompletionKind::Property,
            lsp_types::CompletionItemKind::UNIT => CompletionKind::Unit,
            lsp_types::CompletionItemKind::VALUE => CompletionKind::Value,
            lsp_types::CompletionItemKind::ENUM => CompletionKind::Enum,
            lsp_types::CompletionItemKind::KEYWORD => CompletionKind::Keyword,
            lsp_types::CompletionItemKind::SNIPPET => CompletionKind::Snippet,
            lsp_types::CompletionItemKind::COLOR => CompletionKind::Color,
            lsp_types::CompletionItemKind::FILE => CompletionKind::File,
            lsp_types::CompletionItemKind::REFERENCE => CompletionKind::Reference,
            lsp_types::CompletionItemKind::FOLDER => CompletionKind::Folder,
            lsp_types::CompletionItemKind::ENUM_MEMBER => CompletionKind::EnumMember,
            lsp_types::CompletionItemKind::CONSTANT => CompletionKind::Constant,
            lsp_types::CompletionItemKind::STRUCT => CompletionKind::Struct,
            lsp_types::CompletionItemKind::EVENT => CompletionKind::Event,
            lsp_types::CompletionItemKind::OPERATOR => CompletionKind::Operator,
            lsp_types::CompletionItemKind::TYPE_PARAMETER => CompletionKind::TypeParameter,
            _ => CompletionKind::Text,
        }
    }
}

/// 補完アイテム
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// ラベル（表示名）
    pub label: String,
    /// 種類
    pub kind: CompletionKind,
    /// 詳細情報
    pub detail: Option<String>,
    /// ドキュメント
    pub documentation: Option<String>,
    /// 挿入テキスト
    pub insert_text: Option<String>,
    /// フィルターテキスト
    pub filter_text: Option<String>,
    /// ソート用テキスト
    pub sort_text: Option<String>,
}

impl CompletionItem {
    /// 新しい補完アイテムを作成する
    pub fn new(label: impl Into<String>, kind: CompletionKind) -> Self {
        Self {
            label: label.into(),
            kind,
            detail: None,
            documentation: None,
            insert_text: None,
            filter_text: None,
            sort_text: None,
        }
    }

    /// 詳細情報を設定する
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// ドキュメントを設定する
    pub fn with_documentation(mut self, doc: impl Into<String>) -> Self {
        self.documentation = Some(doc.into());
        self
    }

    /// 挿入テキストを設定する
    pub fn with_insert_text(mut self, text: impl Into<String>) -> Self {
        self.insert_text = Some(text.into());
        self
    }

    /// 挿入するテキストを取得する（insert_textがなければlabelを使用）
    pub fn text_to_insert(&self) -> &str {
        self.insert_text.as_deref().unwrap_or(&self.label)
    }

    /// フォーマットされた表示文字列を取得する
    pub fn formatted_display(&self) -> String {
        let icon = self.kind.icon();
        if let Some(ref detail) = self.detail {
            format!("{} {} - {}", icon, self.label, detail)
        } else {
            format!("{} {}", icon, self.label)
        }
    }
}

impl From<lsp_types::CompletionItem> for CompletionItem {
    fn from(item: lsp_types::CompletionItem) -> Self {
        let kind = item
            .kind
            .map(CompletionKind::from)
            .unwrap_or(CompletionKind::Text);

        let documentation = item.documentation.map(|doc| match doc {
            lsp_types::Documentation::String(s) => s,
            lsp_types::Documentation::MarkupContent(mc) => mc.value,
        });

        let insert_text = item.insert_text.or_else(|| {
            item.text_edit.and_then(|edit| match edit {
                lsp_types::CompletionTextEdit::Edit(e) => Some(e.new_text),
                lsp_types::CompletionTextEdit::InsertAndReplace(e) => Some(e.new_text),
            })
        });

        Self {
            label: item.label,
            kind,
            detail: item.detail,
            documentation,
            insert_text,
            filter_text: item.filter_text,
            sort_text: item.sort_text,
        }
    }
}

/// 補完リスト
#[derive(Debug, Clone, Default)]
pub struct CompletionList {
    /// 補完アイテムのリスト
    items: Vec<CompletionItem>,
    /// 不完全なリストかどうか（追加の入力で更なる補完候補がある可能性）
    pub is_incomplete: bool,
}

impl CompletionList {
    /// 新しい補完リストを作成する
    pub fn new() -> Self {
        Self::default()
    }

    /// 補完アイテムを追加する
    pub fn add(&mut self, item: CompletionItem) {
        self.items.push(item);
    }

    /// すべてのアイテムを取得する
    pub fn items(&self) -> &[CompletionItem] {
        &self.items
    }

    /// アイテム数を取得する
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 空かどうか
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// フィルターテキストでフィルタリングする
    pub fn filter(&self, query: &str) -> Vec<&CompletionItem> {
        let query_lower = query.to_lowercase();
        self.items
            .iter()
            .filter(|item| {
                let filter_text = item.filter_text.as_deref().unwrap_or(&item.label);
                filter_text.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// ソートテキストでソートする
    pub fn sorted(&self) -> Vec<&CompletionItem> {
        let mut items: Vec<_> = self.items.iter().collect();
        items.sort_by(|a, b| {
            let a_sort = a.sort_text.as_deref().unwrap_or(&a.label);
            let b_sort = b.sort_text.as_deref().unwrap_or(&b.label);
            a_sort.cmp(b_sort)
        });
        items
    }
}

impl From<lsp_types::CompletionResponse> for CompletionList {
    fn from(response: lsp_types::CompletionResponse) -> Self {
        match response {
            lsp_types::CompletionResponse::Array(items) => {
                let mut list = CompletionList::new();
                for item in items {
                    list.add(item.into());
                }
                list
            }
            lsp_types::CompletionResponse::List(completion_list) => {
                let mut list = CompletionList {
                    items: Vec::new(),
                    is_incomplete: completion_list.is_incomplete,
                };
                for item in completion_list.items {
                    list.add(item.into());
                }
                list
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_kind_display_name() {
        assert_eq!(CompletionKind::Function.display_name(), "関数");
        assert_eq!(CompletionKind::Variable.display_name(), "変数");
        assert_eq!(CompletionKind::Keyword.display_name(), "キーワード");
        assert_eq!(CompletionKind::Struct.display_name(), "構造体");
    }

    #[test]
    fn test_completion_kind_icon() {
        assert_eq!(CompletionKind::Function.icon(), "ƒ");
        assert_eq!(CompletionKind::Variable.icon(), "𝑥");
        assert_eq!(CompletionKind::Keyword.icon(), "🔤");
    }

    #[test]
    fn test_completion_kind_from_lsp() {
        let lsp_kind = lsp_types::CompletionItemKind::FUNCTION;
        let kind: CompletionKind = lsp_kind.into();
        assert_eq!(kind, CompletionKind::Function);

        let lsp_kind = lsp_types::CompletionItemKind::STRUCT;
        let kind: CompletionKind = lsp_kind.into();
        assert_eq!(kind, CompletionKind::Struct);
    }

    #[test]
    fn test_completion_item_creation() {
        let item = CompletionItem::new("test_function", CompletionKind::Function)
            .with_detail("fn test_function()")
            .with_documentation("テスト関数です");

        assert_eq!(item.label, "test_function");
        assert_eq!(item.kind, CompletionKind::Function);
        assert_eq!(item.detail, Some("fn test_function()".to_string()));
        assert_eq!(item.documentation, Some("テスト関数です".to_string()));
    }

    #[test]
    fn test_completion_item_text_to_insert() {
        // insert_textがない場合はlabelを使用
        let item = CompletionItem::new("println!", CompletionKind::Macro);
        assert_eq!(item.text_to_insert(), "println!");

        // insert_textがある場合はそれを使用
        let item = CompletionItem::new("println!", CompletionKind::Macro)
            .with_insert_text("println!(\"$1\")");
        assert_eq!(item.text_to_insert(), "println!(\"$1\")");
    }

    #[test]
    fn test_completion_item_formatted_display() {
        let item = CompletionItem::new("test_function", CompletionKind::Function)
            .with_detail("fn()");
        let display = item.formatted_display();
        assert!(display.contains("test_function"));
        assert!(display.contains("fn()"));
        assert!(display.contains("ƒ"));
    }

    #[test]
    fn test_completion_item_from_lsp() {
        let lsp_item = lsp_types::CompletionItem {
            label: "test_method".to_string(),
            kind: Some(lsp_types::CompletionItemKind::METHOD),
            detail: Some("fn test_method(&self)".to_string()),
            ..Default::default()
        };

        let item: CompletionItem = lsp_item.into();
        assert_eq!(item.label, "test_method");
        assert_eq!(item.kind, CompletionKind::Method);
        assert_eq!(item.detail, Some("fn test_method(&self)".to_string()));
    }

    #[test]
    fn test_completion_list_creation() {
        let mut list = CompletionList::new();
        assert!(list.is_empty());

        list.add(CompletionItem::new("item1", CompletionKind::Function));
        list.add(CompletionItem::new("item2", CompletionKind::Variable));

        assert_eq!(list.len(), 2);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_completion_list_filter() {
        let mut list = CompletionList::new();
        list.add(CompletionItem::new("test_function", CompletionKind::Function));
        list.add(CompletionItem::new("another_function", CompletionKind::Function));
        list.add(CompletionItem::new("test_variable", CompletionKind::Variable));

        let filtered = list.filter("test");
        assert_eq!(filtered.len(), 2);

        let filtered = list.filter("function");
        assert_eq!(filtered.len(), 2);

        let filtered = list.filter("another");
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_completion_list_filter_case_insensitive() {
        let mut list = CompletionList::new();
        list.add(CompletionItem::new("TestFunction", CompletionKind::Function));
        list.add(CompletionItem::new("testVariable", CompletionKind::Variable));

        let filtered = list.filter("test");
        assert_eq!(filtered.len(), 2);

        let filtered = list.filter("TEST");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_completion_list_sorted() {
        let mut list = CompletionList::new();
        list.add(CompletionItem::new("zebra", CompletionKind::Function));
        list.add(CompletionItem::new("apple", CompletionKind::Function));
        list.add(CompletionItem::new("mango", CompletionKind::Function));

        let sorted = list.sorted();
        assert_eq!(sorted[0].label, "apple");
        assert_eq!(sorted[1].label, "mango");
        assert_eq!(sorted[2].label, "zebra");
    }

    #[test]
    fn test_completion_list_from_lsp_array() {
        let lsp_response = lsp_types::CompletionResponse::Array(vec![
            lsp_types::CompletionItem {
                label: "item1".to_string(),
                kind: Some(lsp_types::CompletionItemKind::FUNCTION),
                ..Default::default()
            },
            lsp_types::CompletionItem {
                label: "item2".to_string(),
                kind: Some(lsp_types::CompletionItemKind::VARIABLE),
                ..Default::default()
            },
        ]);

        let list: CompletionList = lsp_response.into();
        assert_eq!(list.len(), 2);
        assert!(!list.is_incomplete);
    }

    #[test]
    fn test_completion_list_from_lsp_list() {
        let lsp_response = lsp_types::CompletionResponse::List(lsp_types::CompletionList {
            is_incomplete: true,
            items: vec![lsp_types::CompletionItem {
                label: "partial_item".to_string(),
                ..Default::default()
            }],
        });

        let list: CompletionList = lsp_response.into();
        assert_eq!(list.len(), 1);
        assert!(list.is_incomplete);
    }
}
