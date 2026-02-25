use crate::models::news::{AnnouncementItem, NewsItem, ReportItem};
use crate::services::news_service;

/// 获取财联社电报快讯
#[tauri::command]
pub async fn fetch_cls_telegraph(count: Option<u32>) -> Result<Vec<NewsItem>, String> {
    let count = count.unwrap_or(30);
    news_service::fetch_cls_telegraph(count)
        .await
        .map_err(|e| format!("获取财联社快讯失败: {}", e))
}

/// 获取东方财富财经要闻
#[tauri::command]
pub async fn fetch_eastmoney_news(
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<Vec<NewsItem>, String> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    news_service::fetch_eastmoney_news(page, page_size)
        .await
        .map_err(|e| format!("获取东方财富新闻失败: {}", e))
}

/// 获取个股相关新闻
#[tauri::command]
pub async fn fetch_stock_news(
    keyword: String,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<Vec<NewsItem>, String> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    news_service::fetch_stock_news(&keyword, page, page_size)
        .await
        .map_err(|e| format!("获取个股新闻失败: {}", e))
}

/// 获取公司公告
#[tauri::command]
pub async fn fetch_announcements(
    stock_code: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<Vec<AnnouncementItem>, String> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    news_service::fetch_announcements(stock_code.as_deref(), page, page_size)
        .await
        .map_err(|e| format!("获取公司公告失败: {}", e))
}

/// 获取研报
#[tauri::command]
pub async fn fetch_reports(
    stock_code: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<Vec<ReportItem>, String> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    news_service::fetch_reports(stock_code.as_deref(), page, page_size)
        .await
        .map_err(|e| format!("获取研报失败: {}", e))
}

/// 获取新浪财经滚动新闻
#[tauri::command]
pub async fn fetch_sina_news(
    page: Option<u32>,
    count: Option<u32>,
) -> Result<Vec<NewsItem>, String> {
    let page = page.unwrap_or(1);
    let count = count.unwrap_or(20);
    news_service::fetch_sina_roll_news(page, count)
        .await
        .map_err(|e| format!("获取新浪新闻失败: {}", e))
}
