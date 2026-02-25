import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen as tauriListen, type EventCallback, type UnlistenFn } from '@tauri-apps/api/event';

export const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;

export async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) {
    console.warn(`[mock] invoke("${cmd}") — not in Tauri runtime`);
    return getMockData(cmd) as T;
  }
  return tauriInvoke<T>(cmd, args);
}

export async function safeListen<T>(event: string, handler: EventCallback<T>): Promise<UnlistenFn> {
  if (!isTauri) {
    console.warn(`[mock] listen("${event}") — not in Tauri runtime`);
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
        strategies: [
          {
            id: 'default',
            name: '多因子综合选股',
            description: '基于价值、质量、动量、资金、风险五大维度的量化选股策略',
            watch_codes: [],
            weights: {
              value: 15,
              quality: 15,
              momentum: 30,
              capital: 25,
              risk: 15,
            },
            filters: {
              exclude_st: true,
              exclude_new_stock_days: 60,
              min_market_cap: 30,
              max_market_cap: 0,
              min_price: 3,
              min_amount: 5000,
              pe_max: 100,
              pe_min: 0,
              pb_max: 20,
              roe_min: 5,
            },
            enabled: true,
            top_n: 50,
          },
        ],
        active_strategy_id: 'default',
        token_usage_today: 0,
        qgqp_b_id: '',
      };
    case 'get_market_status':
      return '休市';
    case 'get_today_token_usage':
      return 0;
    case 'scan_market':
    case 'refresh_strategy':
      return [];
    case 'get_market_stock_count':
      return 0;
    case 'get_hot_strategies':
      return [];
    case 'smart_search_stock':
      return { code: -1, message: '非 Tauri 环境' };
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
    default:
      return null;
  }
}
