use std::collections::HashMap;

use crate::models::stock::MarketStockSnapshot;
use crate::models::strategy::{FactorWeights, FactorScoreDetail, StockFilters};

pub struct MultiFactorEngine;

impl MultiFactorEngine {
    /// 对全市场股票列表进行过滤 + 打分 + 排名（无消息面）
    pub fn screen_and_rank(
        stocks: &[MarketStockSnapshot],
        weights: &FactorWeights,
        filters: &StockFilters,
        top_n: usize,
    ) -> Vec<(MarketStockSnapshot, u32, FactorScoreDetail)> {
        let empty: HashMap<String, f64> = HashMap::new();
        Self::screen_and_rank_with_sentiment(stocks, weights, filters, top_n, &empty)
    }

    /// 消息面融合打分：将新闻/主题热度映射到 sentiment 因子
    pub fn screen_and_rank_with_sentiment(
        stocks: &[MarketStockSnapshot],
        weights: &FactorWeights,
        filters: &StockFilters,
        top_n: usize,
        sentiment_scores: &HashMap<String, f64>,
    ) -> Vec<(MarketStockSnapshot, u32, FactorScoreDetail)> {
        // Step 1: 过滤
        let filtered: Vec<&MarketStockSnapshot> = stocks.iter()
            .filter(|s| Self::passes_filter(s, filters))
            .collect();

        if filtered.is_empty() {
            return vec![];
        }

        // Step 2: 计算各维度百分位排名（0-1），用于标准化
        let pe_values: Vec<f64> = filtered.iter().map(|s| s.pe_ttm).collect();
        let pb_values: Vec<f64> = filtered.iter().map(|s| s.pb).collect();
        let roe_values: Vec<f64> = filtered.iter().map(|s| s.roe).collect();
        let revenue_yoy_values: Vec<f64> = filtered.iter().map(|s| s.revenue_yoy).collect();
        let main_net_values: Vec<f64> = filtered.iter().map(|s| s.main_net_inflow).collect();

        // Step 3: 对每只股票打分
        let mut scored: Vec<(MarketStockSnapshot, u32, FactorScoreDetail)> = filtered.iter().map(|stock| {
            let mut detail = Self::compute_factor_scores(
                stock,
                &pe_values, &pb_values, &roe_values, &revenue_yoy_values,
                &main_net_values,
            );

            detail.sentiment_score = sentiment_scores
                .get(&stock.code)
                .copied()
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);

            let total = detail.value_score * weights.value as f64
                + detail.quality_score * weights.quality as f64
                + detail.momentum_score * weights.momentum as f64
                + detail.capital_score * weights.capital as f64
                + detail.risk_score * weights.risk as f64
                + detail.sentiment_score * weights.sentiment as f64;

            let weight_sum = (weights.value + weights.quality + weights.momentum + weights.capital + weights.risk + weights.sentiment) as f64;
            let score = if weight_sum > 0.0 {
                ((total / weight_sum) * 100.0).round().min(100.0).max(0.0) as u32
            } else {
                0
            };

            ((*stock).clone(), score, detail)
        }).collect();

        // Step 4: 按得分降序排名
        scored.sort_by(|a, b| b.1.cmp(&a.1));

        // Step 5: 取 Top N
        scored.truncate(top_n);

        scored
    }

    /// 检查股票是否通过过滤条件
    fn passes_filter(stock: &MarketStockSnapshot, f: &StockFilters) -> bool {
        // 排除 ST
        if f.exclude_st && (stock.name.contains("ST") || stock.name.contains("st")) {
            return false;
        }

        // 排除停牌（成交额=0或价格=0）
        if stock.price <= 0.0 || stock.amount <= 0.0 {
            return false;
        }

        // 最低股价
        if stock.price < f.min_price {
            return false;
        }

        // 市值过滤（转换为亿元）
        let cap_yi = stock.total_market_cap / 1e8;
        if cap_yi < f.min_market_cap {
            return false;
        }
        if f.max_market_cap > 0.0 && cap_yi > f.max_market_cap {
            return false;
        }

        // 成交额过滤（转换为万元）
        let amount_wan = stock.amount / 1e4;
        if amount_wan < f.min_amount {
            return false;
        }

        // PE 过滤
        if f.pe_min > 0.0 && stock.pe_ttm < f.pe_min {
            return false;
        }
        if f.pe_max > 0.0 && stock.pe_ttm > f.pe_max {
            return false;
        }

        // PB 过滤
        if f.pb_max > 0.0 && stock.pb > f.pb_max {
            return false;
        }

        // ROE 过滤
        if f.roe_min > 0.0 && stock.roe < f.roe_min {
            return false;
        }

        // ===== 趋势过滤：排除明显下跌趋势的股票 =====
        // 20日跌幅超过15%的不选（深度下跌趋势）
        if stock.pct_20d < -15.0 {
            return false;
        }

        // 今日跌停的不选（涨跌幅 < -9.5%）
        if stock.change_pct < -9.5 {
            return false;
        }

        // ===== 次新股过滤：排除上市不足 N 天的新股 =====
        if f.exclude_new_stock_days > 0 && !stock.list_date.is_empty() && stock.list_date != "-" {
            if let Ok(list_dt) = chrono::NaiveDate::parse_from_str(&stock.list_date, "%Y%m%d") {
                let today = chrono::Local::now().date_naive();
                let days_listed = (today - list_dt).num_days();
                if days_listed >= 0 && days_listed < f.exclude_new_stock_days as i64 {
                    return false;
                }
            }
        }

        true
    }

    /// 计算五大因子得分（每个 0-1）
    /// 改造核心：引入买入时机判断，选出"基本面好 + 当前适合买入"的股票
    fn compute_factor_scores(
        stock: &MarketStockSnapshot,
        pe_vals: &[f64], pb_vals: &[f64], roe_vals: &[f64], revenue_vals: &[f64],
        main_net_vals: &[f64],
    ) -> FactorScoreDetail {
        // ===== 价值因子 =====
        // PE 越低越好（反向百分位），PB 越低越好（反向百分位）
        let pe_score = 1.0 - percentile_rank(stock.pe_ttm, pe_vals);
        let pb_score = 1.0 - percentile_rank(stock.pb, pb_vals);
        let value_score = pe_score * 0.5 + pb_score * 0.5;

        // ===== 质量因子 =====
        // ROE 越高越好，营收增速越高越好
        let roe_score = percentile_rank(stock.roe, roe_vals);
        let revenue_score = percentile_rank(stock.revenue_yoy, revenue_vals);
        let quality_score = roe_score * 0.6 + revenue_score * 0.4;

        // ===== 动量因子（重构：买入时机导向）=====
        // 核心思想：选"正在启动上涨"的股票，而非"已经涨了很多"的
        let momentum_score = Self::compute_momentum_timing(stock);

        // ===== 资金因子（增强：量价配合）=====
        let capital_score = Self::compute_capital_timing(stock, main_net_vals);

        // ===== 风险因子 =====
        // 市值不要太小也不要太大，波动率（振幅）适中
        let cap_score = Self::score_market_cap_optimal(stock.total_market_cap / 1e8);
        let amp_score = Self::score_amplitude_optimal(stock.amplitude);
        let risk_score = cap_score * 0.5 + amp_score * 0.5;

        FactorScoreDetail {
            value_score: value_score.max(0.0).min(1.0),
            quality_score: quality_score.max(0.0).min(1.0),
            momentum_score: momentum_score.max(0.0).min(1.0),
            capital_score: capital_score.max(0.0).min(1.0),
            risk_score: risk_score.max(0.0).min(1.0),
            sentiment_score: 0.0,
        }
    }

    /// 动量因子-买入时机评分
    /// 核心逻辑：
    /// 1. 短期趋势（今日涨幅）：当日温和上涨(0%~5%)得分最高，暴涨/下跌扣分
    /// 2. 中期趋势（20日涨幅）：适度上涨(0%~15%)得分最高，说明处于上涨初中期
    /// 3. 短中期配合（5日 vs 20日）：5日强于20日平均→加速启动信号
    /// 4. 量比配合：适中量比(0.8~3.0)说明有资金关注但不过热
    fn compute_momentum_timing(stock: &MarketStockSnapshot) -> f64 {
        // --- 1. 今日涨幅评分（偏好温和上涨）---
        let today_score = Self::score_today_change(stock.change_pct);

        // --- 2. 中期趋势评分（20日涨幅：上涨初中期最佳）---
        let trend_score = Self::score_mid_trend(stock.pct_20d);

        // --- 3. 短中期加速信号 ---
        // 5日日均涨幅 vs 20日日均涨幅，5日更强说明在加速
        let avg_5d = stock.pct_5d / 5.0;
        let avg_20d = stock.pct_20d / 20.0;
        let accel_score = if avg_5d > avg_20d && avg_5d > 0.0 {
            // 短期加速上涨，奖励
            (1.0 + (avg_5d - avg_20d).min(1.0) * 0.5).min(1.0)
        } else if avg_5d > 0.0 {
            // 短期仍在涨，但速度放缓
            0.6
        } else if stock.pct_20d > 0.0 && stock.pct_5d > -3.0 {
            // 中期上涨但短期小幅回调（可能是回调买点）
            0.5
        } else {
            // 短期下跌
            0.2
        };

        // --- 4. 量比配合 ---
        let vol_ratio_score = Self::score_volume_ratio_timing(stock.volume_ratio);

        // 综合：趋势40% + 今日表现25% + 加速信号20% + 量比15%
        trend_score * 0.40 + today_score * 0.25 + accel_score * 0.20 + vol_ratio_score * 0.15
    }

    /// 今日涨幅评分：温和上涨最佳
    fn score_today_change(pct: f64) -> f64 {
        if pct >= 0.5 && pct <= 5.0 {
            // 温和上涨：最佳买入窗口
            1.0
        } else if pct > 5.0 && pct <= 7.0 {
            // 较大涨幅：可以但追高风险增加
            0.7
        } else if pct > 7.0 {
            // 大涨/涨停：追高风险大，不适合买入
            0.2
        } else if pct >= -1.0 && pct < 0.5 {
            // 小幅波动/微跌：尚可
            0.6
        } else if pct >= -3.0 {
            // 小幅下跌
            0.4
        } else {
            // 大幅下跌：不是好的买入时机
            0.1
        }
    }

    /// 中期趋势评分：处于上涨初中期最佳
    fn score_mid_trend(pct_20d: f64) -> f64 {
        if pct_20d >= 3.0 && pct_20d <= 15.0 {
            // 上涨初中期：最佳！趋势已确立但未过热
            1.0
        } else if pct_20d > 0.0 && pct_20d < 3.0 {
            // 刚开始上涨或横盘微涨
            0.7
        } else if pct_20d > 15.0 && pct_20d <= 25.0 {
            // 涨幅偏大，有追高风险
            0.5
        } else if pct_20d > 25.0 {
            // 严重过热，不宜追入
            0.2
        } else if pct_20d >= -5.0 && pct_20d < 0.0 {
            // 小幅回调，可能是低吸机会（需配合其他因子）
            0.5
        } else if pct_20d >= -15.0 {
            // 中度下跌（已被过滤器放过来的边界）
            0.2
        } else {
            0.0
        }
    }

    /// 量比评分（买入时机导向）：适中放量最佳
    fn score_volume_ratio_timing(vr: f64) -> f64 {
        if vr <= 0.0 { return 0.1; }
        if vr >= 0.8 && vr <= 3.0 {
            // 适度放量：有资金关注
            1.0
        } else if vr > 3.0 && vr <= 5.0 {
            // 明显放量：可能是启动信号，也可能是出货
            0.7
        } else if vr < 0.8 && vr >= 0.5 {
            // 略缩量：关注度不足
            0.5
        } else if vr < 0.5 {
            // 严重缩量：无人问津
            0.2
        } else {
            // vr > 5.0：异常放量，风险大
            0.3
        }
    }

    /// 资金因子（增强：量价配合评分）
    fn compute_capital_timing(stock: &MarketStockSnapshot, main_net_vals: &[f64]) -> f64 {
        // 1. 主力净流入百分位
        let main_net_score = percentile_rank(stock.main_net_inflow, main_net_vals);

        // 2. 换手率适中
        let turnover_score = Self::score_turnover_optimal(stock.turnover_rate);

        // 3. 量价配合奖励：放量上涨 > 缩量上涨 > 放量下跌
        let vol_price_bonus = if stock.change_pct > 0.0 && stock.volume_ratio > 1.0 {
            // 放量上涨：资金积极进场
            0.3
        } else if stock.change_pct > 0.0 && stock.volume_ratio <= 1.0 {
            // 缩量上涨：主力控盘或关注度不够
            0.1
        } else if stock.change_pct < 0.0 && stock.volume_ratio > 2.0 {
            // 放量下跌：可能是恐慌出逃，不好
            -0.2
        } else {
            0.0
        };

        (main_net_score * 0.45 + turnover_score * 0.30 + 0.25 + vol_price_bonus).max(0.0).min(1.0)
    }

    /// 换手率评分：2%-8% 最优区间
    fn score_turnover_optimal(tr: f64) -> f64 {
        if tr >= 2.0 && tr <= 8.0 { return 1.0; }
        if tr < 2.0 { return (tr / 2.0).max(0.0); }
        // > 8%: 越高越差
        (1.0 - (tr - 8.0) / 20.0).max(0.0)
    }

    /// 市值评分：50-500亿最优，偏小或偏大都降分
    fn score_market_cap_optimal(cap_yi: f64) -> f64 {
        if cap_yi >= 50.0 && cap_yi <= 500.0 { return 1.0; }
        if cap_yi < 50.0 {
            return (cap_yi / 50.0).max(0.2);
        }
        // > 500亿: 温和衰减
        (1.0 - (cap_yi - 500.0) / 5000.0).max(0.3)
    }

    /// 振幅评分：1%-6% 适中，过大波动风险高
    fn score_amplitude_optimal(amp: f64) -> f64 {
        if amp >= 1.0 && amp <= 6.0 { return 1.0; }
        if amp < 1.0 { return (amp / 1.0).max(0.3); }
        // > 6%: 波动大
        (1.0 - (amp - 6.0) / 10.0).max(0.1)
    }
}

/// 计算 val 在 vals 中的百分位排名 (0.0 - 1.0)
fn percentile_rank(val: f64, vals: &[f64]) -> f64 {
    if vals.is_empty() { return 0.5; }
    let count_below = vals.iter().filter(|&&v| v < val).count();
    count_below as f64 / vals.len() as f64
}
