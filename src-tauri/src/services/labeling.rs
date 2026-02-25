use crate::models::stock::MarketStockSnapshot;
use crate::models::strategy::StockLabel;

pub struct LabelingEngine;

impl LabelingEngine {
    /// 基于多因子数据为股票生成结构化标签（买入导向）
    pub fn generate_labels(stock: &MarketStockSnapshot) -> Vec<StockLabel> {
        let mut labels = Vec::new();

        // ===== 买入时机标签（最重要，放在前面）=====

        // 放量突破：今日上涨 + 量比≥1.5 + 5日涨幅为正
        if stock.change_pct > 1.0 && stock.volume_ratio >= 1.5 && stock.pct_5d > 0.0 {
            labels.push(StockLabel {
                text: "放量突破".to_string(),
                color: "#FF4757".to_string(),
                icon: Some("rocket".to_string()),
            });
        }

        // 启动信号：5日涨幅 > 0 且 20日涨幅在0~15%（上涨初期）
        let avg_5d = stock.pct_5d / 5.0;
        let avg_20d = stock.pct_20d / 20.0;
        if avg_5d > avg_20d && avg_5d > 0.0 && stock.pct_20d >= 0.0 && stock.pct_20d <= 15.0 {
            labels.push(StockLabel {
                text: "加速启动".to_string(),
                color: "#FF6348".to_string(),
                icon: Some("trending_up".to_string()),
            });
        }

        // 底部放量：20日跌幅(负)但今日放量上涨（可能是反转信号）
        if stock.pct_20d < -5.0 && stock.pct_20d >= -15.0
            && stock.change_pct > 0.0 && stock.volume_ratio >= 1.5
        {
            labels.push(StockLabel {
                text: "底部放量".to_string(),
                color: "#1ABC9C".to_string(),
                icon: Some("rebound".to_string()),
            });
        }

        // 温和上涨：20日涨幅3%~15%，趋势健康
        if stock.pct_20d >= 3.0 && stock.pct_20d <= 15.0 {
            labels.push(StockLabel {
                text: "趋势健康".to_string(),
                color: "#2ED573".to_string(),
                icon: Some("trend".to_string()),
            });
        }

        // ===== 价值标签 =====
        if stock.pe_ttm > 0.0 && stock.pe_ttm < 15.0 && stock.pb > 0.0 && stock.pb < 2.0 {
            labels.push(StockLabel {
                text: "深度价值".to_string(),
                color: "#2ECC71".to_string(),
                icon: Some("gem".to_string()),
            });
        } else if stock.pe_ttm > 0.0 && stock.pe_ttm < 25.0 {
            labels.push(StockLabel {
                text: "低估值".to_string(),
                color: "#27AE60".to_string(),
                icon: Some("value".to_string()),
            });
        }

        // ===== 质量标签 =====
        if stock.roe >= 20.0 {
            labels.push(StockLabel {
                text: "高ROE".to_string(),
                color: "#3498DB".to_string(),
                icon: Some("quality".to_string()),
            });
        }
        if stock.revenue_yoy > 30.0 {
            labels.push(StockLabel {
                text: "高增长".to_string(),
                color: "#9B59B6".to_string(),
                icon: Some("growth".to_string()),
            });
        }

        // ===== 资金标签 =====
        if stock.main_net_inflow > 50_000_000.0 {
            labels.push(StockLabel {
                text: "主力大幅流入".to_string(),
                color: "#E74C3C".to_string(),
                icon: Some("inflow".to_string()),
            });
        } else if stock.main_net_inflow > 10_000_000.0 {
            labels.push(StockLabel {
                text: "主力流入".to_string(),
                color: "#E67E22".to_string(),
                icon: Some("inflow".to_string()),
            });
        } else if stock.main_net_inflow < -50_000_000.0 {
            labels.push(StockLabel {
                text: "主力流出".to_string(),
                color: "#95A5A6".to_string(),
                icon: Some("outflow".to_string()),
            });
        }

        // ===== 风险警示标签 =====
        if stock.pct_20d > 25.0 {
            labels.push(StockLabel {
                text: "短期过热".to_string(),
                color: "#FF6B6B".to_string(),
                icon: Some("warning".to_string()),
            });
        }

        if stock.volume_ratio >= 2.0 && stock.change_pct < -2.0 {
            labels.push(StockLabel {
                text: "放量下跌".to_string(),
                color: "#636E72".to_string(),
                icon: Some("warning".to_string()),
            });
        }

        // ===== 市值标签 =====
        let cap_yi = stock.total_market_cap / 1e8;
        if cap_yi >= 1000.0 {
            labels.push(StockLabel {
                text: "大盘蓝筹".to_string(),
                color: "#34495E".to_string(),
                icon: Some("blue_chip".to_string()),
            });
        } else if cap_yi < 100.0 && cap_yi >= 30.0 {
            labels.push(StockLabel {
                text: "小盘成长".to_string(),
                color: "#8E44AD".to_string(),
                icon: Some("small_cap".to_string()),
            });
        }

        // ===== 换手率标签 =====
        if stock.turnover_rate > 15.0 {
            labels.push(StockLabel {
                text: "换手过高".to_string(),
                color: "#F39C12".to_string(),
                icon: Some("warning".to_string()),
            });
        }

        labels
    }
}
