//! ターミナルエミュレータモジュール
//!
//! 内蔵ターミナルを提供する。
//! 入力はEnterで改行、Ctrl+Enterでシェルに送信する仕様。

pub mod pty;
pub mod view;

// TODO: Phase 5で実装
