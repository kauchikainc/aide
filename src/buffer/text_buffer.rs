//! テキストバッファ実装
//!
//! ropeyのRopeをラップし、egui TextBuffer traitを実装する。

use ropey::Rope;
use std::ops::Range;
use std::path::Path;

/// テキストバッファ
///
/// ropeyのRopeをラップし、効率的なテキスト編集操作を提供する。
/// 大きなファイルでもO(log n)での操作が可能。
pub struct TextBuffer {
    /// 内部のRopeデータ構造
    rope: Rope,
    /// ファイルパス（存在する場合）
    file_path: Option<std::path::PathBuf>,
    /// 変更フラグ
    modified: bool,
}

impl TextBuffer {
    /// 空のテキストバッファを作成する
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            file_path: None,
            modified: false,
        }
    }

    /// 文字列からテキストバッファを作成する
    pub fn from_str(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            file_path: None,
            modified: false,
        }
    }

    /// ファイルからテキストバッファを読み込む
    pub fn from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = path.as_ref();
        let rope = Rope::from_reader(std::fs::File::open(path)?)?;
        Ok(Self {
            rope,
            file_path: Some(path.to_path_buf()),
            modified: false,
        })
    }

    /// テキストの長さをバイト数で返す
    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    /// テキストの長さを文字数で返す
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// テキストが空かどうかを返す
    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    /// 行数を返す
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// 指定位置にテキストを挿入する（文字インデックス）
    pub fn insert(&mut self, char_idx: usize, text: &str) {
        // 範囲チェック
        let len = self.rope.len_chars();
        let idx = char_idx.min(len);
        self.rope.insert(idx, text);
        self.modified = true;
    }

    /// 指定範囲のテキストを削除する（文字インデックス）
    pub fn remove(&mut self, range: Range<usize>) {
        let len = self.rope.len_chars();
        // 範囲を有効な範囲にクランプ
        let start = range.start.min(len);
        let end = range.end.min(len);
        if start < end {
            self.rope.remove(start..end);
            self.modified = true;
        }
    }

    /// 指定範囲のテキストを取得する（文字インデックス）
    pub fn slice(&self, range: Range<usize>) -> String {
        let len = self.rope.len_chars();
        let start = range.start.min(len);
        let end = range.end.min(len);
        if start < end {
            self.rope.slice(start..end).to_string()
        } else {
            String::new()
        }
    }

    /// 全テキストを文字列として取得する
    pub fn to_string(&self) -> String {
        self.rope.to_string()
    }

    /// 文字インデックスをバイトインデックスに変換する
    pub fn char_to_byte(&self, char_idx: usize) -> usize {
        let len = self.rope.len_chars();
        if char_idx >= len {
            self.rope.len_bytes()
        } else {
            self.rope.char_to_byte(char_idx)
        }
    }

    /// バイトインデックスを文字インデックスに変換する
    pub fn byte_to_char(&self, byte_idx: usize) -> usize {
        let len = self.rope.len_bytes();
        if byte_idx >= len {
            self.rope.len_chars()
        } else {
            self.rope.byte_to_char(byte_idx)
        }
    }

    /// 行インデックスからその行の開始文字インデックスを取得する
    pub fn line_to_char(&self, line_idx: usize) -> usize {
        let len_lines = self.rope.len_lines();
        if line_idx >= len_lines {
            self.rope.len_chars()
        } else {
            self.rope.line_to_char(line_idx)
        }
    }

    /// 文字インデックスから行インデックスを取得する
    pub fn char_to_line(&self, char_idx: usize) -> usize {
        let len = self.rope.len_chars();
        if char_idx >= len {
            self.rope.len_lines().saturating_sub(1)
        } else {
            self.rope.char_to_line(char_idx)
        }
    }

    /// 指定行のテキストを取得する
    pub fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx < self.rope.len_lines() {
            Some(self.rope.line(line_idx).to_string())
        } else {
            None
        }
    }

    /// ファイルに保存する
    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(ref path) = self.file_path {
            let content = self.rope.to_string();
            std::fs::write(path, content)?;
            self.modified = false;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "ファイルパスが設定されていません",
            ))
        }
    }

    /// 指定パスにファイルを保存する
    pub fn save_as<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let path = path.as_ref();
        let content = self.rope.to_string();
        std::fs::write(path, &content)?;
        self.file_path = Some(path.to_path_buf());
        self.modified = false;
        Ok(())
    }

    /// 変更されているかどうかを返す
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// ファイルパスを取得する
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    /// ファイル名を取得する
    pub fn file_name(&self) -> Option<&str> {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

// egui TextBuffer trait実装
impl egui::TextBuffer for TextBuffer {
    /// このバッファが変更可能かどうかを返す
    fn is_mutable(&self) -> bool {
        true
    }

    /// テキストバッファへの参照として文字列を返す
    /// eguiはas_strでimmutableな参照を要求するが、
    /// Ropeは直接&strを返せないため、内部的にキャッシュする必要がある
    /// ただし、この実装では毎回新しいStringを作成するため非効率
    /// 実用上はeguiの組み込みエディタの代わりにカスタム実装を使うべき
    fn as_str(&self) -> &str {
        // この実装は問題がある：&strを返す必要があるが、Ropeからは直接返せない
        // 暫定的にleakを使用（本番では別アプローチが必要）
        // TODO: より良い方法を検討
        let s = self.rope.to_string();
        Box::leak(s.into_boxed_str())
    }

    /// 指定範囲のテキストを新しいテキストに置換する
    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        self.insert(char_index, text);
        text.chars().count()
    }

    /// 指定範囲の文字を削除する
    fn delete_char_range(&mut self, char_range: Range<usize>) {
        self.remove(char_range);
    }

    /// 型IDを返す
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    // ==================== 基本操作テスト ====================

    #[test]
    fn test_new_buffer_is_empty() {
        // 新規バッファは空であること
        let buffer = TextBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len_chars(), 0);
        assert_eq!(buffer.len_bytes(), 0);
    }

    #[test]
    fn test_from_str() {
        // 文字列からバッファを作成できること
        let text = "Hello, World!";
        let buffer = TextBuffer::from_str(text);
        assert_eq!(buffer.to_string(), text);
        assert_eq!(buffer.len_chars(), 13);
    }

    // ==================== 挿入操作テスト ====================

    #[test]
    fn test_insert_at_beginning() {
        // 先頭への挿入が正しく動作すること
        let mut buffer = TextBuffer::from_str("World");
        buffer.insert(0, "Hello, ");
        assert_eq!(buffer.to_string(), "Hello, World");
    }

    #[test]
    fn test_insert_at_end() {
        // 末尾への挿入が正しく動作すること
        let mut buffer = TextBuffer::from_str("Hello");
        buffer.insert(5, ", World!");
        assert_eq!(buffer.to_string(), "Hello, World!");
    }

    #[test]
    fn test_insert_in_middle() {
        // 中間への挿入が正しく動作すること
        let mut buffer = TextBuffer::from_str("HeWorld");
        buffer.insert(2, "llo, ");
        assert_eq!(buffer.to_string(), "Hello, World");
    }

    #[test]
    fn test_insert_empty_string() {
        // 空文字列の挿入は内容を変更しないこと
        let mut buffer = TextBuffer::from_str("Hello");
        buffer.insert(2, "");
        assert_eq!(buffer.to_string(), "Hello");
    }

    #[test]
    fn test_insert_beyond_length() {
        // バッファ長を超える位置への挿入は末尾に挿入されること
        let mut buffer = TextBuffer::from_str("Hello");
        buffer.insert(100, " World");
        assert_eq!(buffer.to_string(), "Hello World");
    }

    // ==================== 削除操作テスト ====================

    #[test]
    fn test_remove_from_beginning() {
        // 先頭からの削除が正しく動作すること
        let mut buffer = TextBuffer::from_str("Hello, World!");
        buffer.remove(0..7);
        assert_eq!(buffer.to_string(), "World!");
    }

    #[test]
    fn test_remove_from_end() {
        // 末尾からの削除が正しく動作すること
        let mut buffer = TextBuffer::from_str("Hello, World!");
        buffer.remove(5..13);
        assert_eq!(buffer.to_string(), "Hello");
    }

    #[test]
    fn test_remove_from_middle() {
        // 中間からの削除が正しく動作すること
        let mut buffer = TextBuffer::from_str("Hello, World!");
        buffer.remove(5..7);
        assert_eq!(buffer.to_string(), "HelloWorld!");
    }

    #[test]
    fn test_remove_empty_range() {
        // 空範囲の削除は内容を変更しないこと
        let mut buffer = TextBuffer::from_str("Hello");
        buffer.remove(2..2);
        assert_eq!(buffer.to_string(), "Hello");
    }

    #[test]
    fn test_remove_beyond_length() {
        // バッファ長を超える範囲の削除はクランプされること
        let mut buffer = TextBuffer::from_str("Hello");
        buffer.remove(3..100);
        assert_eq!(buffer.to_string(), "Hel");
    }

    #[test]
    fn test_remove_all() {
        // 全削除が正しく動作すること
        let mut buffer = TextBuffer::from_str("Hello");
        buffer.remove(0..5);
        assert!(buffer.is_empty());
    }

    // ==================== UTF-8マルチバイト文字テスト ====================

    #[test]
    fn test_multibyte_japanese() {
        // 日本語テキストの基本操作が正しく動作すること
        let text = "こんにちは世界";
        let buffer = TextBuffer::from_str(text);
        assert_eq!(buffer.len_chars(), 7);
        assert_eq!(buffer.to_string(), text);
    }

    #[test]
    fn test_multibyte_insert() {
        // 日本語テキストへの挿入が正しく動作すること
        let mut buffer = TextBuffer::from_str("世界");
        buffer.insert(0, "こんにちは");
        assert_eq!(buffer.to_string(), "こんにちは世界");
    }

    #[test]
    fn test_multibyte_remove() {
        // 日本語テキストからの削除が正しく動作すること
        let mut buffer = TextBuffer::from_str("こんにちは世界");
        buffer.remove(0..5);
        assert_eq!(buffer.to_string(), "世界");
    }

    #[test]
    fn test_multibyte_slice() {
        // 日本語テキストのスライスが正しく動作すること
        let buffer = TextBuffer::from_str("こんにちは世界");
        let slice = buffer.slice(5..7);
        assert_eq!(slice, "世界");
    }

    #[test]
    fn test_mixed_ascii_multibyte() {
        // ASCII文字と日本語の混在が正しく処理されること
        let text = "Hello, 世界! 123";
        let buffer = TextBuffer::from_str(text);
        assert_eq!(buffer.len_chars(), 14);

        let slice = buffer.slice(7..9);
        assert_eq!(slice, "世界");
    }

    #[test]
    fn test_emoji() {
        // 絵文字が正しく処理されること
        let text = "Hello 👋 World 🌍";
        let buffer = TextBuffer::from_str(text);
        assert!(buffer.to_string().contains("👋"));
        assert!(buffer.to_string().contains("🌍"));
    }

    // ==================== 行操作テスト ====================

    #[test]
    fn test_line_count() {
        // 行数が正しくカウントされること
        let buffer = TextBuffer::from_str("Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.len_lines(), 3);
    }

    #[test]
    fn test_get_line() {
        // 指定行が正しく取得できること
        let buffer = TextBuffer::from_str("Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.line(0).unwrap(), "Line 1\n");
        assert_eq!(buffer.line(1).unwrap(), "Line 2\n");
        assert_eq!(buffer.line(2).unwrap(), "Line 3");
    }

    #[test]
    fn test_line_to_char() {
        // 行インデックスから文字インデックスへの変換が正しく動作すること
        let buffer = TextBuffer::from_str("abc\ndef\nghi");
        assert_eq!(buffer.line_to_char(0), 0);
        assert_eq!(buffer.line_to_char(1), 4);
        assert_eq!(buffer.line_to_char(2), 8);
    }

    #[test]
    fn test_char_to_line() {
        // 文字インデックスから行インデックスへの変換が正しく動作すること
        let buffer = TextBuffer::from_str("abc\ndef\nghi");
        assert_eq!(buffer.char_to_line(0), 0);
        assert_eq!(buffer.char_to_line(3), 0);
        assert_eq!(buffer.char_to_line(4), 1);
        assert_eq!(buffer.char_to_line(8), 2);
    }

    // ==================== ファイル操作テスト ====================

    #[test]
    fn test_from_file() {
        // ファイルからの読み込みが正しく動作すること
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Hello from file!").unwrap();

        let buffer = TextBuffer::from_file(file.path()).unwrap();
        assert!(buffer.to_string().contains("Hello from file!"));
        assert!(!buffer.is_modified());
    }

    #[test]
    fn test_save_as() {
        // 名前を付けて保存が正しく動作すること
        let file = NamedTempFile::new().unwrap();
        let mut buffer = TextBuffer::from_str("Test content");

        buffer.save_as(file.path()).unwrap();

        let content = std::fs::read_to_string(file.path()).unwrap();
        assert_eq!(content, "Test content");
        assert!(!buffer.is_modified());
    }

    #[test]
    fn test_modified_flag() {
        // 変更フラグが正しく管理されること
        let mut buffer = TextBuffer::new();
        assert!(!buffer.is_modified());

        buffer.insert(0, "text");
        assert!(buffer.is_modified());

        let file = NamedTempFile::new().unwrap();
        buffer.save_as(file.path()).unwrap();
        assert!(!buffer.is_modified());
    }

    // ==================== 大きなファイルのテスト ====================

    #[test]
    fn test_large_file_creation() {
        // 大きなテキスト（1MB+）の作成が正しく動作すること
        let line = "This is a test line with some content.\n";
        let line_count = 30_000; // 約1.2MB
        let large_text: String = line.repeat(line_count);

        let buffer = TextBuffer::from_str(&large_text);
        // 末尾の改行後の空行も含むため +1 になる
        assert_eq!(buffer.len_lines(), line_count + 1);
    }

    #[test]
    fn test_large_file_insert() {
        // 大きなファイルへの挿入が正しく動作すること
        let line = "Test line\n";
        let large_text: String = line.repeat(10_000);
        let mut buffer = TextBuffer::from_str(&large_text);

        // 中間への挿入
        let middle = buffer.len_chars() / 2;
        buffer.insert(middle, "INSERTED");

        assert!(buffer.to_string().contains("INSERTED"));
    }

    #[test]
    fn test_large_file_remove() {
        // 大きなファイルからの削除が正しく動作すること
        let line = "Test line\n";
        let large_text: String = line.repeat(10_000);
        let mut buffer = TextBuffer::from_str(&large_text);

        let original_len = buffer.len_chars();
        buffer.remove(0..100);

        assert!(buffer.len_chars() < original_len);
    }

    #[test]
    fn test_large_file_read_from_disk() {
        // 大きなファイルのディスクからの読み込みが正しく動作すること
        let mut file = NamedTempFile::new().unwrap();
        let line = "This is a test line for large file testing.\n";
        for _ in 0..25_000 {
            write!(file, "{}", line).unwrap();
        }
        file.flush().unwrap();

        let buffer = TextBuffer::from_file(file.path()).unwrap();
        // 末尾の改行後の空行も含むため +1 になる
        assert_eq!(buffer.len_lines(), 25_000 + 1);
    }

    // ==================== エッジケーステスト ====================

    #[test]
    fn test_byte_char_conversion() {
        // バイト/文字インデックス変換が正しく動作すること
        let buffer = TextBuffer::from_str("aあb");
        assert_eq!(buffer.len_chars(), 3);
        assert_eq!(buffer.len_bytes(), 5); // 'a'(1) + 'あ'(3) + 'b'(1)

        assert_eq!(buffer.char_to_byte(0), 0);
        assert_eq!(buffer.char_to_byte(1), 1);
        assert_eq!(buffer.char_to_byte(2), 4);

        assert_eq!(buffer.byte_to_char(0), 0);
        assert_eq!(buffer.byte_to_char(1), 1);
        assert_eq!(buffer.byte_to_char(4), 2);
    }
}
