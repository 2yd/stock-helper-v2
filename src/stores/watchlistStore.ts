import { create } from 'zustand';
import { safeInvoke as invoke, safeListen } from '../hooks/useTauri';
import {
  WatchlistStock,
  WatchlistQuote,
  StockTechnicalAnalysis,
  AIStreamEvent,
} from '../types';

interface MarketSnapshotRaw {
  code: string;
  name: string;
  price: number;
  change_pct: number;
  change_amount: number;
  volume: number;
  amount: number;
  amplitude: number;
  turnover_rate: number;
  pe_ttm: number;
  pb: number;
  volume_ratio: number;
  high: number;
  low: number;
  open: number;
  pre_close: number;
  total_market_cap: number;
  float_market_cap: number;
  pct_5d: number;
  pct_20d: number;
  roe: number;
  revenue_yoy: number;
  main_net_inflow: number;
  main_net_pct: number;
}

interface WatchlistStore {
  stocks: WatchlistStock[];
  quotes: WatchlistQuote[];
  quotesLoading: boolean;
  loading: boolean;
  selectedCode: string | null;
  analysis: StockTechnicalAnalysis | null;
  analysisLoading: boolean;
  analysisPeriod: 'day' | 'week';

  // AI diagnosis
  diagnosing: boolean;
  diagnoseContent: string;
  diagnoseDone: boolean;
  showDiagnosePanel: boolean;
  diagnoseToolCalls: { name: string; label: string; done: boolean }[];  // 工具调用进度列表

  // Auto refresh
  _refreshTimer: ReturnType<typeof setInterval> | null;

  // Actions
  loadStocks: () => Promise<void>;
  addStock: (code: string, name: string) => Promise<void>;
  removeStock: (code: string) => Promise<void>;
  reorderStocks: (codes: string[]) => Promise<void>;
  selectStock: (code: string | null) => void;
  loadAnalysis: (code: string, name: string, period?: 'day' | 'week') => Promise<void>;
  setPeriod: (period: 'day' | 'week') => void;
  loadQuotes: () => Promise<void>;
  startAutoRefresh: (intervalSecs: number) => void;
  stopAutoRefresh: () => void;

  // AI diagnosis
  startDiagnosis: (code: string, name: string) => Promise<() => void>;
  setShowDiagnosePanel: (show: boolean) => void;
  resetDiagnosis: () => void;
}

export const useWatchlistStore = create<WatchlistStore>((set, get) => ({
  stocks: [],
  quotes: [],
  quotesLoading: false,
  loading: false,
  selectedCode: null,
  analysis: null,
  analysisLoading: false,
  analysisPeriod: 'day',

  diagnosing: false,
  diagnoseContent: '',
  diagnoseDone: false,
  showDiagnosePanel: false,
  diagnoseToolCalls: [],

  _refreshTimer: null,

  loadStocks: async () => {
    set({ loading: true });
    const stocks = await invoke<WatchlistStock[]>('get_watchlist_stocks');
    set({ stocks, loading: false });
    // Also load quotes after loading stock list
    if (stocks.length > 0) {
      get().loadQuotes();
    }
  },

  loadQuotes: async () => {
    const { stocks } = get();
    if (stocks.length === 0) {
      set({ quotes: [] });
      return;
    }
    set({ quotesLoading: true });
    try {
      const codes = stocks.map((s) => s.code);
      const snapshots = await invoke<MarketSnapshotRaw[]>('get_watchlist_enriched', { codes });
      const quotes: WatchlistQuote[] = (snapshots || []).map((s) => ({
        code: s.code,
        name: s.name,
        price: s.price,
        pre_close: s.pre_close,
        open: s.open,
        high: s.high,
        low: s.low,
        volume: s.volume,
        amount: s.amount,
        change_pct: s.change_pct,
        change_price: s.change_amount,
        pe_ttm: s.pe_ttm,
        pb: s.pb,
        roe: s.roe,
        total_market_cap: s.total_market_cap,
        float_market_cap: s.float_market_cap,
        turnover_rate: s.turnover_rate,
        volume_ratio: s.volume_ratio,
        main_net_inflow: s.main_net_inflow,
        pct_5d: s.pct_5d,
        pct_20d: s.pct_20d,
        revenue_yoy: s.revenue_yoy,
        amplitude: s.amplitude,
        date: '',
        time: '',
      }));
      set({ quotes, quotesLoading: false });
    } catch (e) {
      console.error('Failed to load quotes:', e);
      set({ quotesLoading: false });
    }
  },

  startAutoRefresh: (intervalSecs: number) => {
    get().stopAutoRefresh();
    const timer = setInterval(() => {
      get().loadQuotes();
    }, intervalSecs * 1000);
    set({ _refreshTimer: timer });
  },

  stopAutoRefresh: () => {
    const { _refreshTimer } = get();
    if (_refreshTimer) {
      clearInterval(_refreshTimer);
      set({ _refreshTimer: null });
    }
  },

  addStock: async (code: string, name: string) => {
    await invoke('add_watchlist_stock', { code, name });
    await get().loadStocks();
  },

  removeStock: async (code: string) => {
    await invoke('remove_watchlist_stock', { code });
    const { selectedCode } = get();
    if (selectedCode === code) {
      set({ selectedCode: null, analysis: null });
    }
    await get().loadStocks();
  },

  reorderStocks: async (codes: string[]) => {
    await invoke('reorder_watchlist', { codes });
    await get().loadStocks();
  },

  selectStock: (code: string | null) => {
    set({ selectedCode: code, analysis: null });
    if (code) {
      const stock = get().stocks.find((s) => s.code === code);
      if (stock) {
        get().loadAnalysis(code, stock.name, get().analysisPeriod);
      }
    }
  },

  loadAnalysis: async (code: string, name: string, period?: 'day' | 'week') => {
    const p = period || get().analysisPeriod;
    set({ analysisLoading: true, analysisPeriod: p });
    const analysis = await invoke<StockTechnicalAnalysis>(
      'get_stock_technical_analysis',
      { code, name, period: p }
    );
    set({ analysis, analysisLoading: false });
  },

  setPeriod: (period: 'day' | 'week') => {
    set({ analysisPeriod: period });
    const { selectedCode, analysis } = get();
    if (selectedCode && analysis) {
      get().loadAnalysis(selectedCode, analysis.name, period);
    }
  },

  startDiagnosis: async (code: string, name: string) => {
    set({
      diagnosing: true,
      diagnoseContent: '',
      diagnoseDone: false,
      showDiagnosePanel: true,
      diagnoseToolCalls: [],
    });

    const technicalSummary = get().analysis?.summary || '';

    const unlisten = await safeListen<AIStreamEvent>(
      `ai-diagnose-${code}`,
      (event) => {
        const { event_type, content, done, tool_name } = event.payload;
        if (event_type === 'tool_call' && tool_name) {
          set((s) => ({
            diagnoseToolCalls: [...s.diagnoseToolCalls, { name: tool_name, label: content || tool_name, done: false }],
          }));
        }
        if (event_type === 'tool_result' && tool_name) {
          set((s) => ({
            diagnoseToolCalls: s.diagnoseToolCalls.map((t) =>
              t.name === tool_name && !t.done
                ? { ...t, label: content || t.label, done: true }
                : t
            ),
          }));
        }
        if (event_type === 'content' && content) {
          set((s) => ({ diagnoseContent: s.diagnoseContent + content }));
        }
        if (event_type === 'done' || done) {
          set({ diagnosing: false, diagnoseDone: true });
        }
        if (event_type === 'error') {
          set({
            diagnosing: false,
            diagnoseDone: true,
            diagnoseContent: get().diagnoseContent + '\n\n[分析出错: ' + (content || '未知错误') + ']',
          });
        }
      }
    );

    invoke('ai_diagnose_stock', {
      code,
      name,
      technicalSummary,
    }).catch((e: unknown) => {
      console.error('AI diagnosis failed:', e);
      set({ diagnosing: false, diagnoseDone: true });
    });

    return unlisten;
  },

  setShowDiagnosePanel: (show: boolean) => {
    set({ showDiagnosePanel: show });
  },

  resetDiagnosis: () => {
    set({
      diagnosing: false,
      diagnoseContent: '',
      diagnoseDone: false,
      diagnoseToolCalls: [],
    });
  },
}));
