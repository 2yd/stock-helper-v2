import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen as tauriListen, type EventCallback, type UnlistenFn } from '@tauri-apps/api/event';
import logger from '../utils/logger';

export const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;

export async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) {
    logger.warn(`[mock] invoke("${cmd}") — not in Tauri runtime`);
    return getMockData(cmd) as T;
  }
  return tauriInvoke<T>(cmd, args);
}

export async function safeListen<T>(event: string, handler: EventCallback<T>): Promise<UnlistenFn> {
  if (!isTauri) {
    logger.warn(`[mock] listen("${event}") — not in Tauri runtime`);
    return () => {};
  }
  return tauriListen<T>(event, handler);
}

function getMockData(cmd: string): unknown {
  switch (cmd) {
    case 'get_settings':
      return {
        refresh_interval_secs: 30,
        auto_refresh: false,
        ai_instruction_enabled: false,
        data_source_primary: 'sina',
        ai_configs: [],
        active_ai_config_id: null,
        token_usage_today: 0,
        qgqp_b_id: '',
        max_pick_tool_rounds: 10,
        max_pick_token_budget: 100000,
        agent_prompts: [],
        active_pick_prompt_id: null,
      };
    case 'search_stocks':
      return [];
    // Watchlist commands
    case 'get_watchlist_stocks':
      return [];
    case 'add_watchlist_stock':
    case 'remove_watchlist_stock':
    case 'reorder_watchlist':
      return null;
    case 'get_stock_technical_analysis':
      return {
        code: '000001',
        name: '平安银行',
        kline_data: [],
        indicators: {
          dates: [], ma5: [], ma10: [], ma20: [], ma60: [],
          ema12: [], ema26: [],
          macd_dif: [], macd_dea: [], macd_hist: [],
          kdj_k: [], kdj_d: [], kdj_j: [],
          rsi6: [], rsi12: [], rsi24: [],
          boll_upper: [], boll_middle: [], boll_lower: [],
        },
        signals: [],
        ma_alignment: 'tangled',
        volume_price_relation: '正常',
        summary: '非 Tauri 环境，无法获取技术分析数据',
      };
    case 'ai_diagnose_stock':
      return null;
    // Backtest commands
    case 'run_backtest':
      return {
        config: { codes: [], start_date: '', end_date: '', initial_capital: 1000000, commission_rate: 0.0003, slippage: 0.001, buy_threshold: 70, sell_threshold: 40, stop_loss: 0.08, take_profit: 0.20, factor_weights: { value: 15, quality: 15, momentum: 30, capital: 25, risk: 15 }, max_position_pct: 0.25 },
        performance: { total_return: 0, annual_return: 0, max_drawdown: 0, sharpe_ratio: 0, win_rate: 0, profit_loss_ratio: 0, total_trades: 0, winning_trades: 0, losing_trades: 0, max_consecutive_wins: 0, max_consecutive_losses: 0, avg_holding_days: 0, benchmark_return: 0, alpha: 0 },
        equity_curve: [],
        trades: [],
      };
    case 'fetch_history_kline':
      return [];
    // News commands
    case 'fetch_cls_telegraph':
    case 'fetch_eastmoney_news':
    case 'fetch_stock_news':
    case 'fetch_sina_news':
      return [];
    case 'fetch_announcements':
      return [];
    case 'fetch_reports':
      return [];
    // AI Pick commands
    case 'ai_pick_stocks':
      return null;
    case 'get_cached_picks':
      return null;
    case 'stop_ai_pick':
      return null;
    case 'analyze_loss_reasons':
      return null;
    case 'test_ai_config':
      return '模型连接正常（Mock 环境）';
    case 'get_market_overview':
      return {
        market_status: '已收盘',
        indexes: [
          { name: '上证指数', code: 'sh000001', price: 3402.86, change_pct: 0.83, change_amount: 27.99, amount: 512300000000, open: 3380.0, high: 3415.0, low: 3375.0, pre_close: 3374.87 },
          { name: '深证成指', code: 'sz399001', price: 10352.00, change_pct: 0.62, change_amount: 63.92, amount: 623400000000, open: 10300.0, high: 10400.0, low: 10280.0, pre_close: 10288.08 },
          { name: '创业板指', code: 'sz399006', price: 2082.53, change_pct: -0.25, change_amount: -5.18, amount: 198500000000, open: 2090.0, high: 2095.0, low: 2075.0, pre_close: 2087.71 },
        ],
        market_stats: { rise_count: 2856, fall_count: 2103, flat_count: 341 },
        sentiment: { score: 58.5, level: '中性', money_effect: 53.8 },
        sector_top: [
          { name: '船舶制造', change_pct: 4.52, lead_stock: '中国船舶' },
          { name: '航天航空', change_pct: 3.18, lead_stock: '航发动力' },
          { name: '半导体', change_pct: 2.76, lead_stock: '北方华创' },
          { name: '光伏设备', change_pct: 2.31, lead_stock: '隆基绿能' },
          { name: '新能源车', change_pct: 1.95, lead_stock: '比亚迪' },
        ],
        sector_bottom: [
          { name: '房地产', change_pct: -2.41, lead_stock: '万科A' },
          { name: '酿酒行业', change_pct: -1.87, lead_stock: '贵州茅台' },
          { name: '保险', change_pct: -1.52, lead_stock: '中国平安' },
          { name: '银行', change_pct: -1.08, lead_stock: '招商银行' },
          { name: '医药商业', change_pct: -0.93, lead_stock: '国药股份' },
        ],
        global_indexes: [
          { name: '道琼斯', code: 'DJIA', price: '43221.55', change_pct: '0.32%', region: 'america' },
          { name: '纳斯达克', code: 'IXIC', price: '18925.74', change_pct: '-0.18%', region: 'america' },
          { name: '恒生指数', code: 'HSI', price: '22652.14', change_pct: '1.24%', region: 'asia' },
          { name: '日经225', code: 'N225', price: '38451.46', change_pct: '-0.45%', region: 'asia' },
          { name: '英国富时100', code: 'FTSE', price: '8421.70', change_pct: '0.56%', region: 'europe' },
          { name: '德国DAX30', code: 'GDAXI', price: '21108.42', change_pct: '0.78%', region: 'europe' },
        ],
        total_amount: 1334200000000,
        volume_compare: { today_amount: 1334200000000, yesterday_amount: 1198700000000, diff: 135500000000, ratio: 1.113 },
        update_time: '15:01:23',
      };
    case 'generate_market_comment':
      return '今日市场震荡走高，上证指数收涨0.83%站上3400点。两市成交额1.33万亿，较昨日放量11.3%，赚钱效应尚可。船舶制造、航天航空板块领涨，市场情绪偏中性。';
    default:
      return null;
  }
}
