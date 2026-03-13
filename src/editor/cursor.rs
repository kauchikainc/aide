//! カーソル管理
//!
//! エディター内のカーソル位置と選択範囲を管理する。

/// カーソル位置を表す構造体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cursor {
    /// 行番号（0始まり）
    pub line: usize,
    /// 列番号（文字単位、0始まり）
    pub column: usize,
}

impl Cursor {
    /// 新しいカーソルを作成する
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// 原点（0, 0）のカーソルを返す
    pub fn origin() -> Self {
        Self { line: 0, column: 0 }
    }
}

/// 選択範囲を表す構造体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// 選択開始位置
    pub start: Cursor,
    /// 選択終了位置（カーソル位置）
    pub end: Cursor,
}

impl Selection {
    /// 新しい選択範囲を作成する
    pub fn new(start: Cursor, end: Cursor) -> Self {
        Self { start, end }
    }

    /// カーソル位置のみの選択（選択なし）を作成する
    pub fn cursor(cursor: Cursor) -> Self {
        Self {
            start: cursor,
            end: cursor,
        }
    }

    /// 選択範囲があるかどうかを返す
    pub fn has_selection(&self) -> bool {
        self.start != self.end
    }

    /// 正規化された選択範囲を返す（startが常にend以前になる）
    pub fn normalized(&self) -> Self {
        if self.start.line > self.end.line
            || (self.start.line == self.end.line && self.start.column > self.end.column)
        {
            Self {
                start: self.end,
                end: self.start,
            }
        } else {
            *self
        }
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::cursor(Cursor::origin())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_creation() {
        let cursor = Cursor::new(5, 10);
        assert_eq!(cursor.line, 5);
        assert_eq!(cursor.column, 10);
    }

    #[test]
    fn test_selection_has_selection() {
        let no_selection = Selection::cursor(Cursor::new(1, 1));
        assert!(!no_selection.has_selection());

        let with_selection = Selection::new(Cursor::new(1, 1), Cursor::new(2, 5));
        assert!(with_selection.has_selection());
    }

    #[test]
    fn test_selection_normalized() {
        // 逆順の選択
        let selection = Selection::new(Cursor::new(5, 10), Cursor::new(2, 3));
        let normalized = selection.normalized();

        assert_eq!(normalized.start.line, 2);
        assert_eq!(normalized.start.column, 3);
        assert_eq!(normalized.end.line, 5);
        assert_eq!(normalized.end.column, 10);
    }
}
