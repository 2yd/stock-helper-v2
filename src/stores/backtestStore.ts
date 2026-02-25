import { create } from 'zustand';
import { safeInvoke as invoke } from '../hooks/useTauri';
import {
  BacktestConfig,
  BacktestResult,
  FactorWeights,
} from '../types';

const defaultConfig: BacktestConfig = {
  codes: [],
  start_date: '',
  end_date: '',
  initial_capital: 1000000,
  commission_rate: 0.0003,
  slippage: 0.001,
  buy_threshold: 70,
  sell_threshold: 40,
  stop_loss: 0.08,
  take_profit: 0.20,
  factor_weights: {
    value: 25,
    quality: 25,
    momentum: 20,
    capital: 15,
    risk: 15,
  },
  max_position_pct: 0.25,
};

interface BacktestStore {
  config: BacktestConfig;
  result: BacktestResult | null;
  running: boolean;
  error: string | null;

  // Config actions
  updateConfig: (partial: Partial<BacktestConfig>) => void;
  updateWeights: (weights: FactorWeights) => void;
  setCodes: (codes: string[]) => void;
  setDateRange: (start: string, end: string) => void;
  resetConfig: () => void;

  // Backtest actions
  runBacktest: () => Promise<void>;
  clearResult: () => void;
}

export const useBacktestStore = create<BacktestStore>((set, get) => ({
  config: { ...defaultConfig },
  result: null,
  running: false,
  error: null,

  updateConfig: (partial: Partial<BacktestConfig>) => {
    set((s) => ({ config: { ...s.config, ...partial } }));
  },

  updateWeights: (weights: FactorWeights) => {
    set((s) => ({ config: { ...s.config, factor_weights: weights } }));
  },

  setCodes: (codes: string[]) => {
    set((s) => ({ config: { ...s.config, codes } }));
  },

  setDateRange: (start: string, end: string) => {
    set((s) => ({ config: { ...s.config, start_date: start, end_date: end } }));
  },

  resetConfig: () => {
    set({ config: { ...defaultConfig }, result: null, error: null });
  },

  runBacktest: async () => {
    const { config } = get();
    if (config.codes.length === 0) {
      set({ error: '请至少选择一只股票' });
      return;
    }
    if (!config.start_date || !config.end_date) {
      set({ error: '请设置回测时间范围' });
      return;
    }

    set({ running: true, error: null, result: null });
    const result = await invoke<BacktestResult>('run_backtest', { config });
    set({ result, running: false });
  },

  clearResult: () => {
    set({ result: null, error: null });
  },
}));
