import { create } from 'zustand';
import { safeInvoke as invoke, safeListen } from '../hooks/useTauri';
import { AIPickTracking, AIStreamEvent, LossStock, WatchlistQuote } from '../types';
import logger from '../utils/logger';

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

export interface TrackingStockWithQuote extends AIPickTracking {
  current_price?: number;
  change_from_added?: number;  // (current - added) / added * 100
}

export interface DateGroup {
  date: string;
  stocks: TrackingStockWithQuote[];
  winCount: number;
  loseCount: number;
  totalCount: number;
  winRate: number;
  avgReturn: number;
}

interface ToolCallStatus {
  name: string;
  label: string;
  done: boolean;
  summary?: string;
}

interface ThinkingStep {
  content: string;
  timestamp: number;
}

interface TrackingStore {
  trackingStocks: AIPickTracking[];
  quotes: Map<string, WatchlistQuote>;
  loading: boolean;
  quotesLoading: boolean;
  _refreshTimer: ReturnType<typeof setInterval> | null;

  // Loss analysis state
  lossAnalyzing: boolean;
  lossAnalysisContent: string;
  lossAnalysisDate: string | null;
  lossToolCalls: ToolCallStatus[];
  lossThinkingSteps: ThinkingStep[];
  lossError: string | null;
  lossDone: boolean;
  _lossUnlisten: (() => void) | null;

  loadTrackingStocks: () => Promise<void>;
  addTracking: (code: string, name: string, addedPrice: number, rating: string, reason: string, sector: string) => Promise<void>;
  removeTracking: (code: string, addedDate: string) => Promise<void>;
  clearDate: (date: string) => Promise<void>;
  loadQuotes: () => Promise<void>;
  startAutoRefresh: (intervalSecs: number) => void;
  stopAutoRefresh: () => void;

  // Loss analysis actions
  startLossAnalysis: (date: string, lossStocks: LossStock[]) => Promise<void>;
  closeLossAnalysis: () => void;

  // Computed
  getDateGroups: () => DateGroup[];
}

export const useTrackingStore = create<TrackingStore>((set, get) => ({
  trackingStocks: [],
  quotes: new Map(),
  loading: false,
  quotesLoading: false,
  _refreshTimer: null,

  // Loss analysis initial state
  lossAnalyzing: false,
  lossAnalysisContent: '',
  lossAnalysisDate: null,
  lossToolCalls: [],
  lossThinkingSteps: [],
  lossError: null,
  lossDone: false,
  _lossUnlisten: null,

  loadTrackingStocks: async () => {
    set({ loading: true });
    const stocks = await invoke<AIPickTracking[]>('get_tracking_stocks');
    set({ trackingStocks: stocks || [], loading: false });
    if (stocks && stocks.length > 0) {
      get().loadQuotes();
    }
  },

  addTracking: async (code, name, addedPrice, rating, reason, sector) => {
    await invoke('add_tracking_stock', { code, name, addedPrice, rating, reason, sector });
    await get().loadTrackingStocks();
  },

  removeTracking: async (code, addedDate) => {
    await invoke('remove_tracking_stock', { code, addedDate });
    await get().loadTrackingStocks();
  },

  clearDate: async (date) => {
    await invoke('clear_tracking_by_date', { date });
    await get().loadTrackingStocks();
  },

  loadQuotes: async () => {
    const { trackingStocks } = get();
    if (trackingStocks.length === 0) return;

    const uniqueCodes = [...new Set(trackingStocks.map((s) => s.code))];
    set({ quotesLoading: true });
    try {
      const snapshots = await invoke<MarketSnapshotRaw[]>('get_watchlist_enriched', { codes: uniqueCodes });
      const quoteMap = new Map<string, WatchlistQuote>();
      (snapshots || []).forEach((s) => {
        quoteMap.set(s.code, {
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
        });
      });
      set({ quotes: quoteMap, quotesLoading: false });
    } catch (e) {
      logger.error(`Failed to load tracking quotes: ${e}`);
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

  startLossAnalysis: async (date: string, lossStocks: LossStock[]) => {
    // 先清理旧监听和状态，防止日期切换时状态污染
    const { _lossUnlisten } = get();
    if (_lossUnlisten) {
      _lossUnlisten();
    }

    set({
      lossAnalyzing: true,
      lossAnalysisContent: '',
      lossAnalysisDate: date,
      lossToolCalls: [],
      lossThinkingSteps: [],
      lossError: null,
      lossDone: false,
      _lossUnlisten: null,
    });

    const TOOL_LABELS: Record<string, string> = {
      get_market_news: '获取市场新闻',
      get_economic_data: '宏观经济数据',
      get_global_indexes: '全球指数',
      get_financial_calendar: '财经日历',
      search_stocks_by_condition: 'NLP智能选股',
      search_concept_boards: 'NLP板块搜索',
      batch_get_stock_quotes: '批量查看行情',
      get_stock_quote: '查看个股行情',
      get_fund_flow: '查看资金流向',
      get_kline_data: '获取K线数据',
      get_technical_indicators: '获取技术指标',
      search_stock_news: '个股新闻搜索',
      get_stock_notices: '公司公告',
      get_industry_report: '研报摘要',
    };

    const eventName = `ai-loss-analysis-${date}`;
    const unlisten = await safeListen<AIStreamEvent>(eventName, (event) => {
      const data = event.payload;
      const state = get();

      // 忽略非当前日期的事件
      if (state.lossAnalysisDate !== date) return;

      if (data.event_type === 'thinking') {
        set({
          lossThinkingSteps: [
            ...state.lossThinkingSteps,
            { content: data.content || '', timestamp: Date.now() },
          ],
        });
      } else if (data.event_type === 'content') {
        set({ lossAnalysisContent: state.lossAnalysisContent + (data.content || '') });
      } else if (data.event_type === 'tool_call') {
        const toolName = data.tool_name || '';
        const existing = state.lossToolCalls.filter((t) => t.name !== toolName);
        set({
          lossToolCalls: [
            ...existing,
            { name: toolName, label: TOOL_LABELS[toolName] || toolName, done: false },
          ],
        });
      } else if (data.event_type === 'tool_result') {
        const toolName = data.tool_name || '';
        const summary = data.content || '';
        set({
          lossToolCalls: state.lossToolCalls.map((t) =>
            t.name === toolName ? { ...t, done: true, summary } : t,
          ),
        });
      } else if (data.event_type === 'done') {
        const fullContent = data.content || state.lossAnalysisContent;
        set({
          lossAnalyzing: false,
          lossAnalysisContent: fullContent,
          lossDone: true,
        });
        unlisten();
        set({ _lossUnlisten: null });
      } else if (data.event_type === 'error') {
        set({
          lossAnalyzing: false,
          lossError: data.content || '败因分析失败',
          lossDone: true,
        });
        unlisten();
        set({ _lossUnlisten: null });
      }
    });

    set({ _lossUnlisten: unlisten });

    await invoke('analyze_loss_reasons', {
      date,
      lossStocks: lossStocks,
    }).catch((e: Error) => {
      set({ lossAnalyzing: false, lossError: e.message, lossDone: true });
      unlisten();
      set({ _lossUnlisten: null });
    });
  },

  closeLossAnalysis: () => {
    const { _lossUnlisten } = get();
    if (_lossUnlisten) {
      _lossUnlisten();
    }
    set({
      lossAnalyzing: false,
      lossAnalysisContent: '',
      lossAnalysisDate: null,
      lossToolCalls: [],
      lossThinkingSteps: [],
      lossError: null,
      lossDone: false,
      _lossUnlisten: null,
    });
  },

  getDateGroups: () => {
    const { trackingStocks, quotes } = get();
    const dateMap = new Map<string, TrackingStockWithQuote[]>();

    for (const stock of trackingStocks) {
      const quote = quotes.get(stock.code);
      const currentPrice = quote?.price;
      const changeFromAdded = currentPrice && stock.added_price > 0
        ? ((currentPrice - stock.added_price) / stock.added_price) * 100
        : undefined;

      const enriched: TrackingStockWithQuote = {
        ...stock,
        current_price: currentPrice,
        change_from_added: changeFromAdded,
      };

      const list = dateMap.get(stock.added_date) || [];
      list.push(enriched);
      dateMap.set(stock.added_date, list);
    }

    const groups: DateGroup[] = [];
    for (const [date, stocks] of dateMap) {
      const withPrice = stocks.filter((s) => s.change_from_added !== undefined);
      const winCount = withPrice.filter((s) => (s.change_from_added || 0) > 0).length;
      const loseCount = withPrice.filter((s) => (s.change_from_added || 0) <= 0).length;
      const totalCount = stocks.length;
      const winRate = withPrice.length > 0 ? (winCount / withPrice.length) * 100 : 0;
      const avgReturn = withPrice.length > 0
        ? withPrice.reduce((sum, s) => sum + (s.change_from_added || 0), 0) / withPrice.length
        : 0;

      groups.push({ date, stocks, winCount, loseCount, totalCount, winRate, avgReturn });
    }

    groups.sort((a, b) => b.date.localeCompare(a.date));
    return groups;
  },
}));
