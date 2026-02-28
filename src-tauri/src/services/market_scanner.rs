use anyhow::{Result, anyhow};
use crate::models::stock::MarketStockSnapshot;
use crate::utils::http::build_stock_client;

/// 全市场扫描器：通过东方财富 API 获取沪深A股全量多维度数据
pub struct MarketScanner {
    client: reqwest::Client,
}

impl MarketScanner {
    pub fn new() -> Result<Self> {
        let client = build_stock_client()?;
        Ok(Self { client })
    }

    /// 拉取沪深A股全量数据（主板+创业板，不含科创板/北交所）
    /// 字段映射：
    ///   f2=最新价, f3=涨跌幅, f4=涨跌额, f5=成交量(手), f6=成交额,
    ///   f7=振幅, f8=换手率, f9=市盈率TTM, f10=量比, f12=代码, f13=市场(0深1沪),
    ///   f14=名称, f15=最高, f16=最低, f17=今开, f18=昨收,
    ///   f20=总市值, f21=流通市值, f23=市净率,
    ///   f24=近5日涨幅, f25=近20日涨幅, f22=近60日涨幅(?),
    ///   f37=净资产收益率ROE, f115=营收同比增长,
    ///   f62=主力净流入, f184=主力净占比(not in clist, need zjlx)
    /// 分页拉取，每页5000条
    pub async fn scan_full_market(&self) -> Result<Vec<MarketStockSnapshot>> {
        let mut all_stocks = Vec::new();
        let mut page = 1;

        loop {
            let stocks = self.fetch_page(page).await?;
            if stocks.is_empty() {
                break;
            }
            let count = stocks.len();
            all_stocks.extend(stocks);
            if count < 5000 {
                break;
            }
            page += 1;
        }

        Ok(all_stocks)
    }

    async fn fetch_page(&self, page: u32) -> Result<Vec<MarketStockSnapshot>> {
        // fs 参数: m:0 t:6 (深市主板) + m:0 t:80 (深市创业板) + m:1 t:2 (沪市主板) + m:1 t:23 (沪市创业板科创板?)
        // 实际上沪市创业板不存在，t:23 是上证科创板
        // 沪深主板+创业板: m:0 t:6, m:0 t:80, m:1 t:2, m:1 t:23 去掉科创板
        // 只要主板+创业板: m:0 t:6 (深主板), m:0 t:80 (创业板), m:1 t:2 (沪主板)
        let fs = "m:0+t:6,m:0+t:80,m:1+t:2";
        let fields = "f2,f3,f4,f5,f6,f7,f8,f9,f10,f12,f13,f14,f15,f16,f17,f18,f20,f21,f23,f24,f25,f26,f37,f115,f62";

        let url = format!(
            "https://push2.eastmoney.com/api/qt/clist/get?pn={}&pz=5000&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281&fltt=2&invt=2&fid=f3&fs={}&fields={}",
            page, fs, fields
        );

        let resp = self.client.get(&url)
            .header("Referer", "https://quote.eastmoney.com/")
            .send().await?;
        let text = resp.text().await?;
        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| anyhow!("东方财富数据解析失败: {}", e))?;

        let data = json.get("data").ok_or_else(|| anyhow!("响应缺少 data 字段"))?;
        let diff = match data.get("diff") {
            Some(d) => d,
            None => return Ok(vec![]),
        };

        let items = match diff.as_array() {
            Some(arr) => arr,
            None => return Ok(vec![]),
        };

        let mut stocks = Vec::with_capacity(items.len());
        for item in items {
            if let Some(stock) = parse_eastmoney_item(item) {
                stocks.push(stock);
            }
        }

        Ok(stocks)
    }

    /// 按代码列表获取多维度快照数据（PE/PB/ROE/市值/换手率/量比/主力净流入等）
    /// 使用东方财富 ulist.np API，支持指定 secids
    pub async fn fetch_stocks_by_codes(&self, codes: &[String]) -> Result<Vec<MarketStockSnapshot>> {
        if codes.is_empty() {
            return Ok(vec![]);
        }

        let secids: Vec<String> = codes.iter().map(|c| code_to_secid(c)).collect();
        let secid_str = secids.join(",");
        let fields = "f2,f3,f4,f5,f6,f7,f8,f9,f10,f12,f13,f14,f15,f16,f17,f18,f20,f21,f23,f24,f25,f26,f37,f115,f62";

        let url = format!(
            "https://push2.eastmoney.com/api/qt/ulist.np/get?fltt=2&invt=2&fields={}&secids={}",
            fields, secid_str
        );

        let resp = self.client.get(&url)
            .header("Referer", "https://quote.eastmoney.com/")
            .send().await?;
        let json: serde_json::Value = resp.json().await?;

        let mut stocks = Vec::new();
        if let Some(data) = json.get("data") {
            if let Some(diff) = data.get("diff") {
                if let Some(items) = diff.as_array() {
                    for item in items {
                        if let Some(stock) = parse_eastmoney_item(item) {
                            stocks.push(stock);
                        }
                    }
                }
            }
        }

        Ok(stocks)
    }

    /// 拉取个股资金流向数据（主力净流入），合并到快照中
    /// 这是一个补充接口，用于弥补 clist 中 f62 可能不准确的问题
    pub async fn fetch_fund_flow(&self, codes: &[String]) -> Result<Vec<(String, f64, f64)>> {
        if codes.is_empty() {
            return Ok(vec![]);
        }

        // 构建 secids: "1.600000,0.000001,..."
        let secids: Vec<String> = codes.iter().map(|c| code_to_secid(c)).collect();
        let secid_str = secids.join(",");

        let url = format!(
            "https://push2.eastmoney.com/api/qt/ulist.np/get?fltt=2&invt=2&fields=f3,f12,f13,f62,f184&secids={}",
            secid_str
        );

        let resp = self.client.get(&url)
            .header("Referer", "https://quote.eastmoney.com/")
            .send().await?;
        let json: serde_json::Value = resp.json().await?;

        let mut results = Vec::new();
        if let Some(data) = json.get("data") {
            if let Some(diff) = data.get("diff") {
                if let Some(items) = diff.as_array() {
                    for item in items {
                        let code_num = item.get("f12").and_then(|v| v.as_str()).unwrap_or("");
                        let market = item.get("f13").and_then(|v| v.as_i64()).unwrap_or(0);
                        let net_inflow = item.get("f62").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let net_pct = item.get("f184").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let prefix = if market == 1 { "sh" } else { "sz" };
                        results.push((format!("{}{}", prefix, code_num), net_inflow, net_pct));
                    }
                }
            }
        }

        Ok(results)
    }
}

fn parse_eastmoney_item(item: &serde_json::Value) -> Option<MarketStockSnapshot> {
    parse_eastmoney_item_public(item)
}

/// 公开版本：供其他模块复用（如 thematic_scoring）
pub fn parse_eastmoney_item_public(item: &serde_json::Value) -> Option<MarketStockSnapshot> {
    let code_num = item.get("f12")?.as_str()?;
    let market = item.get("f13")?.as_i64()?;
    let name = item.get("f14")?.as_str()?.to_string();

    // 跳过无效数据
    let price = get_f64(item, "f2");
    if price <= 0.0 {
        return None;
    }

    let prefix = if market == 1 { "sh" } else { "sz" };
    let code = format!("{}{}", prefix, code_num);

    Some(MarketStockSnapshot {
        code,
        name,
        price,
        change_pct: get_f64(item, "f3"),
        change_amount: get_f64(item, "f4"),
        volume: get_f64(item, "f5"),
        amount: get_f64(item, "f6"),
        amplitude: get_f64(item, "f7"),
        turnover_rate: get_f64(item, "f8"),
        pe_ttm: get_f64(item, "f9"),
        volume_ratio: get_f64(item, "f10"),
        high: get_f64(item, "f15"),
        low: get_f64(item, "f16"),
        open: get_f64(item, "f17"),
        pre_close: get_f64(item, "f18"),
        total_market_cap: get_f64(item, "f20"),
        float_market_cap: get_f64(item, "f21"),
        pb: get_f64(item, "f23"),
        pct_5d: get_f64(item, "f24"),
        pct_20d: get_f64(item, "f25"),
        pct_60d: 0.0, // clist 中未直接提供60日涨幅
        roe: get_f64(item, "f37"),
        gross_margin: 0.0,   // 需要单独接口
        revenue_yoy: get_f64(item, "f115"),
        profit_yoy: 0.0,     // 需要单独接口或其他字段
        main_net_inflow: get_f64(item, "f62"),
        main_net_pct: 0.0,   // clist 无此字段
        list_date: item.get("f26")
            .and_then(|v| v.as_str())
            .unwrap_or("-")
            .to_string(),     // 上市日期 "YYYYMMDD" 或 "-"
    })
}

fn get_f64(item: &serde_json::Value, key: &str) -> f64 {
    item.get(key)
        .and_then(|v| {
            if v.is_f64() {
                v.as_f64()
            } else if v.is_i64() {
                v.as_i64().map(|i| i as f64)
            } else if v.is_string() {
                v.as_str().and_then(|s| s.parse::<f64>().ok())
            } else {
                None
            }
        })
        .unwrap_or(0.0)
}

fn code_to_secid(code: &str) -> String {
    let code = code.to_lowercase();
    if code.starts_with("sh") {
        format!("1.{}", &code[2..])
    } else if code.starts_with("sz") {
        format!("0.{}", &code[2..])
    } else {
        format!("0.{}", code)
    }
}
