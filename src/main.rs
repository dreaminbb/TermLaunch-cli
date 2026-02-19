use std::io;

mod cli;

// TODO: TermLaunch Development Roadmap

// --- 1. ⚡ Foundation (最優先の基盤機能) ---
// - [ ] 設定ファイル管理 (config.toml): ホットキーや各機能の有効/無効などを管理
// - [ ] エラーロギング: デバッグ用にファイルにエラーを記録
// - [ ] UIの非同期処理化: ファイル検索などでUIが固まらないようにする

// --- 2. ✨ Core Features (中核機能) ---
// - [ ] 計算機能の強化: 単位・通貨換算に対応
// - [ ] ファイル検索 (+プレビュー, パスをコピーなどのアクション)
// - [ ] クリップボード履歴
// - [ ] システムコマンド (スリープ, 再起動, etc.)
// - [ ] スニペット管理・展開
// - [ ] URLサジェストと「ブラウザで開く」アクション

// --- 3. 🚀 Extensibility (拡張性) ---
// - [ ] 拡張機能 (Plugin) のためのAPI設計・実装
// - [ ] 拡張機能のパッケージ管理 (インストール/アップデート)

// --- 4. 🧠 Personalization (個人最適化) ---
// - [ ] ユーザーの選択履歴の保存
// - [ ] 履歴や使用頻度に基づいたサジェスト順位の最適化

// NOTE: 新機能は、原則として設定ファイルで有効/無効を切り替えられるように実装する。
fn main() -> io::Result<()> {
    let mut terminal = cli::terminal::setup_terminal()?;
    let mut app = cli::app::App::new();
    cli::runner::run_app(&mut terminal, &mut app)?;
    cli::terminal::restore_terminal(&mut terminal)?;
    Ok(())
}
