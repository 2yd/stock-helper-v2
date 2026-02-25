use chrono::{Local, Timelike, Weekday, Datelike};

pub struct TradingScheduler;

impl TradingScheduler {
    /// Check if current time is during A-share trading hours
    pub fn is_trading_time() -> bool {
        let now = Local::now();
        let weekday = now.weekday();

        // Weekend: no trading
        if weekday == Weekday::Sat || weekday == Weekday::Sun {
            return false;
        }

        let hour = now.hour();
        let minute = now.minute();
        let time_val = hour * 100 + minute;

        // Pre-market bid: 9:15 - 9:25
        // Morning session: 9:30 - 11:30
        // Afternoon session: 13:00 - 15:00
        (time_val >= 915 && time_val <= 925)
            || (time_val >= 930 && time_val <= 1130)
            || (time_val >= 1300 && time_val <= 1500)
    }

    /// Check if currently in bid phase (9:15-9:25)
    pub fn is_bid_phase() -> bool {
        let now = Local::now();
        let weekday = now.weekday();
        if weekday == Weekday::Sat || weekday == Weekday::Sun {
            return false;
        }
        let hour = now.hour();
        let minute = now.minute();
        let time_val = hour * 100 + minute;
        time_val >= 915 && time_val <= 925
    }

    /// Get market status description
    pub fn market_status() -> String {
        if !Self::is_weekday() {
            return "休市(周末)".to_string();
        }
        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute();
        let time_val = hour * 100 + minute;

        if time_val < 915 {
            "盘前".to_string()
        } else if time_val <= 925 {
            "竞价中".to_string()
        } else if time_val < 930 {
            "集合竞价结束".to_string()
        } else if time_val <= 1130 {
            "交易中(上午)".to_string()
        } else if time_val < 1300 {
            "午间休市".to_string()
        } else if time_val <= 1500 {
            "交易中(下午)".to_string()
        } else {
            "已收盘".to_string()
        }
    }

    fn is_weekday() -> bool {
        let now = Local::now();
        let weekday = now.weekday();
        weekday != Weekday::Sat && weekday != Weekday::Sun
    }
}
