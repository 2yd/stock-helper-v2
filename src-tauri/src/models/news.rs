use serde::{Deserialize, Serialize};

/// 新闻/快讯类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewsCategory {
    /// 财联社电报快讯
    ClsTelegraph,
    /// 东方财富财经要闻
    EastmoneyNews,
    /// 东方财富个股新闻
    StockNews,
    /// 东方财富公司公告
    Announcement,
    /// 东方财富研报
    Report,
    /// 新浪财经滚动新闻
    SinaRoll,
    /// 新浪7x24财经直播快讯
    Sina7x24,
    /// 华尔街见闻快讯
    WallStreetCn,
}

/// 统一的新闻/资讯条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    /// 唯一标识
    pub id: String,
    /// 分类
    pub category: NewsCategory,
    /// 标题
    pub title: String,
    /// 内容摘要
    pub summary: String,
    /// 来源媒体
    pub source: String,
    /// 发布时间 (ISO 格式或时间戳字符串)
    pub publish_time: String,
    /// 原文链接
    pub url: String,
    /// 重要性等级 (0=普通, 1=重要, 2=非常重要)
    pub importance: u8,
    /// 关联股票代码列表
    pub related_stocks: Vec<String>,
}

/// 公司公告条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnouncementItem {
    pub id: String,
    pub title: String,
    pub stock_code: String,
    pub stock_name: String,
    pub notice_date: String,
    pub url: String,
    /// 公告分类 (如: 业绩预告, 股东大会, 增减持等)
    pub category: String,
}

/// 研报条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportItem {
    pub title: String,
    pub stock_code: String,
    pub stock_name: String,
    pub org_name: String,
    pub publish_date: String,
    pub rating: String,
    pub researcher: String,
    pub industry: String,
    pub url: String,
}
