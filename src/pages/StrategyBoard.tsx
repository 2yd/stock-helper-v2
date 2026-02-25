import { useState, useCallback, useMemo } from 'react';
import { Switch, App } from 'antd';
import { RefreshCw, Copy, Sparkles, ScanSearch, Filter, ArrowLeft, Activity, Loader2 } from 'lucide-react';
import StockTable, { UnifiedStockRow, strategyRowToUnified } from '../components/StockTable';
import StatusBar from '../components/StatusBar';
import WatchlistDiagnosePanel from '../components/WatchlistDiagnosePanel';
import KlineChart from '../components/KlineChart';
import TechnicalPanel from '../components/TechnicalPanel';
import { useStockStore } from '../stores/stockStore';
import { useSettingsStore } from '../stores/settingsStore';
import { useStockRefresh } from '../hooks/useStockRefresh';
import { useWatchlistStore } from '../stores/watchlistStore';
import { StrategyResultRow } from '../types';

type ViewTab = 'table' | 'detail';

export default function StrategyBoard() {
  const { message } = App.useApp();
  const {
    results, loading, scanTotal,
    scanMarket, generateInstructions,
  } = useStockStore();
  const { settings } = useSettingsStore();
  const { manualRefresh } = useStockRefresh();

  const {
    analysis, analysisLoading, analysisPeriod,
    showDiagnosePanel,
    loadAnalysis,
    setPeriod, startDiagnosis, setShowDiagnosePanel, resetDiagnosis,
  } = useWatchlistStore();

  const [selectedStock, setSelectedStock] = useState<StrategyResultRow | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [activeTab, setActiveTab] = useState<ViewTab>('table');
  const [activeIndicators, setActiveIndicators] = useState<string[]>(['MA']);

  const activeStrategy = settings?.strategies.find(s => s.id === settings.active_strategy_id);

  const handleToggleAutoRefresh = useCallback((checked: boolean) => {
    setAutoRefresh(checked);
    if (checked) {
      useStockStore.getState().startAutoRefresh(settings?.refresh_interval_secs || 60);
    } else {
      useStockStore.getState().stopAutoRefresh();
    }
  }, [settings?.refresh_interval_secs]);

  const handleCopyHighScore = useCallback(() => {
    const highScoreStocks = results.filter(r => r.score >= 80);
    if (highScoreStocks.length === 0) {
      message.info('暂无≥80分的股票');
      return;
    }
    const text = highScoreStocks
      .map(r => `${r.code.replace(/^(sh|sz|bj)/, '')} ${r.name} ${r.score}分 PE${r.pe_ttm.toFixed(1)} ROE${r.roe.toFixed(1)}%`)
      .join('\n');
    navigator.clipboard.writeText(text);
    message.success(`已复制 ${highScoreStocks.length} 只≥80分股票`);
  }, [results]);

  const handleAIInstructions = useCallback(async () => {
    await generateInstructions();
    message.success('AI指令生成完成');
  }, [generateInstructions]);

  const handleStockClick = useCallback((row: UnifiedStockRow) => {
    const original = results.find(r => r.code === row.code) || null;
    setSelectedStock(original);
    // 直接调用 loadAnalysis，无需股票在自选列表中
    loadAnalysis(row.code, row.name);
    setActiveTab('detail');
  }, [results, loadAnalysis]);

  const handleBackToTable = useCallback(() => {
    setActiveTab('table');
  }, []);

  const handleDiagnose = useCallback(async () => {
    if (!selectedStock) return;
    resetDiagnosis();
    await startDiagnosis(selectedStock.code, selectedStock.name);
  }, [selectedStock, startDiagnosis, resetDiagnosis]);

  const handleScan = useCallback(async () => {
    const sid = settings?.active_strategy_id || 'default';
    await scanMarket(sid);
    message.success(`扫描完成，筛选出 ${useStockStore.getState().results.length} 只标的`);
  }, [settings?.active_strategy_id, scanMarket]);

  const unifiedResults = useMemo(() => results.map(strategyRowToUnified), [results]);

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-bg-elevated/50 border-b border-[#30363D] flex-shrink-0">
        <div className="flex items-center gap-3">
          {activeTab === 'detail' ? (
            <>
              <button
                onClick={handleBackToTable}
                className="flex items-center gap-1.5 text-xs text-txt-secondary hover:text-primary-gold transition-colors cursor-pointer"
              >
                <ArrowLeft size={14} />
                <span>返回列表</span>
              </button>
              {selectedStock && (
                <div className="flex items-center gap-2">
                  <span className="text-sm font-bold text-txt-primary">{selectedStock.name}</span>
                  <span className="text-xs text-txt-muted font-mono">{selectedStock.code}</span>
                  {(() => {
                    const changePct = selectedStock.change_pct;
                    const changeColor = changePct > 0 ? 'text-functional-up' : changePct < 0 ? 'text-functional-down' : 'text-txt-primary';
                    return (
                      <>
                        <span className={`font-mono font-bold text-sm ${changeColor}`}>{selectedStock.price.toFixed(2)}</span>
                        <span className={`font-mono text-xs ${changeColor}`}>
                          {changePct > 0 ? '+' : ''}{changePct.toFixed(2)}%
                        </span>
                        <span className="text-xs text-primary-gold font-semibold ml-1">{selectedStock.score}分</span>
                      </>
                    );
                  })()}
                </div>
              )}
            </>
          ) : (
            <>
              <h2 className="text-sm font-bold text-primary-gold">
                {activeStrategy?.name || '多因子综合选股'}
              </h2>
              <span className="text-xs text-txt-muted">
                {results.length > 0
                  ? `Top ${results.length} · 全市场扫描`
                  : '点击扫描开始全市场选股'
                }
              </span>
              {activeStrategy && (
                <div className="flex items-center gap-1.5 text-[10px] text-txt-muted/70">
                  <Filter size={10} />
                  <span>PE≤{activeStrategy.filters.pe_max} · ROE≥{activeStrategy.filters.roe_min}% · 市值≥{activeStrategy.filters.min_market_cap}亿</span>
                </div>
              )}
            </>
          )}
        </div>

        <div className="flex items-center gap-2">
          {activeTab === 'table' ? (
            <>
              <button
                onClick={handleScan}
                disabled={loading}
                className="flex items-center gap-1 px-4 py-1.5 text-xs font-medium bg-gradient-to-r from-blue-600 to-indigo-500 text-white rounded-md hover:from-blue-500 hover:to-indigo-400 transition-all cursor-pointer shadow-lg shadow-blue-500/20"
              >
                <ScanSearch size={14} className={loading ? 'animate-spin' : ''} />
                {loading ? '扫描中...' : '全市场扫描'}
              </button>

              <button
                onClick={handleAIInstructions}
                disabled={results.length === 0}
                className="flex items-center gap-1 px-3 py-1 text-xs font-medium bg-gradient-to-r from-purple-600/80 to-purple-500/80 text-purple-200 rounded-md hover:from-purple-500/80 hover:to-purple-400/80 transition-all cursor-pointer shadow-lg shadow-purple-500/10 disabled:opacity-40"
              >
                <Sparkles size={13} />
                AI分析
              </button>

              <button
                onClick={manualRefresh}
                disabled={loading}
                className="flex items-center gap-1 px-3 py-1 text-xs font-medium bg-bg-card text-txt-secondary rounded-md border border-[#30363D] hover:border-[#484F58] hover:text-txt-primary transition-all cursor-pointer"
              >
                <RefreshCw size={13} className={loading ? 'animate-spin' : ''} />
                刷新
              </button>

              <div className="flex items-center gap-1.5 text-xs text-txt-secondary">
                <span>自动</span>
                <Switch
                  size="small"
                  checked={autoRefresh}
                  onChange={handleToggleAutoRefresh}
                />
              </div>

              <button
                onClick={handleCopyHighScore}
                disabled={results.length === 0}
                className="flex items-center gap-1 px-3 py-1 text-xs font-semibold bg-gradient-to-r from-red-700 to-red-500 text-white rounded-md hover:from-red-600 hover:to-red-400 transition-all cursor-pointer shadow-lg shadow-red-500/20 disabled:opacity-40"
              >
                <Copy size={13} />
                复制≥80分
              </button>
            </>
          ) : (
            <button
              onClick={handleDiagnose}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-gradient-to-r from-red-600/80 to-red-500/80 text-white hover:from-red-500 hover:to-red-400 shadow-lg shadow-red-500/10 transition-all cursor-pointer"
            >
              <Activity size={13} />
              AI 诊断
            </button>
          )}
        </div>
      </div>

      {/* ===== TABLE VIEW ===== */}
      {activeTab === 'table' && (
        <>
          {/* Empty State */}
          {results.length === 0 && !loading && (
            <div className="flex-1 flex items-center justify-center">
              <div className="text-center space-y-4">
                <div className="text-5xl opacity-15">📈</div>
                <div className="space-y-2">
                  <p className="text-txt-primary text-base font-medium">
                    多因子量化选股系统
                  </p>
                  <p className="text-txt-muted text-sm max-w-md">
                    从沪深主板 + 创业板全市场扫描，基于价值、质量、动量、资金、风险、消息六大维度综合打分，融合AI消息面分析，智能筛选出最优标的
                  </p>
                </div>
                <button
                  onClick={handleScan}
                  disabled={loading}
                  className="px-8 py-2.5 text-sm font-medium bg-gradient-to-r from-blue-600 to-indigo-500 text-white rounded-lg hover:from-blue-500 hover:to-indigo-400 transition-all cursor-pointer shadow-xl shadow-blue-500/30"
                >
                  <ScanSearch size={16} className="inline mr-2 -mt-0.5" />
                  开始全市场扫描
                </button>
                {activeStrategy && (
                  <div className="flex flex-wrap justify-center gap-2 mt-3 text-xs text-txt-muted">
                    <span className="px-2 py-0.5 rounded bg-bg-card border border-[#30363D]">
                      价值 {activeStrategy.weights.value}%
                    </span>
                    <span className="px-2 py-0.5 rounded bg-bg-card border border-[#30363D]">
                      质量 {activeStrategy.weights.quality}%
                    </span>
                    <span className="px-2 py-0.5 rounded bg-bg-card border border-[#30363D]">
                      动量 {activeStrategy.weights.momentum}%
                    </span>
                    <span className="px-2 py-0.5 rounded bg-bg-card border border-[#30363D]">
                      资金 {activeStrategy.weights.capital}%
                    </span>
                    <span className="px-2 py-0.5 rounded bg-bg-card border border-[#30363D]">
                      风险 {activeStrategy.weights.risk}%
                    </span>
                    <span className="px-2 py-0.5 rounded bg-bg-card border border-[#30363D]">
                      消息 {activeStrategy.weights.sentiment || 0}%
                    </span>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Table */}
          {(results.length > 0 || loading) && (
            <div className="flex-1 overflow-hidden">
              <StockTable
                mode="strategy"
                data={unifiedResults}
                loading={loading}
                onStockClick={handleStockClick}
              />
            </div>
          )}
        </>
      )}

      {/* ===== DETAIL VIEW (K线图 + 技术面板) ===== */}
      {activeTab === 'detail' && (
        <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
          {analysisLoading ? (
            <div className="flex-1 flex items-center justify-center">
              <Loader2 size={24} className="animate-spin text-primary-gold" />
              <span className="ml-2 text-xs text-txt-muted">加载K线和技术指标中...</span>
            </div>
          ) : analysis ? (
            <div className="flex-1 flex min-h-0 overflow-hidden">
              <div className="flex-[6] min-h-0 min-w-0 overflow-hidden">
                <KlineChart
                  klineData={analysis.kline_data}
                  indicators={analysis.indicators}
                  period={analysisPeriod}
                  onPeriodChange={setPeriod}
                  activeIndicators={activeIndicators}
                  onIndicatorsChange={setActiveIndicators}
                />
              </div>
              <div className="flex-[4] min-h-0 min-w-0 border-l border-[#30363D] bg-bg-card overflow-auto">
                <TechnicalPanel analysis={analysis} onDiagnose={handleDiagnose} />
              </div>
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center">
              <p className="text-sm text-txt-muted">请从列表中选择一只股票查看详情</p>
            </div>
          )}
        </div>
      )}

      {/* Status Bar */}
      <StatusBar />

      {/* AI Diagnose Panel (with tools support) */}
      {showDiagnosePanel && selectedStock && (
        <WatchlistDiagnosePanel
          code={selectedStock.code}
          name={selectedStock.name}
          onClose={() => setShowDiagnosePanel(false)}
        />
      )}
    </div>
  );
}
