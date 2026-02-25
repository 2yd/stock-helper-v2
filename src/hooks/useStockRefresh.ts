import { useEffect, useCallback } from 'react';
import { useStockStore } from '../stores/stockStore';
import { useSettingsStore } from '../stores/settingsStore';

export function useStockRefresh() {
  const { scanMarket, startAutoRefresh, stopAutoRefresh, fetchMarketStatus, fetchTokenUsage } = useStockStore();
  const { settings, loadSettings } = useSettingsStore();

  useEffect(() => {
    loadSettings();
    fetchMarketStatus();
    fetchTokenUsage();

    const statusTimer = setInterval(() => {
      fetchMarketStatus();
    }, 30000);

    return () => clearInterval(statusTimer);
  }, []);

  useEffect(() => {
    if (!settings) return;

    if (settings.auto_refresh) {
      startAutoRefresh(settings.refresh_interval_secs);
    } else {
      stopAutoRefresh();
    }

    return () => stopAutoRefresh();
  }, [settings?.auto_refresh, settings?.refresh_interval_secs]);

  const manualRefresh = useCallback(() => {
    scanMarket(settings?.active_strategy_id || 'default');
  }, [settings?.active_strategy_id]);

  return { manualRefresh };
}
