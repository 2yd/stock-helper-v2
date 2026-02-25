use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, REFERER, USER_AGENT};
use serde_json::Value;
use std::time::Duration;

use crate::models::news::{AnnouncementItem, NewsCategory, NewsItem, ReportItem};

/// 构建新闻请求客户端
fn build_news_client(referer: &str) -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(REFERER, HeaderValue::from_str(referer)?);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(15))
        .gzip(true)
        .build()?;
    Ok(client)
}

// ============================================================
// 1. 财联社电报快讯
// ============================================================

pub async fn fetch_cls_telegraph(count: u32) -> Result<Vec<NewsItem>> {
    let client = build_news_client("https://www.cls.cn/telegraph")?;
    let url = format!(
        "https://www.cls.cn/nodeapi/telegraphList?app=CailianpressWeb&os=web&sv=8.4.6&rn={}",
        count
    );

    let resp = client.get(&url).send().await?;
    let json: Value = resp.json().await?;

    let mut items = Vec::new();
    if let Some(roll_data) = json["data"]["roll_data"].as_array() {
        for item in roll_data {
            let id = item["id"].as_u64().unwrap_or(0).to_string();
            let title = item["title"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let content = item["content"]
                .as_str()
                .or_else(|| item["brief"].as_str())
                .unwrap_or("")
                .to_string();
            let ctime = item["ctime"].as_i64().unwrap_or(0);
            let level = item["level"].as_str().unwrap_or("0");
            let importance = match level {
                "B" | "b" => 2,
                "A" | "a" => 1,
                _ => 0,
            };

            // 提取关联股票
            let mut related_stocks = Vec::new();
            if let Some(stock_list) = item["stock_list"].as_array() {
                for s in stock_list {
                    if let Some(code) = s["code"].as_str() {
                        related_stocks.push(code.to_string());
                    }
                }
            }

            let share_url = item["shareurl"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let publish_time = if ctime > 0 {
                chrono::DateTime::from_timestamp(ctime, 0)
                    .map(|dt| dt.with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap())
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            };

            // 跳过广告
            if item["is_ad"].as_i64().unwrap_or(0) == 1 {
                continue;
            }

            items.push(NewsItem {
                id: format!("cls_{}", id),
                category: NewsCategory::ClsTelegraph,
                title: if title.is_empty() {
                    content.chars().take(60).collect()
                } else {
                    title
                },
                summary: content,
                source: "财联社".to_string(),
                publish_time,
                url: share_url,
                importance,
                related_stocks,
            });
        }
    }

    Ok(items)
}

// ============================================================
// 2. 东方财富财经要闻
// ============================================================

pub async fn fetch_eastmoney_news(page: u32, page_size: u32) -> Result<Vec<NewsItem>> {
    let client = build_news_client("https://www.eastmoney.com/")?;
    let ts = chrono::Utc::now().timestamp_millis();
    // column=350 是沪深要闻
    let url = format!(
        "https://np-listapi.eastmoney.com/comm/web/getNewsByColumns?client=web&biz=web_sczx&column=350&order=1&needInteractData=0&page_index={}&page_size={}&req_trace={}",
        page, page_size, ts
    );

    let resp = client.get(&url).send().await?;
    let json: Value = resp.json().await?;

    let mut items = Vec::new();
    if let Some(list) = json["data"]["list"].as_array() {
        for item in list {
            let code = item["code"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let title = item["title"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let summary = item["summary"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let show_time = item["showTime"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let media = item["mediaName"]
                .as_str()
                .unwrap_or("东方财富")
                .to_string();
            let url_str = item["url"]
                .as_str()
                .unwrap_or("")
                .to_string();

            items.push(NewsItem {
                id: format!("em_{}", code),
                category: NewsCategory::EastmoneyNews,
                title,
                summary,
                source: media,
                publish_time: show_time,
                url: url_str,
                importance: 0,
                related_stocks: Vec::new(),
            });
        }
    }

    Ok(items)
}

// ============================================================
// 3. 东方财富个股新闻（搜索API）
// ============================================================

pub async fn fetch_stock_news(keyword: &str, page: u32, page_size: u32) -> Result<Vec<NewsItem>> {
    let client = build_news_client("https://so.eastmoney.com/")?;

    let param = serde_json::json!({
        "uid": "",
        "keyword": keyword,
        "type": ["cmsArticleWebOld"],
        "client": "web",
        "clientType": "web",
        "clientVersion": "curr",
        "param": {
            "cmsArticleWebOld": {
                "searchScope": "default",
                "sort": "default",
                "pageIndex": page,
                "pageSize": page_size,
                "preTag": "",
                "postTag": ""
            }
        }
    });

    let param_str = urlencoding::encode(&param.to_string()).to_string();
    let url = format!(
        "https://search-api-web.eastmoney.com/search/jsonp?cb=jQuery&param={}",
        param_str
    );

    let resp = client.get(&url).send().await?;
    let text = resp.text().await?;

    // 去掉 JSONP 包装: jQuery(...)
    let json_str = if let Some(start) = text.find('(') {
        let end = text.rfind(')').unwrap_or(text.len());
        &text[start + 1..end]
    } else {
        &text
    };

    let json: Value = serde_json::from_str(json_str)?;

    let mut items = Vec::new();
    if let Some(articles) = json["result"]["cmsArticleWebOld"].as_array() {
        for art in articles {
            let code = art["code"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let title = art["title"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let content = art["content"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let date = art["date"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let media = art["mediaName"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let url_str = art["url"]
                .as_str()
                .unwrap_or("")
                .to_string();

            items.push(NewsItem {
                id: format!("sn_{}", code),
                category: NewsCategory::StockNews,
                title,
                summary: content,
                source: media,
                publish_time: date,
                url: url_str,
                importance: 0,
                related_stocks: vec![keyword.to_string()],
            });
        }
    }

    Ok(items)
}

// ============================================================
// 4. 东方财富公司公告
// ============================================================

pub async fn fetch_announcements(
    stock_code: Option<&str>,
    page: u32,
    page_size: u32,
) -> Result<Vec<AnnouncementItem>> {
    let client = build_news_client("https://data.eastmoney.com/")?;

    let ann_type = if stock_code.is_some() {
        "SHA,SZA"
    } else {
        "SHA,SZA"
    };

    let mut url = format!(
        "https://np-anotice-stock.eastmoney.com/api/security/ann?sr=-1&page_size={}&page_index={}&ann_type={}&client=web&f_node=0&s_node=0",
        page_size, page, ann_type
    );

    if let Some(code) = stock_code {
        // 提取纯数字代码
        let pure_code = code
            .trim_start_matches("sh")
            .trim_start_matches("sz")
            .trim_start_matches("bj");
        url.push_str(&format!("&stock={}", pure_code));
    }

    let resp = client.get(&url).send().await?;
    let json: Value = resp.json().await?;

    let mut items = Vec::new();
    if let Some(list) = json["data"]["list"].as_array() {
        for item in list {
            let art_code = item["art_code"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let title = item["title"]
                .as_str()
                .or_else(|| item["title_ch"].as_str())
                .unwrap_or("")
                .to_string();
            let notice_date = item["notice_date"]
                .as_str()
                .unwrap_or("")
                .to_string();

            // 提取股票信息
            let (stock_code_str, stock_name_str) = if let Some(codes) = item["codes"].as_array() {
                if let Some(first) = codes.first() {
                    (
                        first["stock_code"].as_str().unwrap_or("").to_string(),
                        first["short_name"].as_str().unwrap_or("").to_string(),
                    )
                } else {
                    (String::new(), String::new())
                }
            } else {
                (String::new(), String::new())
            };

            // 提取分类
            let category = if let Some(cols) = item["columns"].as_array() {
                cols.first()
                    .and_then(|c| c["column_name"].as_str())
                    .unwrap_or("")
                    .to_string()
            } else {
                String::new()
            };

            let pdf_url = format!(
                "https://np-anotice-stock.eastmoney.com/api/security/ann?art_code={}",
                art_code
            );

            items.push(AnnouncementItem {
                id: art_code,
                title,
                stock_code: stock_code_str,
                stock_name: stock_name_str,
                notice_date,
                url: pdf_url,
                category,
            });
        }
    }

    Ok(items)
}

// ============================================================
// 5. 东方财富研报
// ============================================================

pub async fn fetch_reports(
    stock_code: Option<&str>,
    page: u32,
    page_size: u32,
) -> Result<Vec<ReportItem>> {
    let client = build_news_client("https://data.eastmoney.com/")?;
    let ts = chrono::Utc::now().timestamp_millis();

    let mut url = format!(
        "https://reportapi.eastmoney.com/report/list?industryCode=*&pageSize={}&industry=*&rating=&ratingChange=&beginTime=&endTime=&pageNo={}&fields=&qType=0&orgCode=&rcode=&_={}",
        page_size, page, ts
    );

    if let Some(code) = stock_code {
        let pure_code = code
            .trim_start_matches("sh")
            .trim_start_matches("sz")
            .trim_start_matches("bj");
        url.push_str(&format!("&code={}", pure_code));
    }

    let resp = client.get(&url).send().await?;
    let json: Value = resp.json().await?;

    let mut items = Vec::new();
    if let Some(data) = json["data"].as_array() {
        for item in data {
            let title = item["title"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let stock_code_str = item["stockCode"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let stock_name_str = item["stockName"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let org_name = item["orgSName"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let publish_date = item["publishDate"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let rating = item["emRatingName"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let researcher = item["researcher"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let industry = item["industryName"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let encode_url = item["encodeUrl"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let report_url = if !encode_url.is_empty() {
                format!("https://data.eastmoney.com/report/zw/{}.html", encode_url)
            } else {
                String::new()
            };

            items.push(ReportItem {
                title,
                stock_code: stock_code_str,
                stock_name: stock_name_str,
                org_name,
                publish_date,
                rating,
                researcher,
                industry,
                url: report_url,
            });
        }
    }

    Ok(items)
}

// ============================================================
// 6. 新浪财经滚动新闻
// ============================================================

pub async fn fetch_sina_roll_news(page: u32, count: u32) -> Result<Vec<NewsItem>> {
    let client = build_news_client("https://finance.sina.com.cn/")?;
    let ts = chrono::Utc::now().timestamp_millis() as f64 / 1000.0;
    // pageid=153, lid=2516 是财经频道
    let url = format!(
        "https://feed.mix.sina.com.cn/api/roll/get?pageid=153&lid=2516&k=&num={}&page={}&r={}",
        count, page, ts
    );

    let resp = client.get(&url).send().await?;
    let json: Value = resp.json().await?;

    let mut items = Vec::new();
    if let Some(data) = json["result"]["data"].as_array() {
        for item in data {
            let title = item["title"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let summary = item["summary"]
                .as_str()
                .or_else(|| item["intro"].as_str())
                .unwrap_or("")
                .to_string();
            let url_str = item["url"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let media = item["media_name"]
                .as_str()
                .or_else(|| item["author"].as_str())
                .unwrap_or("")
                .to_string();
            let ctime = item["ctime"]
                .as_str()
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);
            let docid = item["docid"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let publish_time = if ctime > 0 {
                chrono::DateTime::from_timestamp(ctime, 0)
                    .map(|dt| dt.with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap())
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            };

            if title.is_empty() {
                continue;
            }

            items.push(NewsItem {
                id: format!("sina_{}", docid),
                category: NewsCategory::SinaRoll,
                title,
                summary,
                source: if media.is_empty() { "新浪财经".to_string() } else { media },
                publish_time,
                url: url_str,
                importance: 0,
                related_stocks: Vec::new(),
            });
        }
    }

    Ok(items)
}
