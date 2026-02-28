export interface StrategyResultRow {
  code: string;
  name: string;
  price: number;
  change_pct: number;
  pe_ttm: number;
  pb: number;
  roe: number;
  revenue_yoy: number;
  profit_yoy: number;
  total_market_cap: number;  // 亿元
  float_market_cap: number;  // 亿元
  turnover_rate: number;     // %
  volume_ratio: number;
  amount: number;            // 万元
  main_net_inflow: number;   // 万元
  main_net_pct: number;      // %
  pct_5d: number;            // %
  pct_20d: number;           // %
  pct_60d: number;           // %
  score: number;             // 0-100
  score_detail: FactorScoreDetail;
  sentiment_score: number;   // 消息面得分 0-1
  news_heat: number;         // 消息热度
  matched_themes: string[];  // 匹配的主题
  labels: StockLabel[];
  instruction: AIInstruction | null;
}

export interface FactorScoreDetail {
  value_score: number;    // 0-1
  quality_score: number;
  momentum_score: number;
  capital_score: number;
  risk_score: number;
  sentiment_score: number;
}

export interface StockLabel {
  text: string;
  color: string;
  icon: string | null;
}

export interface AIInstruction {
  action: 'buy' | 'watch' | 'eliminate';
  label: string;
  reason: string;
}

export interface AIConfig {
  id: string;
  name: string;
  base_url: string;
  api_key: string;
  model_name: string;
  max_tokens: number;
  temperature: number;
  timeout_secs: number;
  enabled: boolean;
}

export interface FactorWeights {
  value: number;
  quality: number;
  momentum: number;
  capital: number;
  risk: number;
  sentiment: number;
}

export interface StockFilters {
  exclude_st: boolean;
  exclude_new_stock_days: number;
  min_market_cap: number;
  max_market_cap: number;
  min_price: number;
  min_amount: number;
  pe_max: number;
  pe_min: number;
  pb_max: number;
  roe_min: number;
}

export interface StrategyConfig {
  id: string;
  name: string;
  description: string;
  weights: FactorWeights;
  filters: StockFilters;
  enabled: boolean;
  top_n: number;
  watch_codes: string[];
}

export interface AppSettings {
  refresh_interval_secs: number;
  auto_refresh: boolean;
  ai_instruction_enabled: boolean;
  data_source_primary: 'sina' | 'tencent';
  ai_configs: AIConfig[];
  active_ai_config_id: string | null;
  strategies: StrategyConfig[];
  active_strategy_id: string;
  token_usage_today: number;
  qgqp_b_id: string;
}

export interface AIAnalysisResult {
  id: string;
  code: string;
  name: string;
  model_name: string;
  question: string;
  content: string;
  created_at: string;
}

export interface AIStreamEvent {
  event_type: string;  // "content" | "tool_call" | "tool_result" | "done" | "error"
  content: string | null;
  done: boolean;
  usage: { prompt_tokens: number; completion_tokens: number; total_tokens: number } | null;
  tool_name?: string | null;
}

// Smart Stock types
export interface SmartStockColumn {
  key: string;
  title: string;
  unit?: string;
  date_msg?: string;
  hidden_need?: boolean;
  children?: SmartStockColumn[];
}

export interface HotStrategyItem {
  rank: number;
  question: string;
  chg?: number;
}

// ====== Watchlist Types ======

export interface WatchlistStock {
  code: string;
  name: string;
  sort_order: number;
  group_name: string;
  created_at: string;
}

/** 自选股实时行情 (来自 MarketStockSnapshot，东财 ulist API) */
export interface WatchlistQuote {
  code: string;
  name: string;
  price: number;
  pre_close: number;
  open: number;
  high: number;
  low: number;
  volume: number;       // 手
  amount: number;       // 元
  change_pct: number;   // 涨跌幅 %
  change_price: number; // 涨跌额
  pe_ttm: number;
  pb: number;
  roe: number;
  total_market_cap: number;  // 元
  float_market_cap: number;  // 元
  turnover_rate: number;     // %
  volume_ratio: number;
  main_net_inflow: number;   // 元
  pct_5d: number;            // %
  pct_20d: number;           // %
  revenue_yoy: number;       // %
  amplitude: number;         // %
  date: string;
  time: string;
}

export interface KlineItem {
  date: string;
  open: number;
  close: number;
  high: number;
  low: number;
  volume: number;
  amount: number;
  change_pct: number;
  turnover_rate: number;
}

export interface TechnicalIndicators {
  dates: string[];
  ma5: (number | null)[];
  ma10: (number | null)[];
  ma20: (number | null)[];
  ma60: (number | null)[];
  ema12: (number | null)[];
  ema26: (number | null)[];
  macd_dif: (number | null)[];
  macd_dea: (number | null)[];
  macd_hist: (number | null)[];
  kdj_k: (number | null)[];
  kdj_d: (number | null)[];
  kdj_j: (number | null)[];
  rsi6: (number | null)[];
  rsi12: (number | null)[];
  rsi24: (number | null)[];
  boll_upper: (number | null)[];
  boll_middle: (number | null)[];
  boll_lower: (number | null)[];
}

export interface TechnicalSignal {
  signal_type: string;
  direction: 'bullish' | 'bearish' | 'neutral';
  description: string;
  strength: number;
  date: string;
}

export interface StockTechnicalAnalysis {
  code: string;
  name: string;
  kline_data: KlineItem[];
  indicators: TechnicalIndicators;
  signals: TechnicalSignal[];
  ma_alignment: 'bullish' | 'bearish' | 'tangled';
  volume_price_relation: string;
  summary: string;
}

// ====== AI Pick Tracking Types ======

export interface AIPickTracking {
  code: string;
  name: string;
  added_date: string;
  added_price: number;
  rating: string;
  reason: string;
  sector: string;
  created_at: string;
}

// ====== News / Info Types ======

export type NewsCategory =
  | 'ClsTelegraph'
  | 'EastmoneyNews'
  | 'StockNews'
  | 'Announcement'
  | 'Report'
  | 'SinaRoll';

export interface NewsItem {
  id: string;
  category: NewsCategory;
  title: string;
  summary: string;
  source: string;
  publish_time: string;
  url: string;
  importance: number;
  related_stocks: string[];
}

export interface AnnouncementItem {
  id: string;
  title: string;
  stock_code: string;
  stock_name: string;
  notice_date: string;
  url: string;
  category: string;
}

export interface ReportItem {
  title: string;
  stock_code: string;
  stock_name: string;
  org_name: string;
  publish_date: string;
  rating: string;
  researcher: string;
  industry: string;
  url: string;
}

// ========== AI 自主选股 ==========

export interface AIPickRecommendation {
  code: string;
  name: string;
  price?: number;
  change_pct?: number;
  reason: string;
  rating: 'strong_buy' | 'buy' | 'watch';
  sector?: string;
  highlights?: string[];
}

export interface AIPickResult {
  recommendations: AIPickRecommendation[];
  analysis_summary: string;
  timestamp: string;
}
