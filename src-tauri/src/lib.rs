pub mod models;
pub mod services;
pub mod commands;
pub mod db;
pub mod utils;

use db::database::Database;
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub db: Database,
    pub watch_codes: Mutex<Vec<String>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");
            let database = Database::new(app_data_dir)
                .expect("Failed to initialize database");

            let settings = database.load_settings().unwrap_or_default();
            let active_strategy = settings.strategies.iter()
                .find(|s| s.id == settings.active_strategy_id)
                .cloned()
                .unwrap_or_default();
            let saved_codes = active_strategy.watch_codes;

            app.manage(AppState {
                db: database,
                watch_codes: Mutex::new(saved_codes),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::stock_cmd::get_realtime_data,
            commands::stock_cmd::get_kline_data,
            commands::stock_cmd::search_stocks,
            commands::stock_cmd::get_watchlist_enriched,
            commands::strategy_cmd::scan_market,
            commands::strategy_cmd::refresh_strategy,
            commands::strategy_cmd::generate_ai_instructions,
            commands::strategy_cmd::get_market_status,
            commands::strategy_cmd::is_trading_time,
            commands::strategy_cmd::update_watch_codes,
            commands::strategy_cmd::get_watch_codes,
            commands::strategy_cmd::get_market_stock_count,
            commands::ai_cmd::analyze_stock,
            commands::ai_cmd::get_analysis_history,
            commands::ai_cmd::get_today_token_usage,
            commands::settings_cmd::get_settings,
            commands::settings_cmd::save_settings,
            commands::settings_cmd::add_ai_config,
            commands::settings_cmd::remove_ai_config,
            commands::settings_cmd::update_ai_config,
            commands::settings_cmd::set_active_ai_config,
            commands::settings_cmd::update_strategy_config,
            commands::pool_cmd::fetch_limit_up_pool,
            commands::pool_cmd::fetch_streak_pool,
            commands::pool_cmd::fetch_and_apply_high_pool,
            commands::smart_stock_cmd::smart_search_stock,
            commands::smart_stock_cmd::get_hot_strategies,
            commands::smart_stock_cmd::ai_smart_pick,
            commands::watchlist_cmd::add_watchlist_stock,
            commands::watchlist_cmd::remove_watchlist_stock,
            commands::watchlist_cmd::get_watchlist_stocks,
            commands::watchlist_cmd::reorder_watchlist,
            commands::watchlist_cmd::get_stock_technical_analysis,
            commands::watchlist_cmd::ai_diagnose_stock,
            commands::backtest_cmd::run_backtest,
            commands::backtest_cmd::fetch_history_kline,
            commands::news_cmd::fetch_cls_telegraph,
            commands::news_cmd::fetch_eastmoney_news,
            commands::news_cmd::fetch_stock_news,
            commands::news_cmd::fetch_announcements,
            commands::news_cmd::fetch_reports,
            commands::news_cmd::fetch_sina_news,
            commands::ai_pick_cmd::ai_pick_stocks,
            commands::ai_pick_cmd::get_cached_picks,
            commands::ai_pick_cmd::find_similar_stocks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
