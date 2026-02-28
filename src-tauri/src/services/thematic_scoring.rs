use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::models::news::NewsItem;
use crate::services::news_service::{fetch_cls_telegraph, fetch_eastmoney_news, fetch_sina_roll_news, fetch_sina_7x24, fetch_wallstreetcn_lives};
use crate::utils::http::build_stock_client;

#[derive(Debug, Clone, Default)]
pub struct ThematicScoringResult {
    pub stock_sentiment: HashMap<String, f64>,
    pub stock_heat: HashMap<String, f64>,
    pub stock_themes: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
struct ThemeRule {
    name: &'static str,
    keywords: &'static [&'static str],
    concept_keywords: &'static [&'static str],
}

#[derive(Debug, Clone, Default)]
struct ThemeStats {
    count: usize,
    weight: f64,
}

#[derive(Debug, Clone)]
pub struct ConceptBoard {
    pub code: String,
    pub name: String,
    pub change_pct: f64,
    pub rise_count: u32,
    pub fall_count: u32,
}

pub struct ThematicScoringEngine {
    client: reqwest::Client,
}

impl ThematicScoringEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: build_stock_client()?,
        })
    }

    pub async fn build_sentiment_map(&self) -> Result<ThematicScoringResult> {
        let (cls_news, em_news, sina_news, sina7x24_news, wscn_news) = tokio::join!(
            fetch_cls_telegraph(120),
            fetch_eastmoney_news(1, 80),
            fetch_sina_roll_news(1, 80),
            fetch_sina_7x24(50),
            fetch_wallstreetcn_lives(50)
        );

        let mut all_news: Vec<NewsItem> = Vec::new();
        if let Ok(v) = cls_news { all_news.extend(v); }
        if let Ok(v) = em_news { all_news.extend(v); }
        if let Ok(v) = sina_news { all_news.extend(v); }
        if let Ok(v) = sina7x24_news { all_news.extend(v); }
        if let Ok(v) = wscn_news { all_news.extend(v); }

        if all_news.is_empty() {
            return Ok(ThematicScoringResult::default());
        }

        let rules = theme_rules();
        let mut theme_stats: HashMap<&'static str, ThemeStats> = HashMap::new();
        let mut direct_theme_codes: HashMap<&'static str, HashSet<String>> = HashMap::new();

        for item in &all_news {
            let text = format!("{} {}", item.title, item.summary);
            for rule in &rules {
                if rule.keywords.iter().any(|k| text.contains(k)) {
                    let entry = theme_stats.entry(rule.name).or_default();
                    entry.count += 1;
                    entry.weight += 1.0 + item.importance as f64 * 0.45;

                    if !item.related_stocks.is_empty() {
                        let set = direct_theme_codes.entry(rule.name).or_default();
                        for c in &item.related_stocks {
                            if let Some(code) = normalize_code(c) {
                                set.insert(code);
                            }
                        }
                    }
                }
            }
        }

        if theme_stats.is_empty() {
            return Ok(ThematicScoringResult::default());
        }

        let mut ranked_themes: Vec<(&ThemeRule, ThemeStats)> = rules
            .iter()
            .filter_map(|rule| theme_stats.get(rule.name).map(|s| (rule, s.clone())))
            .collect();

        ranked_themes.sort_by(|a, b| b.1.weight.partial_cmp(&a.1.weight).unwrap_or(std::cmp::Ordering::Equal));
        ranked_themes.truncate(5);

        let boards = self.fetch_concept_boards().await.unwrap_or_default();

        let mut stock_score_raw: HashMap<String, f64> = HashMap::new();
        let mut stock_heat_raw: HashMap<String, f64> = HashMap::new();
        let mut stock_themes: HashMap<String, Vec<String>> = HashMap::new();

        for (rule, stats) in ranked_themes {
            let theme_strength = (stats.weight / 6.0).clamp(0.15, 1.0);

            if let Some(codes) = direct_theme_codes.get(rule.name) {
                for code in codes {
                    *stock_score_raw.entry(code.clone()).or_insert(0.0) += theme_strength * 1.25;
                    *stock_heat_raw.entry(code.clone()).or_insert(0.0) += theme_strength;
                    push_theme(&mut stock_themes, code, rule.name);
                }
            }

            let matched_boards: Vec<&ConceptBoard> = boards
                .iter()
                .filter(|b| rule.concept_keywords.iter().any(|k| b.name.contains(k)))
                .collect();

            let mut board_rank = matched_boards;
            board_rank.sort_by(|a, b| b.change_pct.partial_cmp(&a.change_pct).unwrap_or(std::cmp::Ordering::Equal));

            for board in board_rank.into_iter().take(3) {
                let board_boost = if board.change_pct >= 0.0 { 1.0 } else { 0.75 };
                let members = self.fetch_board_members(&board.code).await.unwrap_or_default();
                for code in members {
                    *stock_score_raw.entry(code.clone()).or_insert(0.0) += theme_strength * board_boost;
                    *stock_heat_raw.entry(code.clone()).or_insert(0.0) += theme_strength * 0.85;
                    push_theme(&mut stock_themes, &code, rule.name);
                }
            }
        }

        if stock_score_raw.is_empty() {
            return Ok(ThematicScoringResult::default());
        }

        let max_score = stock_score_raw.values().copied().fold(0.0, f64::max).max(1.0);
        let max_heat = stock_heat_raw.values().copied().fold(0.0, f64::max).max(1.0);

        let stock_sentiment = stock_score_raw
            .into_iter()
            .map(|(k, v)| (k, (v / max_score).clamp(0.0, 1.0)))
            .collect();

        let stock_heat = stock_heat_raw
            .into_iter()
            .map(|(k, v)| (k, (v / max_heat).clamp(0.0, 1.0)))
            .collect();

        Ok(ThematicScoringResult {
            stock_sentiment,
            stock_heat,
            stock_themes,
        })
    }

    async fn fetch_concept_boards(&self) -> Result<Vec<ConceptBoard>> {
        let url = "https://push2.eastmoney.com/api/qt/clist/get?pn=1&pz=500&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281&fltt=2&invt=2&fid=f3&fs=m:90+t:3&fields=f2,f3,f12,f14,f104,f105";
        let resp = self.client.get(url)
            .header("Referer", "https://quote.eastmoney.com/")
            .send().await?;
        let json: serde_json::Value = resp.json().await?;
        let mut boards = Vec::new();

        if let Some(items) = json.get("data").and_then(|d| d.get("diff")).and_then(|v| v.as_array()) {
            for it in items {
                let code = it.get("f12").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let name = it.get("f14").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if code.is_empty() || name.is_empty() {
                    continue;
                }
                boards.push(ConceptBoard {
                    code,
                    name,
                    change_pct: parse_f64(it.get("f3")),
                    rise_count: parse_f64(it.get("f104")) as u32,
                    fall_count: parse_f64(it.get("f105")) as u32,
                });
            }
        }

        Ok(boards)
    }

    /// 公开方法：获取概念板块列表（供 stock_tools 调用）
    pub async fn fetch_concept_boards_public(&self) -> Result<Vec<ConceptBoard>> {
        self.fetch_concept_boards().await
    }

    async fn fetch_board_members(&self, board_code: &str) -> Result<Vec<String>> {
        let url = format!(
            "https://push2.eastmoney.com/api/qt/clist/get?pn=1&pz=300&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281&fltt=2&invt=2&fid=f3&fs=b:{}&fields=f12,f13",
            board_code
        );

        let resp = self.client.get(&url)
            .header("Referer", "https://quote.eastmoney.com/")
            .send().await?;
        let json: serde_json::Value = resp.json().await?;

        let mut members = Vec::new();
        if let Some(items) = json.get("data").and_then(|d| d.get("diff")).and_then(|v| v.as_array()) {
            for it in items {
                let code_num = it.get("f12").and_then(|v| v.as_str()).unwrap_or("");
                let market = it.get("f13").and_then(|v| v.as_i64()).unwrap_or(0);
                if code_num.is_empty() {
                    continue;
                }
                let prefix = if market == 1 { "sh" } else { "sz" };
                members.push(format!("{}{}", prefix, code_num));
            }
        }

        Ok(members)
    }

    /// 公开方法：获取板块成分股并携带行情数据（供 stock_tools 调用）
    pub async fn fetch_board_members_with_data(&self, board_code: &str) -> Result<Vec<crate::models::stock::MarketStockSnapshot>> {
        let fields = "f2,f3,f4,f5,f6,f7,f8,f9,f10,f12,f13,f14,f15,f16,f17,f18,f20,f21,f23,f24,f25,f37,f115,f62";
        let url = format!(
            "https://push2.eastmoney.com/api/qt/clist/get?pn=1&pz=300&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281&fltt=2&invt=2&fid=f3&fs=b:{}&fields={}",
            board_code, fields
        );

        let resp = self.client.get(&url)
            .header("Referer", "https://quote.eastmoney.com/")
            .send().await?;
        let json: serde_json::Value = resp.json().await?;

        let mut stocks = Vec::new();
        if let Some(items) = json.get("data").and_then(|d| d.get("diff")).and_then(|v| v.as_array()) {
            for it in items {
                if let Some(stock) = crate::services::market_scanner::parse_eastmoney_item_public(it) {
                    stocks.push(stock);
                }
            }
        }

        Ok(stocks)
    }
}

fn push_theme(map: &mut HashMap<String, Vec<String>>, code: &str, theme: &str) {
    let entry = map.entry(code.to_string()).or_default();
    if !entry.iter().any(|x| x == theme) {
        entry.push(theme.to_string());
    }
}

fn normalize_code(raw: &str) -> Option<String> {
    let cleaned = raw.trim().to_lowercase();
    if cleaned.starts_with("sh") || cleaned.starts_with("sz") || cleaned.starts_with("bj") {
        return Some(cleaned);
    }

    let digits: String = cleaned.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 6 {
        return None;
    }

    let prefix = if digits.starts_with('6') { "sh" } else { "sz" };
    Some(format!("{}{}", prefix, digits))
}

fn parse_f64(v: Option<&serde_json::Value>) -> f64 {
    match v {
        Some(val) if val.is_f64() => val.as_f64().unwrap_or(0.0),
        Some(val) if val.is_i64() => val.as_i64().unwrap_or(0) as f64,
        Some(val) if val.is_string() => val.as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0),
        _ => 0.0,
    }
}

fn theme_rules() -> Vec<ThemeRule> {
    vec![
        ThemeRule {
            name: "海南自贸港",
            keywords: &["海南", "自贸港", "封关", "离岛免税"],
            concept_keywords: &["海南", "自贸港", "免税"],
        },
        ThemeRule {
            name: "黄金避险",
            keywords: &["黄金", "避险", "地缘", "冲突", "国际形势"],
            concept_keywords: &["黄金", "贵金属"],
        },
        ThemeRule {
            name: "英伟达产业链",
            keywords: &["英伟达", "nvidia", "gpu", "算力", "液冷", "hbm"],
            concept_keywords: &["英伟达", "液冷", "算力", "服务器", "gpu"],
        },
        ThemeRule {
            name: "军工安全",
            keywords: &["军工", "国防", "导弹", "装备", "安全"],
            concept_keywords: &["军工", "国防"],
        },
        ThemeRule {
            name: "新能源储能",
            keywords: &["光伏", "风电", "储能", "锂电", "新能源"],
            concept_keywords: &["储能", "光伏", "锂电", "风电", "新能源"],
        },
        ThemeRule {
            name: "半导体自主",
            keywords: &["半导体", "芯片", "国产替代", "先进封装", "晶圆"],
            concept_keywords: &["芯片", "半导体", "封装", "EDA"],
        },
    ]
}
