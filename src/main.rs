use std::io;

mod config;
use crate::config::CONFIG;

mod app_logger;
mod cli; // Changed from `mod log;`
use log; // The external log crate

// TODO: TermLaunch Development Roadmap

// --- 1. ⚡ Foundation (最優先の基盤機能) ---
// - [ ] UIの非同期処理化: ファイル検索などでUIが固まらないようにする // NOTE:　一旦置いとく、core Features やったらやる
// --- 2. ✨ Core Features (中核機能) ---
// - [ ] ファイル検索 (+プレビュー, パスをコピーなどのアクション)
// - [ ] URLサジェストと「ブラウザで開く」アクション
// - [ ] クリップボード履歴
// - [ ] システムコマンド (スリープ, 再起動, etc.)
// - [ ] スニペット管理・展開
// - [ ] 計算機能の強化: 単位・通貨換算に対応

// --- 3. 🚀 Extensibility (拡張性) ---
// - [ ] 拡張機能 (Plugin) のためのAPI設計・実装
// - [ ] 拡張機能のパッケージ管理 (インストール/アップデート)

// --- 4. 🧠 Personalization (個人最適化) ---
// - [ ] ユーザーの選択履歴の保存
// - [ ] 履歴や使用頻度に基づいたサジェスト順位の最適化

// NOTE: 新機能は、原則として設定ファイルで有効/無効を切り替えられるように実装する。
fn main() -> io::Result<()> {
    app_logger::init_logger("cli").expect("Failed to initialize logger"); // Changed to app_logger

    // Print loaded config in debug builds
    #[cfg(debug_assertions)]
    {
        log::info!("[DEBUG] Loaded CLI Config: {:#?}", *CONFIG); // Changed to log::info!
    }

    let mut terminal = cli::terminal::setup_terminal()?;
    let mut app = cli::app::App::new();
    cli::runner::run_app(&mut terminal, &mut app)?;
    cli::terminal::restore_terminal(&mut terminal)?;
    Ok(())
}
