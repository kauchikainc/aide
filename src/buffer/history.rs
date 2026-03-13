//! Undo/Redo履歴管理
//!
//! Commandパターンを使用した編集履歴の管理を提供する。
//! 連続した入力操作のグループ化にも対応。

use std::ops::Range;

/// 編集操作を表すコマンド
#[derive(Debug, Clone, PartialEq)]
pub enum EditCommand {
    /// テキストの挿入
    Insert {
        /// 挿入位置（文字インデックス）
        position: usize,
        /// 挿入されたテキスト
        text: String,
    },
    /// テキストの削除
    Delete {
        /// 削除範囲（文字インデックス）
        range: Range<usize>,
        /// 削除されたテキスト
        deleted_text: String,
    },
    /// 複数操作のグループ
    Group {
        /// グループ化された操作のリスト
        commands: Vec<EditCommand>,
    },
}

impl EditCommand {
    /// 挿入コマンドを作成する
    pub fn insert(position: usize, text: impl Into<String>) -> Self {
        EditCommand::Insert {
            position,
            text: text.into(),
        }
    }

    /// 削除コマンドを作成する
    pub fn delete(range: Range<usize>, deleted_text: impl Into<String>) -> Self {
        EditCommand::Delete {
            range,
            deleted_text: deleted_text.into(),
        }
    }

    /// グループコマンドを作成する
    pub fn group(commands: Vec<EditCommand>) -> Self {
        EditCommand::Group { commands }
    }

    /// このコマンドの逆操作を生成する
    pub fn inverse(&self) -> Self {
        match self {
            EditCommand::Insert { position, text } => EditCommand::Delete {
                range: *position..(*position + text.chars().count()),
                deleted_text: text.clone(),
            },
            EditCommand::Delete { range, deleted_text } => EditCommand::Insert {
                position: range.start,
                text: deleted_text.clone(),
            },
            EditCommand::Group { commands } => EditCommand::Group {
                // 逆順で逆操作を生成
                commands: commands.iter().rev().map(|c| c.inverse()).collect(),
            },
        }
    }
}

/// 編集履歴を管理する構造体
pub struct History {
    /// Undo用のスタック
    undo_stack: Vec<EditCommand>,
    /// Redo用のスタック
    redo_stack: Vec<EditCommand>,
    /// 履歴の最大サイズ
    max_size: usize,
    /// 連続入力をグループ化するための一時バッファ
    pending_group: Option<PendingGroup>,
}

/// 連続入力をグループ化するための一時構造
struct PendingGroup {
    /// グループ化対象の操作
    commands: Vec<EditCommand>,
    /// 最後の操作のタイプ（挿入か削除か）
    last_type: EditType,
    /// 最後の操作位置
    last_position: usize,
}

#[derive(PartialEq, Clone, Copy)]
enum EditType {
    Insert,
    Delete,
}

impl History {
    /// 新しい履歴管理を作成する
    ///
    /// # 引数
    /// * `max_size` - 保持する最大履歴数（デフォルト: 1000）
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
            pending_group: None,
        }
    }

    /// デフォルトサイズの履歴管理を作成する
    pub fn with_default_size() -> Self {
        Self::new(1000)
    }

    /// コマンドを記録する
    ///
    /// 連続した同じタイプの操作は自動的にグループ化される
    pub fn record(&mut self, command: EditCommand) {
        // 新しい操作を記録するとRedoスタックはクリアされる
        self.redo_stack.clear();

        // グループ化の判定
        let should_group = self.should_group_with_pending(&command);

        if should_group {
            // 既存のグループに追加
            if let Some(ref mut pending) = self.pending_group {
                pending.commands.push(command.clone());
                self.update_pending_position(&command);
            }
        } else {
            // 既存のグループをフラッシュ
            self.flush_pending_group();

            // 新しいグループを開始
            let edit_type = match &command {
                EditCommand::Insert { .. } => EditType::Insert,
                EditCommand::Delete { .. } => EditType::Delete,
                EditCommand::Group { .. } => {
                    // グループはそのまま追加
                    self.push_undo(command);
                    return;
                }
            };

            // last_positionは「次のコマンドが来るべき位置」を表す
            let last_position = match &command {
                EditCommand::Insert { position, text } => *position + text.chars().count(),
                EditCommand::Delete { range, .. } => range.start,
                _ => 0,
            };

            self.pending_group = Some(PendingGroup {
                commands: vec![command],
                last_type: edit_type,
                last_position,
            });
        }
    }

    /// 保留中のグループを確定する
    pub fn flush_pending_group(&mut self) {
        if let Some(pending) = self.pending_group.take() {
            if pending.commands.len() == 1 {
                self.push_undo(pending.commands.into_iter().next().unwrap());
            } else if !pending.commands.is_empty() {
                self.push_undo(EditCommand::Group {
                    commands: pending.commands,
                });
            }
        }
    }

    /// Undo操作を実行し、逆操作を返す
    pub fn undo(&mut self) -> Option<EditCommand> {
        self.flush_pending_group();

        let command = self.undo_stack.pop()?;
        let inverse = command.inverse();
        self.redo_stack.push(command);
        Some(inverse)
    }

    /// Redo操作を実行し、再実行する操作を返す
    pub fn redo(&mut self) -> Option<EditCommand> {
        self.flush_pending_group();

        let command = self.redo_stack.pop()?;
        let cloned = command.clone();
        self.undo_stack.push(command);
        Some(cloned)
    }

    /// Undoが可能かどうかを返す
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty() || self.pending_group.is_some()
    }

    /// Redoが可能かどうかを返す
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// 履歴をクリアする
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.pending_group = None;
    }

    /// Undoスタックのサイズを返す
    pub fn undo_count(&self) -> usize {
        let pending_count = if self.pending_group.is_some() { 1 } else { 0 };
        self.undo_stack.len() + pending_count
    }

    /// Redoスタックのサイズを返す
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    // ==================== プライベートメソッド ====================

    /// コマンドを保留中のグループと結合すべきかどうかを判定する
    fn should_group_with_pending(&self, command: &EditCommand) -> bool {
        let Some(ref pending) = self.pending_group else {
            return false;
        };

        match command {
            EditCommand::Insert { position, text } => {
                // 挿入操作は連続している場合にグループ化
                // 条件: 前回も挿入で、位置が連続している
                if pending.last_type != EditType::Insert {
                    return false;
                }

                // 改行が含まれる場合はグループを分ける
                if text.contains('\n') {
                    return false;
                }

                // 空白文字の後の通常文字は分ける（単語単位でグループ化）
                let last_char_is_space = pending.commands.last().map_or(false, |cmd| {
                    if let EditCommand::Insert { text, .. } = cmd {
                        text.chars().last().map_or(false, |c| c.is_whitespace())
                    } else {
                        false
                    }
                });
                let current_is_space = text.chars().next().map_or(false, |c| c.is_whitespace());
                if last_char_is_space && !current_is_space {
                    return false;
                }

                // 位置が連続している場合のみグループ化
                // last_positionは既に「次に挿入されるべき位置」を表している
                *position == pending.last_position
            }
            EditCommand::Delete { range, .. } => {
                // 削除操作は連続している場合にグループ化
                if pending.last_type != EditType::Delete {
                    return false;
                }

                // Backspace: 前の位置から連続して削除
                // Delete: 同じ位置から削除
                range.end == pending.last_position || range.start == pending.last_position
            }
            EditCommand::Group { .. } => false,
        }
    }

    /// 保留中グループの最後の位置を更新する
    fn update_pending_position(&mut self, command: &EditCommand) {
        if let Some(ref mut pending) = self.pending_group {
            pending.last_position = match command {
                EditCommand::Insert { position, text } => *position + text.chars().count(),
                EditCommand::Delete { range, .. } => range.start,
                EditCommand::Group { .. } => pending.last_position,
            };
        }
    }

    /// Undoスタックにプッシュし、サイズを制限する
    fn push_undo(&mut self, command: EditCommand) {
        self.undo_stack.push(command);
        while self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
    }
}

impl Default for History {
    fn default() -> Self {
        Self::with_default_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== EditCommandテスト ====================

    #[test]
    fn test_insert_command_creation() {
        // 挿入コマンドが正しく作成されること
        let cmd = EditCommand::insert(5, "hello");
        match cmd {
            EditCommand::Insert { position, text } => {
                assert_eq!(position, 5);
                assert_eq!(text, "hello");
            }
            _ => panic!("Expected Insert command"),
        }
    }

    #[test]
    fn test_delete_command_creation() {
        // 削除コマンドが正しく作成されること
        let cmd = EditCommand::delete(5..10, "world");
        match cmd {
            EditCommand::Delete { range, deleted_text } => {
                assert_eq!(range, 5..10);
                assert_eq!(deleted_text, "world");
            }
            _ => panic!("Expected Delete command"),
        }
    }

    #[test]
    fn test_insert_inverse() {
        // 挿入の逆操作が削除であること
        let cmd = EditCommand::insert(5, "hello");
        let inverse = cmd.inverse();
        match inverse {
            EditCommand::Delete { range, deleted_text } => {
                assert_eq!(range, 5..10);
                assert_eq!(deleted_text, "hello");
            }
            _ => panic!("Expected Delete command as inverse"),
        }
    }

    #[test]
    fn test_delete_inverse() {
        // 削除の逆操作が挿入であること
        let cmd = EditCommand::delete(5..10, "world");
        let inverse = cmd.inverse();
        match inverse {
            EditCommand::Insert { position, text } => {
                assert_eq!(position, 5);
                assert_eq!(text, "world");
            }
            _ => panic!("Expected Insert command as inverse"),
        }
    }

    #[test]
    fn test_group_inverse() {
        // グループの逆操作が正しく逆順で生成されること
        let group = EditCommand::group(vec![
            EditCommand::insert(0, "a"),
            EditCommand::insert(1, "b"),
        ]);
        let inverse = group.inverse();
        match inverse {
            EditCommand::Group { commands } => {
                assert_eq!(commands.len(), 2);
                // 逆順になっていることを確認
                match &commands[0] {
                    EditCommand::Delete { range, deleted_text } => {
                        assert_eq!(*range, 1..2);
                        assert_eq!(deleted_text, "b");
                    }
                    _ => panic!("Expected Delete command"),
                }
                match &commands[1] {
                    EditCommand::Delete { range, deleted_text } => {
                        assert_eq!(*range, 0..1);
                        assert_eq!(deleted_text, "a");
                    }
                    _ => panic!("Expected Delete command"),
                }
            }
            _ => panic!("Expected Group command as inverse"),
        }
    }

    // ==================== Historyテスト ====================

    #[test]
    fn test_empty_history() {
        // 空の履歴ではundo/redoができないこと
        let history = History::new(100);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_single_undo() {
        // 単一操作のundoが正しく動作すること
        let mut history = History::new(100);
        history.record(EditCommand::insert(0, "hello"));
        history.flush_pending_group();

        assert!(history.can_undo());
        let inverse = history.undo().unwrap();

        match inverse {
            EditCommand::Delete { range, deleted_text } => {
                assert_eq!(range, 0..5);
                assert_eq!(deleted_text, "hello");
            }
            _ => panic!("Expected Delete command"),
        }
    }

    #[test]
    fn test_undo_redo_cycle() {
        // undo後にredoで元に戻せること
        let mut history = History::new(100);
        history.record(EditCommand::insert(0, "hello"));
        history.flush_pending_group();

        history.undo();
        assert!(history.can_redo());

        let redo_cmd = history.redo().unwrap();
        match redo_cmd {
            EditCommand::Insert { position, text } => {
                assert_eq!(position, 0);
                assert_eq!(text, "hello");
            }
            _ => panic!("Expected Insert command"),
        }
    }

    #[test]
    fn test_multiple_undo() {
        // 複数回のundoが正しく動作すること
        let mut history = History::new(100);

        // 個別に記録してフラッシュ（グループ化を防ぐ）
        history.record(EditCommand::insert(0, "a"));
        history.flush_pending_group();
        history.record(EditCommand::insert(1, "b"));
        history.flush_pending_group();
        history.record(EditCommand::insert(2, "c"));
        history.flush_pending_group();

        assert_eq!(history.undo_count(), 3);

        // 3回undo
        history.undo();
        assert_eq!(history.undo_count(), 2);
        assert_eq!(history.redo_count(), 1);

        history.undo();
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 2);

        history.undo();
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 3);
    }

    #[test]
    fn test_new_edit_clears_redo() {
        // 新しい編集でredoスタックがクリアされること
        let mut history = History::new(100);

        history.record(EditCommand::insert(0, "hello"));
        history.flush_pending_group();

        history.undo();
        assert!(history.can_redo());

        history.record(EditCommand::insert(0, "world"));
        assert!(!history.can_redo());
    }

    #[test]
    fn test_continuous_insert_grouping() {
        // 連続した挿入がグループ化されること
        let mut history = History::new(100);

        // 連続した文字入力（グループ化されるべき）
        history.record(EditCommand::insert(0, "h"));
        history.record(EditCommand::insert(1, "e"));
        history.record(EditCommand::insert(2, "l"));
        history.record(EditCommand::insert(3, "l"));
        history.record(EditCommand::insert(4, "o"));
        history.flush_pending_group();

        // 1つのグループとしてカウントされる
        assert_eq!(history.undo_count(), 1);

        // 1回のundoで全て戻る
        let inverse = history.undo().unwrap();
        match inverse {
            EditCommand::Group { commands } => {
                assert_eq!(commands.len(), 5);
            }
            _ => panic!("Expected Group command"),
        }
    }

    #[test]
    fn test_newline_breaks_group() {
        // 改行で新しいグループが開始されること
        let mut history = History::new(100);

        history.record(EditCommand::insert(0, "a"));
        history.record(EditCommand::insert(1, "b"));
        history.record(EditCommand::insert(2, "\n")); // 改行自体も別グループ
        history.record(EditCommand::insert(3, "c"));
        history.flush_pending_group();

        // ["ab"], ["\n"], ["c"] の3グループに分かれる
        assert_eq!(history.undo_count(), 3);
    }

    #[test]
    fn test_space_after_word_breaks_group() {
        // スペース後の非スペース文字で新しいグループが開始されること
        let mut history = History::new(100);

        history.record(EditCommand::insert(0, "h"));
        history.record(EditCommand::insert(1, "i"));
        history.record(EditCommand::insert(2, " "));
        history.flush_pending_group();
        history.record(EditCommand::insert(3, "t"));
        history.record(EditCommand::insert(4, "h"));
        history.record(EditCommand::insert(5, "e"));
        history.record(EditCommand::insert(6, "r"));
        history.record(EditCommand::insert(7, "e"));
        history.flush_pending_group();

        // "hi " と "there" で2グループ
        assert_eq!(history.undo_count(), 2);
    }

    #[test]
    fn test_delete_grouping() {
        // 連続した削除がグループ化されること（Backspace）
        let mut history = History::new(100);

        // Backspaceでの連続削除
        history.record(EditCommand::delete(4..5, "o"));
        history.record(EditCommand::delete(3..4, "l"));
        history.record(EditCommand::delete(2..3, "l"));
        history.record(EditCommand::delete(1..2, "e"));
        history.record(EditCommand::delete(0..1, "h"));
        history.flush_pending_group();

        assert_eq!(history.undo_count(), 1);
    }

    #[test]
    fn test_history_max_size() {
        // 履歴サイズが制限されること
        let mut history = History::new(5);

        for i in 0..10 {
            history.record(EditCommand::insert(i, format!("{}", i)));
            history.flush_pending_group();
        }

        assert_eq!(history.undo_count(), 5);
    }

    #[test]
    fn test_clear_history() {
        // 履歴のクリアが正しく動作すること
        let mut history = History::new(100);

        history.record(EditCommand::insert(0, "hello"));
        history.flush_pending_group();
        history.undo();

        assert!(history.can_redo());

        history.clear();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_mixed_operations() {
        // 挿入と削除の混在が正しく処理されること
        let mut history = History::new(100);

        // 挿入
        history.record(EditCommand::insert(0, "hello"));
        history.flush_pending_group();

        // 削除
        history.record(EditCommand::delete(0..2, "he"));
        history.flush_pending_group();

        assert_eq!(history.undo_count(), 2);

        // 削除をundo（"he"を復元）
        let inverse = history.undo().unwrap();
        match inverse {
            EditCommand::Insert { position, text } => {
                assert_eq!(position, 0);
                assert_eq!(text, "he");
            }
            _ => panic!("Expected Insert command"),
        }
    }
}
