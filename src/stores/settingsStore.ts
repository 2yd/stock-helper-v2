import { create } from 'zustand';
import { safeInvoke as invoke } from '../hooks/useTauri';
import { AppSettings, AIConfig } from '../types';
import logger from '../utils/logger';

const defaultSettings: AppSettings = {
  refresh_interval_secs: 30,
  auto_refresh: true,
  ai_instruction_enabled: true,
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

interface SettingsStore {
  settings: AppSettings | null;
  loading: boolean;
  testingConfigId: string | null;
  exportingLogs: boolean;

  loadSettings: () => Promise<void>;
  saveSettings: (settings: AppSettings) => Promise<void>;
  addAIConfig: (config: AIConfig) => Promise<void>;
  removeAIConfig: (configId: string) => Promise<void>;
  updateAIConfig: (config: AIConfig) => Promise<void>;
  setActiveAIConfig: (configId: string) => Promise<void>;
  testAIConfig: (config: AIConfig) => Promise<string>;
  exportLogs: () => Promise<string>;
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: null,
  loading: false,
  testingConfigId: null,
  exportingLogs: false,

  loadSettings: async () => {
    set({ loading: true });
    try {
      const settings = await invoke<AppSettings>('get_settings');
      set({ settings, loading: false });
    } catch (e) {
      logger.error(`Failed to load settings: ${e}`);
      set({ settings: { ...defaultSettings }, loading: false });
    }
  },

  saveSettings: async (settings: AppSettings) => {
    try {
      await invoke('save_settings', { settings });
      set({ settings });
    } catch (e) {
      logger.error(`Failed to save settings: ${e}`);
    }
  },

  addAIConfig: async (config: AIConfig) => {
    try {
      const settings = await invoke<AppSettings>('add_ai_config', { config });
      set({ settings });
    } catch (e) {
      logger.error(`Failed to add AI config: ${e}`);
    }
  },

  removeAIConfig: async (configId: string) => {
    try {
      const settings = await invoke<AppSettings>('remove_ai_config', { configId });
      set({ settings });
    } catch (e) {
      logger.error(`Failed to remove AI config: ${e}`);
    }
  },

  updateAIConfig: async (config: AIConfig) => {
    try {
      const settings = await invoke<AppSettings>('update_ai_config', { config });
      set({ settings });
    } catch (e) {
      logger.error(`Failed to update AI config: ${e}`);
    }
  },

  setActiveAIConfig: async (configId: string) => {
    try {
      const settings = await invoke<AppSettings>('set_active_ai_config', { configId });
      set({ settings });
    } catch (e) {
      logger.error(`Failed to set active AI config: ${e}`);
    }
  },

  testAIConfig: async (config: AIConfig) => {
    set({ testingConfigId: config.id });
    try {
      const result = await invoke<string>('test_ai_config', { config });
      return result;
    } finally {
      set({ testingConfigId: null });
    }
  },

  exportLogs: async () => {
    set({ exportingLogs: true });
    try {
      const result = await invoke<string>('export_logs');
      return result;
    } finally {
      set({ exportingLogs: false });
    }
  },
}));
