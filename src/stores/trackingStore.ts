import { create } from 'zustand';
import { safeInvoke as invoke } from '../hooks/useTauri';
import { AIPickTracking, WatchlistQuote } from '../types';

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

interface TrackingStore {
  trackingStocks: AIPickTracking[];
  quotes: Map<string, WatchlistQuote>;
  loading: boolean;
  quotesLoading: boolean;
  _refreshTimer: ReturnType<typeof setInterval> | null;

  loadTrackingStocks: () => Promise<void>;
  addTracking: (code: string, name: string, addedPrice: number, rating: string, reason: string, sector: string) => Promise<void>;
  removeTracking: (code: string, addedDate: string) => Promise<void>;
  clearDate: (date: string) => Promise<void>;
  loadQuotes: () => Promise<void>;
  startAutoRefresh: (intervalSecs: number) => void;
  stopAutoRefresh: () => void;

  // Computed
  getDateGroups: () => DateGroup[];
}

export const useTrackingStore = create<TrackingStore>((set, get) => ({
  trackingStocks: [],
  quotes: new Map(),
  loading: false,
  quotesLoading: false,
  _refreshTimer: null,

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
      console.error('Failed to load tracking quotes:', e);
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
