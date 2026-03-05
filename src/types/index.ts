export interface StockLabel {
  text: string;
  color: string;
  icon: string | null;
}

export interface AIConfig {
  id: string;
  name: string;
  base_url: string;
  api_key: string;
  model_name: string;
  max_tokens: number;
  temperature: number;
  /** 选股 temperature（发散场景，建议 0.5~0.8） */
  pick_temperature: number;
  timeout_secs: number;
  enabled: boolean;
}

export interface AgentPrompt {
  id: string;
  name: string;
  strategy_prompt: string;
  is_builtin: boolean;
  created_at: string;
  updated_at: string;
  description?: string;
}

export interface AppSettings {
  refresh_interval_secs: number;
  auto_refresh: boolean;
  ai_instruction_enabled: boolean;
  data_source_primary: 'sina' | 'tencent';
  ai_configs: AIConfig[];
  active_ai_config_id: string | null;
  token_usage_today: number;
  qgqp_b_id: string;
  max_pick_tool_rounds: number;
  max_pick_token_budget: number;
  agent_prompts: AgentPrompt[];
  active_pick_prompt_id: string | null;
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

/** 败因分析入参 — 单只亏损股信息 */
export interface LossStock {
  code: string;
  name: string;
  added_price: number;
  current_price: number;
  change_pct: number;
  reason: string;
  sector: string;
}

// ====== News / Info Types ======

export type NewsCategory =
  | 'ClsTelegraph'
  | 'EastmoneyNews'
  | 'StockNews'
  | 'Announcement'
  | 'Report'
  | 'SinaRoll'
  | 'Sina7x24'
  | 'WallStreetCn';

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
  fund_flow?: string;    // 资金流向状态，如"主力净流入2.3亿"
  valuation?: string;    // 估值水平，如"PE 15.2 低估"
}

export interface AIPickResult {
  recommendations: AIPickRecommendation[];
  analysis_summary: string;
  timestamp: string;
}
