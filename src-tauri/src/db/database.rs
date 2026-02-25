use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::models::ai::AIAnalysisResult;
use crate::models::settings::AppSettings;
use crate::models::stock::StockDailyHistory;
use crate::models::watchlist::WatchlistStock;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("stock_helper.db");
        let conn = Connection::open(db_path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS settings (
                id TEXT PRIMARY KEY DEFAULT 'default',
                data TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS ai_analysis (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL,
                name TEXT NOT NULL,
                model_name TEXT NOT NULL,
                question TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_ai_analysis_code ON ai_analysis(code);
            CREATE INDEX IF NOT EXISTS idx_ai_analysis_date ON ai_analysis(created_at);

            CREATE TABLE IF NOT EXISTS stock_daily_history (
                code TEXT NOT NULL,
                date TEXT NOT NULL,
                close REAL NOT NULL,
                high REAL NOT NULL,
                low REAL NOT NULL,
                open_price REAL NOT NULL,
                volume REAL NOT NULL,
                amount REAL NOT NULL,
                change_pct REAL NOT NULL,
                is_limit_up INTEGER NOT NULL DEFAULT 0,
                turnover_rate REAL NOT NULL DEFAULT 0,
                PRIMARY KEY (code, date)
            );

            CREATE INDEX IF NOT EXISTS idx_daily_code ON stock_daily_history(code);

            CREATE TABLE IF NOT EXISTS token_usage (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL,
                model_name TEXT NOT NULL,
                prompt_tokens INTEGER NOT NULL,
                completion_tokens INTEGER NOT NULL,
                total_tokens INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_token_date ON token_usage(date);

            CREATE TABLE IF NOT EXISTS watchlist_stocks (
                code TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                group_name TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS ai_pick_cache (
                date TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            ",
        )?;
        Ok(())
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let data = serde_json::to_string(settings)?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (id, data, updated_at) VALUES ('default', ?1, datetime('now'))",
            rusqlite::params![data],
        )?;
        Ok(())
    }

    pub fn load_settings(&self) -> Result<AppSettings> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT data FROM settings WHERE id = 'default'",
            [],
            |row| {
                let data: String = row.get(0)?;
                Ok(data)
            },
        );
        match result {
            Ok(data) => Ok(serde_json::from_str(&data)?),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                let default = AppSettings::default();
                drop(conn);
                self.save_settings(&default)?;
                Ok(default)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn save_ai_analysis(&self, result: &AIAnalysisResult) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO ai_analysis (id, code, name, model_name, question, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![result.id, result.code, result.name, result.model_name, result.question, result.content, result.created_at],
        )?;
        Ok(())
    }

    pub fn get_ai_analysis_history(&self, code: &str, limit: usize) -> Result<Vec<AIAnalysisResult>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, code, name, model_name, question, content, created_at FROM ai_analysis WHERE code = ?1 ORDER BY created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![code, limit], |row| {
            Ok(AIAnalysisResult {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                model_name: row.get(3)?,
                question: row.get(4)?,
                content: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_today_ai_analysis(&self, code: &str) -> Result<Option<AIAnalysisResult>> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let result = conn.query_row(
            "SELECT id, code, name, model_name, question, content, created_at FROM ai_analysis WHERE code = ?1 AND created_at >= ?2 ORDER BY created_at DESC LIMIT 1",
            rusqlite::params![code, today],
            |row| {
                Ok(AIAnalysisResult {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name: row.get(2)?,
                    model_name: row.get(3)?,
                    question: row.get(4)?,
                    content: row.get(5)?,
                    created_at: row.get(6)?,
                })
            },
        );
        match result {
            Ok(r) => Ok(Some(r)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn save_daily_history(&self, records: &[StockDailyHistory]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        for r in records {
            tx.execute(
                "INSERT OR REPLACE INTO stock_daily_history (code, date, close, high, low, open_price, volume, amount, change_pct, is_limit_up, turnover_rate) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![r.code, r.date, r.close, r.high, r.low, r.open, r.volume, r.amount, r.change_pct, r.is_limit_up as i32, r.turnover_rate],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_daily_history(&self, code: &str, days: usize) -> Result<Vec<StockDailyHistory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT code, date, close, high, low, open_price, volume, amount, change_pct, is_limit_up, turnover_rate FROM stock_daily_history WHERE code = ?1 ORDER BY date DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![code, days], |row| {
            Ok(StockDailyHistory {
                code: row.get(0)?,
                date: row.get(1)?,
                close: row.get(2)?,
                high: row.get(3)?,
                low: row.get(4)?,
                open: row.get(5)?,
                volume: row.get(6)?,
                amount: row.get(7)?,
                change_pct: row.get(8)?,
                is_limit_up: row.get::<_, i32>(9)? != 0,
                turnover_rate: row.get(10)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn record_token_usage(&self, model_name: &str, prompt_tokens: u32, completion_tokens: u32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        conn.execute(
            "INSERT INTO token_usage (date, model_name, prompt_tokens, completion_tokens, total_tokens, created_at) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
            rusqlite::params![today, model_name, prompt_tokens, completion_tokens, prompt_tokens + completion_tokens],
        )?;
        Ok(())
    }

    pub fn get_today_token_usage(&self) -> Result<u32> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let result = conn.query_row(
            "SELECT COALESCE(SUM(total_tokens), 0) FROM token_usage WHERE date = ?1",
            rusqlite::params![today],
            |row| row.get::<_, u32>(0),
        );
        match result {
            Ok(total) => Ok(total),
            Err(_) => Ok(0),
        }
    }

    // ====== Watchlist Methods ======

    pub fn add_watchlist_stock(&self, stock: &WatchlistStock) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO watchlist_stocks (code, name, sort_order, group_name, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![stock.code, stock.name, stock.sort_order, stock.group_name, stock.created_at],
        )?;
        Ok(())
    }

    pub fn remove_watchlist_stock(&self, code: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM watchlist_stocks WHERE code = ?1", rusqlite::params![code])?;
        Ok(())
    }

    pub fn get_watchlist_stocks(&self) -> Result<Vec<WatchlistStock>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT code, name, sort_order, group_name, created_at FROM watchlist_stocks ORDER BY sort_order ASC, created_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(WatchlistStock {
                code: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
                group_name: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn reorder_watchlist(&self, codes: &[String]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        for (i, code) in codes.iter().enumerate() {
            tx.execute(
                "UPDATE watchlist_stocks SET sort_order = ?1 WHERE code = ?2",
                rusqlite::params![i as i32, code],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    // ====== History Kline Extended Methods ======

    pub fn get_latest_history_date(&self, code: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT MAX(date) FROM stock_daily_history WHERE code = ?1",
            rusqlite::params![code],
            |row| row.get::<_, Option<String>>(0),
        );
        match result {
            Ok(date) => Ok(date),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_daily_history_range(&self, code: &str, start: &str, end: &str) -> Result<Vec<StockDailyHistory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT code, date, close, high, low, open_price, volume, amount, change_pct, is_limit_up, turnover_rate FROM stock_daily_history WHERE code = ?1 AND date >= ?2 AND date <= ?3 ORDER BY date ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![code, start, end], |row| {
            Ok(StockDailyHistory {
                code: row.get(0)?,
                date: row.get(1)?,
                close: row.get(2)?,
                high: row.get(3)?,
                low: row.get(4)?,
                open: row.get(5)?,
                volume: row.get(6)?,
                amount: row.get(7)?,
                change_pct: row.get(8)?,
                is_limit_up: row.get::<_, i32>(9)? != 0,
                turnover_rate: row.get(10)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_daily_history_asc(&self, code: &str, days: usize) -> Result<Vec<StockDailyHistory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT code, date, close, high, low, open_price, volume, amount, change_pct, is_limit_up, turnover_rate FROM stock_daily_history WHERE code = ?1 ORDER BY date DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![code, days], |row| {
            Ok(StockDailyHistory {
                code: row.get(0)?,
                date: row.get(1)?,
                close: row.get(2)?,
                high: row.get(3)?,
                low: row.get(4)?,
                open: row.get(5)?,
                volume: row.get(6)?,
                amount: row.get(7)?,
                change_pct: row.get(8)?,
                is_limit_up: row.get::<_, i32>(9)? != 0,
                turnover_rate: row.get(10)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        results.reverse();
        Ok(results)
    }

    // ====== AI Pick Cache ======

    pub fn save_ai_pick_cache(&self, content: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        conn.execute(
            "INSERT OR REPLACE INTO ai_pick_cache (date, content, created_at) VALUES (?1, ?2, datetime('now'))",
            rusqlite::params![today, content],
        )?;
        Ok(())
    }

    pub fn get_ai_pick_cache(&self) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let result = conn.query_row(
            "SELECT content FROM ai_pick_cache WHERE date = ?1",
            rusqlite::params![today],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(content) => Ok(Some(content)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
