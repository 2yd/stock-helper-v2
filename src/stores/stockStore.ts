import { create } from 'zustand';
import { safeInvoke as invoke } from '../hooks/useTauri';
import { StrategyResultRow } from '../types';

interface StockStore {
  results: StrategyResultRow[];
  loading: boolean;
  lastRefreshTime: string | null;
  marketStatus: string;
  tokenUsageToday: number;
  autoRefreshTimer: ReturnType<typeof setInterval> | null;
  scanTotal: number;  // 扫描的全市场股票总数

  scanMarket: (strategyId?: string) => Promise<void>;
  refreshStrategy: (strategyId?: string) => Promise<void>;
  generateInstructions: () => Promise<void>;
  fetchMarketStatus: () => Promise<void>;
  fetchTokenUsage: () => Promise<void>;
  startAutoRefresh: (intervalSecs: number) => void;
  stopAutoRefresh: () => void;
}

export const useStockStore = create<StockStore>((set, get) => ({
  results: [],
  loading: false,
  lastRefreshTime: null,
  marketStatus: '未连接',
  tokenUsageToday: 0,
  autoRefreshTimer: null,
  scanTotal: 0,

  scanMarket: async (strategyId = 'default') => {
    set({ loading: true });
    try {
      const results = await invoke<StrategyResultRow[]>('scan_market', {
        strategyId,
      });
      const now = new Date().toLocaleTimeString('zh-CN', { hour12: false });
      set({ results, lastRefreshTime: now, loading: false, scanTotal: results.length });
    } catch (e) {
      console.error('Failed to scan market:', e);
      set({ loading: false });
    }
  },

  // 兼容旧调用
  refreshStrategy: async (strategyId = 'default') => {
    return get().scanMarket(strategyId);
  },

  generateInstructions: async () => {
    const { results } = get();
    if (results.length === 0) return;
    try {
      const updated = await invoke<StrategyResultRow[]>('generate_ai_instructions', {
        results,
      });
      set({ results: updated });
      get().fetchTokenUsage();
    } catch (e) {
      console.error('Failed to generate AI instructions:', e);
    }
  },

  fetchMarketStatus: async () => {
    try {
      const status = await invoke<string>('get_market_status');
      set({ marketStatus: status });
    } catch (e) {
      console.error('Failed to fetch market status:', e);
    }
  },

  fetchTokenUsage: async () => {
    try {
      const usage = await invoke<number>('get_today_token_usage');
      set({ tokenUsageToday: usage });
    } catch (e) {
      console.error('Failed to fetch token usage:', e);
    }
  },

  startAutoRefresh: (intervalSecs: number) => {
    const { autoRefreshTimer } = get();
    if (autoRefreshTimer) clearInterval(autoRefreshTimer);

    const timer = setInterval(() => {
      get().scanMarket();
    }, intervalSecs * 1000);

    set({ autoRefreshTimer: timer });
  },

  stopAutoRefresh: () => {
    const { autoRefreshTimer } = get();
    if (autoRefreshTimer) {
      clearInterval(autoRefreshTimer);
      set({ autoRefreshTimer: null });
    }
  },
}));
