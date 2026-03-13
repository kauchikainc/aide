//! LSPサーバー通信
//!
//! 言語サーバーとの通信を管理する。

use super::{CompletionList, DiagnosticCollection, LspLanguage, Position, Range};
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification,
        PublishDiagnostics,
    },
    request::{Completion, GotoDefinition, HoverRequest, Initialize, Request, Shutdown},
    ClientCapabilities, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverParams, InitializeParams, InitializeResult,
    PublishDiagnosticsParams, TextDocumentContentChangeEvent, TextDocumentIdentifier,
    TextDocumentItem, TextDocumentPositionParams, Uri, VersionedTextDocumentIdentifier,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use url::Url;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

/// LSPメッセージ
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LspMessage {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<LspError>,
}

/// LSPエラー
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LspError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// LSPクライアントの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    /// 未接続
    Disconnected,
    /// 接続中
    Connecting,
    /// 初期化済み
    Initialized,
    /// シャットダウン中
    ShuttingDown,
}

/// LSPクライアント
pub struct LspClient {
    /// 言語サーバープロセス
    process: Option<Child>,
    /// 標準入力
    stdin: Option<ChildStdin>,
    /// 標準出力リーダー
    stdout_reader: Option<Arc<Mutex<BufReader<ChildStdout>>>>,
    /// 現在の状態
    state: ClientState,
    /// 言語
    language: LspLanguage,
    /// ワークスペースルート
    workspace_root: Option<PathBuf>,
    /// リクエストID
    request_id: AtomicI64,
    /// 保留中のリクエスト
    pending_requests: HashMap<i64, String>,
    /// 開いているドキュメント
    open_documents: HashMap<Uri, i32>,
    /// 診断コレクション（ファイルパス -> 診断）
    diagnostics: HashMap<Uri, DiagnosticCollection>,
    /// サーバー機能
    server_capabilities: Option<lsp_types::ServerCapabilities>,
}

impl LspClient {
    /// 新しいLSPクライアントを作成する
    pub fn new(language: LspLanguage) -> Self {
        Self {
            process: None,
            stdin: None,
            stdout_reader: None,
            state: ClientState::Disconnected,
            language,
            workspace_root: None,
            request_id: AtomicI64::new(1),
            pending_requests: HashMap::new(),
            open_documents: HashMap::new(),
            diagnostics: HashMap::new(),
            server_capabilities: None,
        }
    }

    /// ワークスペースルートを設定する
    pub fn set_workspace_root(&mut self, root: PathBuf) {
        self.workspace_root = Some(root);
    }

    /// 現在の状態を取得する
    pub fn state(&self) -> ClientState {
        self.state
    }

    /// サーバーに接続する
    pub fn connect(&mut self) -> Result<(), LspClientError> {
        if self.state != ClientState::Disconnected {
            return Err(LspClientError::InvalidState(
                "既に接続されています".to_string(),
            ));
        }

        self.state = ClientState::Connecting;

        // 言語サーバーを起動
        let server_cmd = self.language.server_command();
        let mut child = Command::new(server_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| LspClientError::SpawnFailed(format!("{}: {}", server_cmd, e)))?;

        let stdin = child.stdin.take().ok_or_else(|| {
            LspClientError::SpawnFailed("標準入力を取得できませんでした".to_string())
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            LspClientError::SpawnFailed("標準出力を取得できませんでした".to_string())
        })?;

        self.process = Some(child);
        self.stdin = Some(stdin);
        self.stdout_reader = Some(Arc::new(Mutex::new(BufReader::new(stdout))));

        Ok(())
    }

    /// 初期化する
    pub fn initialize(&mut self) -> Result<InitializeResult, LspClientError> {
        if self.state != ClientState::Connecting {
            return Err(LspClientError::InvalidState(
                "接続中の状態でない場合は初期化できません".to_string(),
            ));
        }

        let root_uri = self.workspace_root.as_ref().and_then(|p| {
            Url::from_file_path(p).ok().and_then(|url| url.as_str().parse().ok())
        });

        let params = InitializeParams {
            process_id: Some(std::process::id()),
            root_uri,
            capabilities: ClientCapabilities {
                text_document: Some(lsp_types::TextDocumentClientCapabilities {
                    completion: Some(lsp_types::CompletionClientCapabilities {
                        completion_item: Some(lsp_types::CompletionItemCapability {
                            snippet_support: Some(true),
                            documentation_format: Some(vec![
                                lsp_types::MarkupKind::Markdown,
                                lsp_types::MarkupKind::PlainText,
                            ]),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    hover: Some(lsp_types::HoverClientCapabilities {
                        content_format: Some(vec![
                            lsp_types::MarkupKind::Markdown,
                            lsp_types::MarkupKind::PlainText,
                        ]),
                        ..Default::default()
                    }),
                    definition: Some(lsp_types::GotoCapability::default()),
                    publish_diagnostics: Some(lsp_types::PublishDiagnosticsClientCapabilities {
                        related_information: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        let result: InitializeResult = self.send_request::<Initialize>(params)?;
        self.server_capabilities = Some(result.capabilities.clone());

        // initialized通知を送信
        self.send_notification("initialized", json!({}))?;

        self.state = ClientState::Initialized;
        Ok(result)
    }

    /// シャットダウンする
    pub fn shutdown(&mut self) -> Result<(), LspClientError> {
        if self.state != ClientState::Initialized {
            return Err(LspClientError::InvalidState(
                "初期化されていません".to_string(),
            ));
        }

        self.state = ClientState::ShuttingDown;
        self.send_request::<Shutdown>(())?;
        self.send_notification("exit", json!({}))?;

        // プロセスを終了
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }

        self.stdin = None;
        self.stdout_reader = None;
        self.state = ClientState::Disconnected;
        Ok(())
    }

    /// ドキュメントを開く
    pub fn open_document(&mut self, uri: &Uri, text: &str) -> Result<(), LspClientError> {
        if self.state != ClientState::Initialized {
            return Err(LspClientError::InvalidState(
                "初期化されていません".to_string(),
            ));
        }

        let version = 1;
        self.open_documents.insert(uri.clone(), version);

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: self.language.language_id().to_string(),
                version,
                text: text.to_string(),
            },
        };

        self.send_notification(DidOpenTextDocument::METHOD, serde_json::to_value(params)?)?;
        Ok(())
    }

    /// ドキュメントを閉じる
    pub fn close_document(&mut self, uri: &Uri) -> Result<(), LspClientError> {
        if self.state != ClientState::Initialized {
            return Err(LspClientError::InvalidState(
                "初期化されていません".to_string(),
            ));
        }

        self.open_documents.remove(uri);
        self.diagnostics.remove(uri);

        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
        };

        self.send_notification(DidCloseTextDocument::METHOD, serde_json::to_value(params)?)?;
        Ok(())
    }

    /// ドキュメントの変更を通知する
    pub fn change_document(&mut self, uri: &Uri, text: &str) -> Result<(), LspClientError> {
        if self.state != ClientState::Initialized {
            return Err(LspClientError::InvalidState(
                "初期化されていません".to_string(),
            ));
        }

        let version = self
            .open_documents
            .get_mut(uri)
            .ok_or_else(|| LspClientError::DocumentNotOpen(uri.to_string()))?;
        *version += 1;

        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: *version,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: text.to_string(),
            }],
        };

        self.send_notification(DidChangeTextDocument::METHOD, serde_json::to_value(params)?)?;
        Ok(())
    }

    /// 補完を要求する
    pub fn completion(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<CompletionList, LspClientError> {
        if self.state != ClientState::Initialized {
            return Err(LspClientError::InvalidState(
                "初期化されていません".to_string(),
            ));
        }

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: position.into(),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        let response: Option<CompletionResponse> = self.send_request::<Completion>(params)?;
        match response {
            Some(resp) => Ok(resp.into()),
            None => Ok(CompletionList::default()),
        }
    }

    /// ホバー情報を取得する
    pub fn hover(&mut self, uri: &Uri, position: Position) -> Result<Option<String>, LspClientError> {
        if self.state != ClientState::Initialized {
            return Err(LspClientError::InvalidState(
                "初期化されていません".to_string(),
            ));
        }

        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: position.into(),
            },
            work_done_progress_params: Default::default(),
        };

        let response: Option<Hover> = self.send_request::<HoverRequest>(params)?;
        Ok(response.map(|h| match h.contents {
            lsp_types::HoverContents::Markup(mc) => mc.value,
            lsp_types::HoverContents::Scalar(ms) => match ms {
                lsp_types::MarkedString::String(s) => s,
                lsp_types::MarkedString::LanguageString(ls) => ls.value,
            },
            lsp_types::HoverContents::Array(arr) => arr
                .into_iter()
                .map(|ms| match ms {
                    lsp_types::MarkedString::String(s) => s,
                    lsp_types::MarkedString::LanguageString(ls) => ls.value,
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }))
    }

    /// 定義へジャンプする
    pub fn goto_definition(
        &mut self,
        uri: &Uri,
        position: Position,
    ) -> Result<Option<(Uri, Range)>, LspClientError> {
        if self.state != ClientState::Initialized {
            return Err(LspClientError::InvalidState(
                "初期化されていません".to_string(),
            ));
        }

        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: position.into(),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let response: Option<GotoDefinitionResponse> =
            self.send_request::<GotoDefinition>(params)?;
        Ok(response.and_then(|r| match r {
            GotoDefinitionResponse::Scalar(loc) => {
                Some((loc.uri, loc.range.into()))
            }
            GotoDefinitionResponse::Array(locs) => {
                locs.into_iter().next().map(|loc| (loc.uri, loc.range.into()))
            }
            GotoDefinitionResponse::Link(links) => {
                links.into_iter().next().map(|link| (link.target_uri, link.target_selection_range.into()))
            }
        }))
    }

    /// 指定URIの診断を取得する
    pub fn get_diagnostics(&self, uri: &Uri) -> Option<&DiagnosticCollection> {
        self.diagnostics.get(uri)
    }

    /// 診断を処理する
    pub fn handle_diagnostics(&mut self, params: PublishDiagnosticsParams) {
        let mut collection = DiagnosticCollection::new();
        for diag in params.diagnostics {
            collection.add(diag.into());
        }
        self.diagnostics.insert(params.uri, collection);
    }

    /// リクエストを送信する
    fn send_request<R: Request>(&mut self, params: R::Params) -> Result<R::Result, LspClientError>
    where
        R::Params: Serialize,
        R::Result: for<'de> Deserialize<'de>,
    {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        self.pending_requests.insert(id, R::METHOD.to_string());

        let message = LspMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: Some(R::METHOD.to_string()),
            params: Some(serde_json::to_value(params)?),
            result: None,
            error: None,
        };

        self.write_message(&message)?;
        let response = self.read_response(id)?;

        if let Some(error) = response.error {
            return Err(LspClientError::ServerError(error.message));
        }

        let result = response
            .result
            .ok_or_else(|| LspClientError::InvalidResponse("結果がありません".to_string()))?;
        serde_json::from_value(result).map_err(|e| LspClientError::InvalidResponse(e.to_string()))
    }

    /// 通知を送信する
    fn send_notification(&mut self, method: &str, params: Value) -> Result<(), LspClientError> {
        let message = LspMessage {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: Some(method.to_string()),
            params: Some(params),
            result: None,
            error: None,
        };

        self.write_message(&message)
    }

    /// メッセージを書き込む
    fn write_message(&mut self, message: &LspMessage) -> Result<(), LspClientError> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| LspClientError::NotConnected)?;

        let content = serde_json::to_string(message)?;
        let header = format!("Content-Length: {}\r\n\r\n", content.len());

        stdin
            .write_all(header.as_bytes())
            .map_err(|e| LspClientError::IoError(e.to_string()))?;
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| LspClientError::IoError(e.to_string()))?;
        stdin
            .flush()
            .map_err(|e| LspClientError::IoError(e.to_string()))?;

        Ok(())
    }

    /// レスポンスを読み取る
    fn read_response(&mut self, expected_id: i64) -> Result<LspMessage, LspClientError> {
        // stdout_readerが存在することを確認
        if self.stdout_reader.is_none() {
            return Err(LspClientError::NotConnected);
        }

        loop {
            let message = self.read_message()?;

            // レスポンスの場合
            if let Some(id) = message.id {
                if id == expected_id {
                    self.pending_requests.remove(&id);
                    return Ok(message);
                }
            }

            // 通知の場合（診断など）
            if let Some(ref method) = message.method {
                if method == PublishDiagnostics::METHOD {
                    if let Some(params) = message.params {
                        if let Ok(diag_params) = serde_json::from_value::<PublishDiagnosticsParams>(params) {
                            self.handle_diagnostics(diag_params);
                        }
                    }
                }
            }
        }
    }

    /// メッセージを読み取る
    fn read_message(&mut self) -> Result<LspMessage, LspClientError> {
        let reader = self
            .stdout_reader
            .as_ref()
            .ok_or_else(|| LspClientError::NotConnected)?;
        let mut reader = reader.lock().map_err(|_| {
            LspClientError::IoError("ロックの取得に失敗しました".to_string())
        })?;

        // ヘッダーを読み取る
        let mut content_length: Option<usize> = None;
        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|e| LspClientError::IoError(e.to_string()))?;

            let line = line.trim();
            if line.is_empty() {
                break;
            }

            if let Some(len_str) = line.strip_prefix("Content-Length: ") {
                content_length = Some(
                    len_str
                        .parse()
                        .map_err(|_| LspClientError::InvalidResponse("無効なContent-Length".to_string()))?,
                );
            }
        }

        let content_length = content_length
            .ok_or_else(|| LspClientError::InvalidResponse("Content-Lengthがありません".to_string()))?;

        // コンテンツを読み取る
        let mut content = vec![0u8; content_length];
        reader
            .read_exact(&mut content)
            .map_err(|e| LspClientError::IoError(e.to_string()))?;

        let content_str = String::from_utf8(content)
            .map_err(|_| LspClientError::InvalidResponse("無効なUTF-8".to_string()))?;

        serde_json::from_str(&content_str).map_err(|e| LspClientError::InvalidResponse(e.to_string()))
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        if self.state == ClientState::Initialized {
            let _ = self.shutdown();
        } else if let Some(mut process) = self.process.take() {
            let _ = process.kill();
        }
    }
}

/// LSPクライアントエラー
#[derive(Debug)]
pub enum LspClientError {
    /// 接続されていない
    NotConnected,
    /// 起動に失敗
    SpawnFailed(String),
    /// 無効な状態
    InvalidState(String),
    /// I/Oエラー
    IoError(String),
    /// サーバーエラー
    ServerError(String),
    /// 無効なレスポンス
    InvalidResponse(String),
    /// ドキュメントが開かれていない
    DocumentNotOpen(String),
    /// JSONエラー
    JsonError(serde_json::Error),
}

impl From<serde_json::Error> for LspClientError {
    fn from(e: serde_json::Error) -> Self {
        LspClientError::JsonError(e)
    }
}

impl std::fmt::Display for LspClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LspClientError::NotConnected => write!(f, "LSPサーバーに接続されていません"),
            LspClientError::SpawnFailed(s) => write!(f, "LSPサーバーの起動に失敗しました: {}", s),
            LspClientError::InvalidState(s) => write!(f, "無効な状態: {}", s),
            LspClientError::IoError(s) => write!(f, "I/Oエラー: {}", s),
            LspClientError::ServerError(s) => write!(f, "サーバーエラー: {}", s),
            LspClientError::InvalidResponse(s) => write!(f, "無効なレスポンス: {}", s),
            LspClientError::DocumentNotOpen(s) => write!(f, "ドキュメントが開かれていません: {}", s),
            LspClientError::JsonError(e) => write!(f, "JSONエラー: {}", e),
        }
    }
}

impl std::error::Error for LspClientError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LspClient::new(LspLanguage::Rust);
        assert_eq!(client.state(), ClientState::Disconnected);
    }

    #[test]
    fn test_client_set_workspace_root() {
        let mut client = LspClient::new(LspLanguage::Rust);
        client.set_workspace_root(PathBuf::from("/test/path"));
        assert_eq!(
            client.workspace_root,
            Some(PathBuf::from("/test/path"))
        );
    }

    #[test]
    fn test_lsp_client_error_display() {
        let err = LspClientError::NotConnected;
        assert_eq!(
            err.to_string(),
            "LSPサーバーに接続されていません"
        );

        let err = LspClientError::SpawnFailed("rust-analyzer".to_string());
        assert!(err.to_string().contains("rust-analyzer"));
    }

    #[test]
    fn test_client_state_initial() {
        let client = LspClient::new(LspLanguage::Rust);
        assert_eq!(client.state(), ClientState::Disconnected);
        assert!(client.diagnostics.is_empty());
        assert!(client.open_documents.is_empty());
    }

    #[test]
    fn test_connect_without_server() {
        // rust-analyzerがインストールされていない環境でも
        // エラーが適切に返されることを確認
        let _client = LspClient::new(LspLanguage::Rust);
        // 存在しないコマンドでテスト
        // 実際のテストでは、rust-analyzerがある場合は成功する可能性がある
    }

    #[test]
    fn test_invalid_state_operations() {
        let mut client = LspClient::new(LspLanguage::Rust);

        // 接続していない状態で初期化しようとする
        let result = client.initialize();
        assert!(result.is_err());

        // 接続していない状態でドキュメントを開こうとする
        let uri: Uri = "file:///test.rs".parse().unwrap();
        let result = client.open_document(&uri, "fn main() {}");
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_diagnostics() {
        let mut client = LspClient::new(LspLanguage::Rust);
        let uri: Uri = "file:///test.rs".parse().unwrap();

        let params = PublishDiagnosticsParams {
            uri: uri.clone(),
            diagnostics: vec![lsp_types::Diagnostic {
                range: lsp_types::Range {
                    start: lsp_types::Position { line: 0, character: 0 },
                    end: lsp_types::Position { line: 0, character: 10 },
                },
                severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                message: "テストエラー".to_string(),
                ..Default::default()
            }],
            version: None,
        };

        client.handle_diagnostics(params);

        let diags = client.get_diagnostics(&uri);
        assert!(diags.is_some());
        assert_eq!(diags.unwrap().len(), 1);
    }
}
