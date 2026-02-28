use anyhow::{Result, anyhow};
use crate::models::stock::MarketStockSnapshot;
use crate::utils::http::build_stock_client;

/// 全市场扫描器：通过东方财富 API 获取沪深A股全量多维度数据
/// 当东财接口不可用时（非交易时间/限流），自动 fallback 到腾讯行情接口
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
        let fs = "m:0+t:6,m:0+t:80,m:1+t:2";
        let fields = "f2,f3,f4,f5,f6,f7,f8,f9,f10,f12,f13,f14,f15,f16,f17,f18,f20,f21,f23,f24,f25,f26,f37,f115,f62";

        let url = format!(
            "https://push2.eastmoney.com/api/qt/clist/get?pn={}&pz=5000&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281&fltt=2&invt=2&fid=f3&fs={}&fields={}",
            page, fs, fields
        );

        let text = match self.client.get(&url)
            .header("Referer", "https://quote.eastmoney.com/")
            .send().await
        {
            Ok(resp) => resp.text().await.unwrap_or_default(),
            Err(e) => {
                log::warn!("东财全市场接口请求失败: {}，返回空列表", e);
                return Ok(vec![]);
            }
        };

        if text.is_empty() {
            log::warn!("东财全市场接口返回空响应（可能非交易时间），page={}", page);
            return Ok(vec![]);
        }

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
    /// 优先东方财富 ulist.np，失败时 fallback 到腾讯 qt.gtimg.cn
    pub async fn fetch_stocks_by_codes(&self, codes: &[String]) -> Result<Vec<MarketStockSnapshot>> {
        if codes.is_empty() {
            return Ok(vec![]);
        }

        // 尝试东财
        match self.fetch_stocks_by_codes_eastmoney(codes).await {
            Ok(stocks) if !stocks.is_empty() => return Ok(stocks),
            Ok(_) => log::info!("东财 ulist 返回空，fallback 腾讯行情"),
            Err(e) => log::warn!("东财 ulist 失败: {}，fallback 腾讯行情", e),
        }

        // Fallback: 腾讯
        self.fetch_stocks_by_codes_tencent(codes).await
    }

    /// 东财 ulist.np 接口
    async fn fetch_stocks_by_codes_eastmoney(&self, codes: &[String]) -> Result<Vec<MarketStockSnapshot>> {
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
        let text = resp.text().await?;
        if text.is_empty() {
            return Ok(vec![]);
        }
        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| anyhow!("东财数据解析失败: {}", e))?;

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

    /// 腾讯行情接口 fallback（qt.gtimg.cn）
    /// 字段较少（无 ROE/营收增速/主力资金流向），但基础价格数据齐全且非交易时间也能用
    async fn fetch_stocks_by_codes_tencent(&self, codes: &[String]) -> Result<Vec<MarketStockSnapshot>> {
        // 腾讯接口每次最多约 50 只，分批请求
        let mut all_stocks = Vec::new();
        for chunk in codes.chunks(50) {
            let symbols: Vec<String> = chunk.iter().map(|c| code_to_tencent_symbol(c)).collect();
            let symbols_str = symbols.join(",");
            let url = format!("https://qt.gtimg.cn/q={}", symbols_str);

            let resp = self.client.get(&url)
                .header("Referer", "https://finance.qq.com/")
                .send().await?;
            // 腾讯接口返回 GBK 编码，reqwest 默认按 UTF-8 读取会乱码
            // 但数值字段不受影响，名称可能乱码
            let bytes = resp.bytes().await?;
            let (text, _, _) = encoding_rs::GBK.decode(&bytes);

            for line in text.lines() {
                if let Some(stock) = parse_tencent_quote(line) {
                    all_stocks.push(stock);
                }
            }
        }
        Ok(all_stocks)
    }

    /// 拉取个股资金流向数据（主力净流入），合并到快照中
    /// 东财失败时优雅降级，返回空列表（腾讯无资金流向数据）
    pub async fn fetch_fund_flow(&self, codes: &[String]) -> Result<Vec<(String, f64, f64)>> {
        if codes.is_empty() {
            return Ok(vec![]);
        }

        let secids: Vec<String> = codes.iter().map(|c| code_to_secid(c)).collect();
        let secid_str = secids.join(",");

        let url = format!(
            "https://push2.eastmoney.com/api/qt/ulist.np/get?fltt=2&invt=2&fields=f3,f12,f13,f62,f184&secids={}",
            secid_str
        );

        let text = match self.client.get(&url)
            .header("Referer", "https://quote.eastmoney.com/")
            .send().await
        {
            Ok(resp) => resp.text().await.unwrap_or_default(),
            Err(e) => {
                log::warn!("东财资金流向接口请求失败: {}，跳过", e);
                return Ok(vec![]);
            }
        };

        if text.is_empty() {
            log::warn!("东财资金流向接口返回空响应（可能非交易时间），跳过");
            return Ok(vec![]);
        }

        let json: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("东财资金流向数据解析失败: {}，跳过", e);
                return Ok(vec![]);
            }
        };

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

/// 解析腾讯行情接口返回的单行数据
/// 格式: v_sz000002="51~万  科Ａ~000002~4.84~4.82~4.83~1132768~..."
/// 字段以 ~ 分隔，索引含义：
///   [1]=名称, [2]=代码, [3]=最新价, [4]=昨收, [5]=今开,
///   [6]=成交量(手), [32]=涨跌额, [33]=涨跌幅%, [34]=最高, [35]=最低,
///   [38]=成交额(万), [39]=换手率%, [44]=振幅%,
///   [45]=总市值(亿), [46]=流通市值(亿), [47]=PE(TTM),
///   [49]=PB (?), [50]=量比
fn parse_tencent_quote(line: &str) -> Option<MarketStockSnapshot> {
    // 格式: v_sz000002="51~...~";\n
    let line = line.trim();
    if line.is_empty() || !line.starts_with("v_") {
        return None;
    }

    // 提取市场前缀 (sz/sh)
    let prefix = if line.starts_with("v_sz") {
        "sz"
    } else if line.starts_with("v_sh") {
        "sh"
    } else {
        return None;
    };

    // 提取引号内的内容
    let start = line.find('"')? + 1;
    let end = line.rfind('"')?;
    if start >= end {
        return None;
    }
    let content = &line[start..end];
    let fields: Vec<&str> = content.split('~').collect();

    if fields.len() < 50 {
        return None;
    }

    let code_num = fields.get(2)?;
    let price: f64 = fields.get(3)?.parse().ok()?;
    if price <= 0.0 {
        return None;
    }

    let code = format!("{}{}", prefix, code_num);
    let name = fields.get(1).unwrap_or(&"").to_string();

    let pre_close = parse_field(fields.get(4));
    let open = parse_field(fields.get(5));
    let volume = parse_field(fields.get(6)); // 手
    let change_amount = parse_field(fields.get(32));
    let change_pct = parse_field(fields.get(33));
    let high = parse_field(fields.get(34));
    let low = parse_field(fields.get(35));
    let amount = parse_field(fields.get(38)) * 10_000.0; // 万 → 元
    let turnover_rate = parse_field(fields.get(39));
    let amplitude = parse_field(fields.get(44));
    let total_market_cap = parse_field(fields.get(45)) * 100_000_000.0; // 亿 → 元
    let float_market_cap = parse_field(fields.get(46)) * 100_000_000.0; // 亿 → 元
    let pe_ttm = parse_field(fields.get(47));
    let pb = parse_field(fields.get(49));
    let volume_ratio = parse_field(fields.get(50));

    Some(MarketStockSnapshot {
        code,
        name,
        price,
        change_pct,
        change_amount,
        volume,
        amount,
        amplitude,
        turnover_rate,
        pe_ttm,
        volume_ratio,
        high,
        low,
        open,
        pre_close,
        total_market_cap,
        float_market_cap,
        pb,
        pct_5d: 0.0,           // 腾讯接口无此字段
        pct_20d: 0.0,
        pct_60d: 0.0,
        roe: 0.0,              // 腾讯接口无此字段
        gross_margin: 0.0,
        revenue_yoy: 0.0,
        profit_yoy: 0.0,
        main_net_inflow: 0.0,  // 腾讯接口无此字段
        main_net_pct: 0.0,
        list_date: String::new(),
    })
}

/// 安全解析浮点数
fn parse_field(field: Option<&&str>) -> f64 {
    field.and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0)
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

/// 转换股票代码为腾讯行情接口格式
/// "sh600519" → "sh600519", "sz000002" → "sz000002", "000002" → "sz000002"
fn code_to_tencent_symbol(code: &str) -> String {
    let code = code.to_lowercase();
    if code.starts_with("sh") || code.starts_with("sz") {
        code
    } else if code.starts_with("6") {
        format!("sh{}", code)
    } else {
        format!("sz{}", code)
    }
}
