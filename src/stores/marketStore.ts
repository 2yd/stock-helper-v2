import { create } from 'zustand';
import { safeInvoke as invoke } from '../hooks/useTauri';
import { MarketOverview, KlineItem } from '../types';
import logger from '../utils/logger';

interface MarketStore {
  overview: MarketOverview | null;
  aiComment: string | null;
  aiCommentLoading: boolean;
  loading: boolean;
  error: string | null;
  indexKlines: Record<string, KlineItem[]>;
  refreshTimer: ReturnType<typeof setInterval> | null;

  fetchOverview: () => Promise<void>;
  generateAiComment: () => Promise<void>;
  fetchIndexKlines: () => Promise<void>;
  startAutoRefresh: () => void;
  stopAutoRefresh: () => void;
}

export const useMarketStore = create<MarketStore>((set, get) => ({
  overview: null,
  aiComment: null,
  aiCommentLoading: false,
  loading: false,
  error: null,
  indexKlines: {},
  refreshTimer: null,

  fetchOverview: async () => {
    set({ loading: true, error: null });
    const result = await invoke<MarketOverview>('get_market_overview').catch((e: unknown) => {
      const msg = e instanceof Error ? e.message : String(e);
      logger.error(`[marketStore] fetchOverview failed: ${msg}`);
      set({ error: msg, loading: false });
      return null;
    });
    if (result) {
      set({ overview: result, loading: false });
    }
  },

  generateAiComment: async () => {
    const overview = get().overview;
    if (!overview) return;

    set({ aiCommentLoading: true });
    const overviewJson = JSON.stringify(overview);
    const result = await invoke<string>('generate_market_comment', { overviewJson }).catch((e: unknown) => {
      const msg = e instanceof Error ? e.message : String(e);
      logger.warn(`[marketStore] AI comment failed: ${msg}`);
      return null;
    });
    set({ aiComment: result, aiCommentLoading: false });
  },

  fetchIndexKlines: async () => {
    const codes = ['sh000001', 'sz399001', 'sz399006'];
    const promises = codes.map(code =>
      invoke<KlineItem[]>('get_kline_data', { code, scale: '1', days: 240 }).catch(() => [] as KlineItem[])
    );
    const results = await Promise.all(promises);
    const klines: Record<string, KlineItem[]> = {};
    codes.forEach((code, i) => { klines[code] = results[i]; });
    set({ indexKlines: klines });
  },

  startAutoRefresh: () => {
    const existing = get().refreshTimer;
    if (existing) clearInterval(existing);

    const timer = setInterval(() => {
      get().fetchOverview();
    }, 30000);
    set({ refreshTimer: timer });
  },

  stopAutoRefresh: () => {
    const timer = get().refreshTimer;
    if (timer) {
      clearInterval(timer);
      set({ refreshTimer: null });
    }
  },
}));
