//! PTY管理
//!
//! 疑似端末の作成と管理を行う。

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

/// PTYのサイズ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    /// 列数
    pub cols: u16,
    /// 行数
    pub rows: u16,
    /// ピクセル幅（オプション）
    pub pixel_width: u16,
    /// ピクセル高さ（オプション）
    pub pixel_height: u16,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            pixel_width: 0,
            pixel_height: 0,
        }
    }
}

impl From<TerminalSize> for PtySize {
    fn from(size: TerminalSize) -> Self {
        PtySize {
            cols: size.cols,
            rows: size.rows,
            pixel_width: size.pixel_width,
            pixel_height: size.pixel_height,
        }
    }
}

/// PTYマネージャー
pub struct PtyManager {
    /// マスターPTY
    master: Option<Box<dyn MasterPty + Send>>,
    /// ライター（シェルへの入力）
    writer: Option<Box<dyn Write + Send>>,
    /// 出力バッファ
    output_buffer: Arc<Mutex<Vec<u8>>>,
    /// 現在のサイズ
    size: TerminalSize,
    /// 実行中かどうか
    running: Arc<Mutex<bool>>,
}

impl PtyManager {
    /// 新しいPTYマネージャーを作成する
    pub fn new() -> Self {
        Self {
            master: None,
            writer: None,
            output_buffer: Arc::new(Mutex::new(Vec::new())),
            size: TerminalSize::default(),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// サイズを設定してPTYマネージャーを作成する
    pub fn with_size(size: TerminalSize) -> Self {
        Self {
            master: None,
            writer: None,
            output_buffer: Arc::new(Mutex::new(Vec::new())),
            size,
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// シェルを起動する
    pub fn spawn_shell(&mut self) -> Result<(), PtyError> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(self.size.into())
            .map_err(|e| PtyError::OpenFailed(e.to_string()))?;

        // デフォルトシェルを取得
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        let mut cmd = CommandBuilder::new(&shell);
        cmd.env("TERM", "xterm-256color");

        // スレーブにシェルを起動
        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        // マスターからリーダーを取得
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyError::CloneFailed(e.to_string()))?;

        // マスターからライターを取得
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyError::CloneFailed(e.to_string()))?;

        self.writer = Some(writer);
        self.master = Some(pair.master);

        // 出力読み取りスレッドを開始
        let output_buffer = Arc::clone(&self.output_buffer);
        let running = Arc::clone(&self.running);
        *running.lock().unwrap() = true;

        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                {
                    let running_guard = running.lock().unwrap();
                    if !*running_guard {
                        break;
                    }
                }

                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let mut output = output_buffer.lock().unwrap();
                        output.extend_from_slice(&buf[..n]);
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::WouldBlock {
                            eprintln!("PTY読み取りエラー: {}", e);
                            break;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// シェルにテキストを送信する
    pub fn write(&mut self, text: &str) -> Result<(), PtyError> {
        if let Some(ref mut writer) = self.writer {
            writer
                .write_all(text.as_bytes())
                .map_err(|e| PtyError::WriteFailed(e.to_string()))?;
            writer
                .flush()
                .map_err(|e| PtyError::WriteFailed(e.to_string()))?;
            Ok(())
        } else {
            Err(PtyError::NotRunning)
        }
    }

    /// シェルにバイト列を送信する
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), PtyError> {
        if let Some(ref mut writer) = self.writer {
            writer
                .write_all(bytes)
                .map_err(|e| PtyError::WriteFailed(e.to_string()))?;
            writer
                .flush()
                .map_err(|e| PtyError::WriteFailed(e.to_string()))?;
            Ok(())
        } else {
            Err(PtyError::NotRunning)
        }
    }

    /// 出力を読み取る
    pub fn read_output(&self) -> Vec<u8> {
        let mut output = self.output_buffer.lock().unwrap();
        let data = output.clone();
        output.clear();
        data
    }

    /// 出力をクリアせずに取得する
    pub fn peek_output(&self) -> Vec<u8> {
        let output = self.output_buffer.lock().unwrap();
        output.clone()
    }

    /// サイズを変更する
    pub fn resize(&mut self, size: TerminalSize) -> Result<(), PtyError> {
        self.size = size;
        if let Some(ref master) = self.master {
            master
                .resize(size.into())
                .map_err(|e| PtyError::ResizeFailed(e.to_string()))?;
        }
        Ok(())
    }

    /// 現在のサイズを取得する
    pub fn size(&self) -> TerminalSize {
        self.size
    }

    /// 実行中かどうか
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// シェルを終了する
    pub fn kill(&mut self) {
        *self.running.lock().unwrap() = false;
        self.writer = None;
        self.master = None;
    }
}

impl Drop for PtyManager {
    fn drop(&mut self) {
        self.kill();
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// PTYエラー
#[derive(Debug)]
pub enum PtyError {
    /// PTYを開けなかった
    OpenFailed(String),
    /// シェル起動に失敗した
    SpawnFailed(String),
    /// クローンに失敗した
    CloneFailed(String),
    /// 書き込みに失敗した
    WriteFailed(String),
    /// リサイズに失敗した
    ResizeFailed(String),
    /// 実行されていない
    NotRunning,
}

impl std::fmt::Display for PtyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PtyError::OpenFailed(s) => write!(f, "PTYを開けませんでした: {}", s),
            PtyError::SpawnFailed(s) => write!(f, "シェルの起動に失敗しました: {}", s),
            PtyError::CloneFailed(s) => write!(f, "PTYのクローンに失敗しました: {}", s),
            PtyError::WriteFailed(s) => write!(f, "書き込みに失敗しました: {}", s),
            PtyError::ResizeFailed(s) => write!(f, "リサイズに失敗しました: {}", s),
            PtyError::NotRunning => write!(f, "シェルが実行されていません"),
        }
    }
}

impl std::error::Error for PtyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_size_default() {
        let size = TerminalSize::default();
        assert_eq!(size.cols, 80);
        assert_eq!(size.rows, 24);
    }

    #[test]
    fn test_terminal_size_conversion() {
        let size = TerminalSize {
            cols: 120,
            rows: 40,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pty_size: PtySize = size.into();
        assert_eq!(pty_size.cols, 120);
        assert_eq!(pty_size.rows, 40);
    }

    #[test]
    fn test_pty_manager_creation() {
        let manager = PtyManager::new();
        assert!(!manager.is_running());
    }

    #[test]
    fn test_pty_manager_with_size() {
        let size = TerminalSize {
            cols: 100,
            rows: 30,
            ..Default::default()
        };
        let manager = PtyManager::with_size(size);
        assert_eq!(manager.size().cols, 100);
        assert_eq!(manager.size().rows, 30);
    }

    #[test]
    fn test_pty_error_display() {
        let err = PtyError::NotRunning;
        assert_eq!(err.to_string(), "シェルが実行されていません");

        let err = PtyError::SpawnFailed("テストエラー".to_string());
        assert!(err.to_string().contains("シェルの起動に失敗しました"));
    }

    #[test]
    fn test_write_without_shell() {
        let mut manager = PtyManager::new();
        let result = manager.write("test");
        assert!(matches!(result, Err(PtyError::NotRunning)));
    }

    // 注: spawn_shell()のテストは実際のPTYを作成するため、
    // CI環境では失敗する可能性がある。手動テストで確認。
}
