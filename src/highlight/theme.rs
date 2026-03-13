//! カラーテーマ
//!
//! 構文ハイライトのカラーテーマを管理する。
//! egui::Color32を使用して各ハイライトタイプに色を割り当てる。

use egui::Color32;
use std::collections::HashMap;

use super::HighlightType;

/// カラーテーマ
#[derive(Debug, Clone)]
pub struct Theme {
    /// テーマ名
    pub name: String,
    /// 背景色
    pub background: Color32,
    /// 前景色（デフォルトテキスト）
    pub foreground: Color32,
    /// 選択範囲の背景色
    pub selection: Color32,
    /// カーソル色
    pub cursor: Color32,
    /// 行番号の色
    pub line_number: Color32,
    /// 現在行のハイライト色
    pub current_line: Color32,
    /// ハイライトタイプごとの色
    pub highlights: HashMap<HighlightType, Color32>,
}

impl Theme {
    /// 新しいテーマを作成する
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            background: Color32::from_rgb(30, 30, 30),
            foreground: Color32::from_rgb(212, 212, 212),
            selection: Color32::from_rgba_unmultiplied(38, 79, 120, 180),
            cursor: Color32::from_rgb(255, 255, 255),
            line_number: Color32::from_rgb(133, 133, 133),
            current_line: Color32::from_rgba_unmultiplied(255, 255, 255, 10),
            highlights: HashMap::new(),
        }
    }

    /// ハイライトタイプの色を設定する
    pub fn set_highlight_color(&mut self, highlight_type: HighlightType, color: Color32) {
        self.highlights.insert(highlight_type, color);
    }

    /// ハイライトタイプの色を取得する
    pub fn get_highlight_color(&self, highlight_type: HighlightType) -> Color32 {
        self.highlights
            .get(&highlight_type)
            .copied()
            .unwrap_or(self.foreground)
    }

    /// ダークテーマ（VS Code風）を作成する
    pub fn dark() -> Self {
        let mut theme = Self::new("Dark");

        // 基本色
        theme.background = Color32::from_rgb(30, 30, 30);
        theme.foreground = Color32::from_rgb(212, 212, 212);
        theme.selection = Color32::from_rgba_unmultiplied(38, 79, 120, 180);
        theme.cursor = Color32::from_rgb(255, 255, 255);
        theme.line_number = Color32::from_rgb(133, 133, 133);
        theme.current_line = Color32::from_rgba_unmultiplied(255, 255, 255, 10);

        // 構文ハイライト色
        theme.set_highlight_color(HighlightType::Keyword, Color32::from_rgb(86, 156, 214)); // 青
        theme.set_highlight_color(HighlightType::Type, Color32::from_rgb(78, 201, 176)); // ティール
        theme.set_highlight_color(HighlightType::Function, Color32::from_rgb(220, 220, 170)); // 黄
        theme.set_highlight_color(HighlightType::Variable, Color32::from_rgb(156, 220, 254)); // 水色
        theme.set_highlight_color(HighlightType::String, Color32::from_rgb(206, 145, 120)); // オレンジ
        theme.set_highlight_color(HighlightType::Number, Color32::from_rgb(181, 206, 168)); // 薄緑
        theme.set_highlight_color(HighlightType::Comment, Color32::from_rgb(106, 153, 85)); // 緑
        theme.set_highlight_color(HighlightType::Operator, Color32::from_rgb(212, 212, 212)); // 白
        theme.set_highlight_color(HighlightType::Attribute, Color32::from_rgb(86, 156, 214)); // 青
        theme.set_highlight_color(HighlightType::Macro, Color32::from_rgb(86, 156, 214)); // 青
        theme.set_highlight_color(HighlightType::Constant, Color32::from_rgb(79, 193, 255)); // 明るい青
        theme.set_highlight_color(HighlightType::Module, Color32::from_rgb(78, 201, 176)); // ティール
        theme.set_highlight_color(HighlightType::Property, Color32::from_rgb(156, 220, 254)); // 水色
        theme.set_highlight_color(HighlightType::Other, Color32::from_rgb(212, 212, 212)); // 白

        theme
    }

    /// ライトテーマを作成する
    pub fn light() -> Self {
        let mut theme = Self::new("Light");

        // 基本色
        theme.background = Color32::from_rgb(255, 255, 255);
        theme.foreground = Color32::from_rgb(0, 0, 0);
        theme.selection = Color32::from_rgba_unmultiplied(173, 214, 255, 180);
        theme.cursor = Color32::from_rgb(0, 0, 0);
        theme.line_number = Color32::from_rgb(133, 133, 133);
        theme.current_line = Color32::from_rgba_unmultiplied(0, 0, 0, 10);

        // 構文ハイライト色
        theme.set_highlight_color(HighlightType::Keyword, Color32::from_rgb(0, 0, 255)); // 青
        theme.set_highlight_color(HighlightType::Type, Color32::from_rgb(38, 127, 153)); // ティール
        theme.set_highlight_color(HighlightType::Function, Color32::from_rgb(121, 94, 38)); // 茶
        theme.set_highlight_color(HighlightType::Variable, Color32::from_rgb(0, 16, 128)); // 紺
        theme.set_highlight_color(HighlightType::String, Color32::from_rgb(163, 21, 21)); // 赤
        theme.set_highlight_color(HighlightType::Number, Color32::from_rgb(9, 134, 88)); // 緑
        theme.set_highlight_color(HighlightType::Comment, Color32::from_rgb(0, 128, 0)); // 緑
        theme.set_highlight_color(HighlightType::Operator, Color32::from_rgb(0, 0, 0)); // 黒
        theme.set_highlight_color(HighlightType::Attribute, Color32::from_rgb(0, 0, 255)); // 青
        theme.set_highlight_color(HighlightType::Macro, Color32::from_rgb(0, 0, 255)); // 青
        theme.set_highlight_color(HighlightType::Constant, Color32::from_rgb(0, 16, 128)); // 紺
        theme.set_highlight_color(HighlightType::Module, Color32::from_rgb(38, 127, 153)); // ティール
        theme.set_highlight_color(HighlightType::Property, Color32::from_rgb(0, 16, 128)); // 紺
        theme.set_highlight_color(HighlightType::Other, Color32::from_rgb(0, 0, 0)); // 黒

        theme
    }

    /// Monokai風テーマを作成する
    pub fn monokai() -> Self {
        let mut theme = Self::new("Monokai");

        // 基本色
        theme.background = Color32::from_rgb(39, 40, 34);
        theme.foreground = Color32::from_rgb(248, 248, 242);
        theme.selection = Color32::from_rgba_unmultiplied(73, 72, 62, 200);
        theme.cursor = Color32::from_rgb(248, 248, 240);
        theme.line_number = Color32::from_rgb(144, 144, 138);
        theme.current_line = Color32::from_rgba_unmultiplied(255, 255, 255, 10);

        // 構文ハイライト色
        theme.set_highlight_color(HighlightType::Keyword, Color32::from_rgb(249, 38, 114)); // ピンク
        theme.set_highlight_color(HighlightType::Type, Color32::from_rgb(102, 217, 239)); // 水色
        theme.set_highlight_color(HighlightType::Function, Color32::from_rgb(166, 226, 46)); // 黄緑
        theme.set_highlight_color(HighlightType::Variable, Color32::from_rgb(248, 248, 242)); // 白
        theme.set_highlight_color(HighlightType::String, Color32::from_rgb(230, 219, 116)); // 黄
        theme.set_highlight_color(HighlightType::Number, Color32::from_rgb(174, 129, 255)); // 紫
        theme.set_highlight_color(HighlightType::Comment, Color32::from_rgb(117, 113, 94)); // グレー
        theme.set_highlight_color(HighlightType::Operator, Color32::from_rgb(249, 38, 114)); // ピンク
        theme.set_highlight_color(HighlightType::Attribute, Color32::from_rgb(166, 226, 46)); // 黄緑
        theme.set_highlight_color(HighlightType::Macro, Color32::from_rgb(166, 226, 46)); // 黄緑
        theme.set_highlight_color(HighlightType::Constant, Color32::from_rgb(174, 129, 255)); // 紫
        theme.set_highlight_color(HighlightType::Module, Color32::from_rgb(102, 217, 239)); // 水色
        theme.set_highlight_color(HighlightType::Property, Color32::from_rgb(248, 248, 242)); // 白
        theme.set_highlight_color(HighlightType::Other, Color32::from_rgb(248, 248, 242)); // 白

        theme
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        // テーマが正しく作成されること
        let theme = Theme::new("Test");
        assert_eq!(theme.name, "Test");
    }

    #[test]
    fn test_dark_theme() {
        // ダークテーマが正しく設定されること
        let theme = Theme::dark();
        assert_eq!(theme.name, "Dark");

        // 背景色が暗いこと
        assert!(theme.background.r() < 100);
        assert!(theme.background.g() < 100);
        assert!(theme.background.b() < 100);

        // キーワード色が設定されていること
        let keyword_color = theme.get_highlight_color(HighlightType::Keyword);
        assert_ne!(keyword_color, theme.foreground);
    }

    #[test]
    fn test_light_theme() {
        // ライトテーマが正しく設定されること
        let theme = Theme::light();
        assert_eq!(theme.name, "Light");

        // 背景色が明るいこと
        assert!(theme.background.r() > 200);
        assert!(theme.background.g() > 200);
        assert!(theme.background.b() > 200);
    }

    #[test]
    fn test_monokai_theme() {
        // Monokaiテーマが正しく設定されること
        let theme = Theme::monokai();
        assert_eq!(theme.name, "Monokai");
    }

    #[test]
    fn test_set_highlight_color() {
        // ハイライト色が設定できること
        let mut theme = Theme::new("Test");
        let color = Color32::from_rgb(255, 0, 0);

        theme.set_highlight_color(HighlightType::Keyword, color);
        assert_eq!(theme.get_highlight_color(HighlightType::Keyword), color);
    }

    #[test]
    fn test_get_highlight_color_default() {
        // 未設定のハイライトタイプはフォアグラウンド色を返すこと
        let theme = Theme::new("Test");
        let color = theme.get_highlight_color(HighlightType::Keyword);
        assert_eq!(color, theme.foreground);
    }

    #[test]
    fn test_default_theme_is_dark() {
        // デフォルトテーマがダークであること
        let theme = Theme::default();
        assert_eq!(theme.name, "Dark");
    }
}
