import { create } from 'zustand';
import { safeInvoke as invoke } from '../hooks/useTauri';
import { AppSettings, AIConfig, StrategyConfig } from '../types';

const defaultSettings: AppSettings = {
  refresh_interval_secs: 30,
  auto_refresh: true,
  ai_instruction_enabled: true,
  data_source_primary: 'sina',
  ai_configs: [],
  active_ai_config_id: null,
  strategies: [
    {
      id: 'default',
      name: '多因子综合选股',
      description: '基于价值、质量、动量、资金、风险、消息六大维度的AI量化选股策略',
      watch_codes: [],
      weights: { value: 15, quality: 15, momentum: 25, capital: 20, risk: 10, sentiment: 15 },
      filters: {
        exclude_st: true, exclude_new_stock_days: 60,
        min_market_cap: 30, max_market_cap: 0, min_price: 3, min_amount: 5000,
        pe_max: 100, pe_min: 0, pb_max: 20, roe_min: 5,
      },
      enabled: true,
      top_n: 50,
    },
  ],
  active_strategy_id: 'default',
  token_usage_today: 0,
  qgqp_b_id: '',
};

interface SettingsStore {
  settings: AppSettings | null;
  loading: boolean;
  testingConfigId: string | null;

  loadSettings: () => Promise<void>;
  saveSettings: (settings: AppSettings) => Promise<void>;
  addAIConfig: (config: AIConfig) => Promise<void>;
  removeAIConfig: (configId: string) => Promise<void>;
  updateAIConfig: (config: AIConfig) => Promise<void>;
  setActiveAIConfig: (configId: string) => Promise<void>;
  updateStrategy: (strategy: StrategyConfig) => Promise<void>;
  testAIConfig: (config: AIConfig) => Promise<string>;
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: null,
  loading: false,
  testingConfigId: null,

  loadSettings: async () => {
    set({ loading: true });
    try {
      const settings = await invoke<AppSettings>('get_settings');
      set({ settings, loading: false });
    } catch (e) {
      console.error('Failed to load settings, using defaults:', e);
      set({ settings: { ...defaultSettings }, loading: false });
    }
  },

  saveSettings: async (settings: AppSettings) => {
    try {
      await invoke('save_settings', { settings });
      set({ settings });
    } catch (e) {
      console.error('Failed to save settings:', e);
    }
  },

  addAIConfig: async (config: AIConfig) => {
    try {
      const settings = await invoke<AppSettings>('add_ai_config', { config });
      set({ settings });
    } catch (e) {
      console.error('Failed to add AI config:', e);
    }
  },

  removeAIConfig: async (configId: string) => {
    try {
      const settings = await invoke<AppSettings>('remove_ai_config', { configId });
      set({ settings });
    } catch (e) {
      console.error('Failed to remove AI config:', e);
    }
  },

  updateAIConfig: async (config: AIConfig) => {
    try {
      const settings = await invoke<AppSettings>('update_ai_config', { config });
      set({ settings });
    } catch (e) {
      console.error('Failed to update AI config:', e);
    }
  },

  setActiveAIConfig: async (configId: string) => {
    try {
      const settings = await invoke<AppSettings>('set_active_ai_config', { configId });
      set({ settings });
    } catch (e) {
      console.error('Failed to set active AI config:', e);
    }
  },

  updateStrategy: async (strategy: StrategyConfig) => {
    try {
      const settings = await invoke<AppSettings>('update_strategy_config', { strategy });
      set({ settings });
    } catch (e) {
      console.error('Failed to update strategy:', e);
    }
  },

  testAIConfig: async (config: AIConfig) => {
    set({ testingConfigId: config.id });
    try {
      const result = await invoke<string>('test_ai_config', { config });
      return result;
    } catch (e: unknown) {
      throw e;
    } finally {
      set({ testingConfigId: null });
    }
  },
}));
