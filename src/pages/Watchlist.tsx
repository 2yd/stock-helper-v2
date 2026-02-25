import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { Search, Plus, Loader2, Activity, X, RefreshCw, Eye, ArrowLeft } from 'lucide-react';
import { useWatchlistStore } from '../stores/watchlistStore';
import { safeInvoke } from '../hooks/useTauri';
import { WatchlistQuote, StockLabel } from '../types';
import StockTable, { UnifiedStockRow, watchlistQuoteToUnified } from '../components/StockTable';
import KlineChart from '../components/KlineChart';
import TechnicalPanel from '../components/TechnicalPanel';
import WatchlistDiagnosePanel from '../components/WatchlistDiagnosePanel';

interface SearchResult {
  code: string;
  name: string;
  market: string;
}

type ViewTab = 'table' | 'detail';

export default function Watchlist() {
  const {
    stocks, quotes, quotesLoading, loading, selectedCode, analysis, analysisLoading, analysisPeriod,
    showDiagnosePanel,
    loadStocks, addStock, removeStock, selectStock, setPeriod,
    startDiagnosis, setShowDiagnosePanel, resetDiagnosis,
    loadQuotes, startAutoRefresh, stopAutoRefresh,
  } = useWatchlistStore();

  const [showAdd, setShowAdd] = useState(false);
  const [addKeyword, setAddKeyword] = useState('');
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [searching, setSearching] = useState(false);
  const [activeIndicators, setActiveIndicators] = useState<string[]>(['MA']);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [activeTab, setActiveTab] = useState<ViewTab>('table');
  const unlistenRef = useRef<(() => void) | null>(null);
  const searchTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const addInputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadStocks();
    return () => { stopAutoRefresh(); };
  }, []);

  const doSearch = useCallback(async (kw: string) => {
    if (!kw.trim()) { setSearchResults([]); return; }
    setSearching(true);
    try {
      const results = await safeInvoke<SearchResult[]>('search_stocks', { keyword: kw.trim() });
      setSearchResults(results || []);
    } catch {
      setSearchResults([]);
    } finally {
      setSearching(false);
    }
  }, []);

  const handleAddKeywordChange = useCallback((value: string) => {
    setAddKeyword(value);
    if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
    if (!value.trim()) { setSearchResults([]); return; }
    searchTimerRef.current = setTimeout(() => doSearch(value), 300);
  }, [doSearch]);

  const handleSelectSearchResult = useCallback(async (result: SearchResult) => {
    await addStock(result.code, result.name);
    setAddKeyword('');
    setSearchResults([]);
    setShowAdd(false);
  }, [addStock]);

  const handleDiagnose = useCallback(async () => {
    if (!analysis) return;
    if (unlistenRef.current) unlistenRef.current();
    resetDiagnosis();
    const unlisten = await startDiagnosis(analysis.code, analysis.name);
    unlistenRef.current = unlisten;
  }, [analysis, startDiagnosis, resetDiagnosis]);

  const handleToggleAutoRefresh = useCallback((checked: boolean) => {
    setAutoRefresh(checked);
    if (checked) { startAutoRefresh(15); } else { stopAutoRefresh(); }
  }, [startAutoRefresh, stopAutoRefresh]);

  const handleStockClick = useCallback((code: string) => {
    selectStock(code);
    setActiveTab('detail');
  }, [selectStock]);

  const handleBackToTable = useCallback(() => {
    setActiveTab('table');
  }, []);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setShowAdd(false);
        setAddKeyword('');
        setSearchResults([]);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      if (unlistenRef.current) unlistenRef.current();
      if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
    };
  }, []);

  const emptyDefault: WatchlistQuote = {
    code: '', name: '', price: 0, pre_close: 0, open: 0, high: 0, low: 0,
    volume: 0, amount: 0, change_pct: 0, change_price: 0,
    pe_ttm: 0, pb: 0, roe: 0, total_market_cap: 0, float_market_cap: 0,
    turnover_rate: 0, volume_ratio: 0, main_net_inflow: 0, pct_5d: 0, pct_20d: 0,
    revenue_yoy: 0, amplitude: 0, date: '', time: '',
  };

  /** 根据行情数据动态生成标签 */
  const computeLabels = useCallback((q: WatchlistQuote): StockLabel[] => {
    const labels: StockLabel[] = [];
    if (q.change_pct >= 9.8) labels.push({ text: '涨停', color: '#F43F5E', icon: null });
    else if (q.change_pct <= -9.8) labels.push({ text: '跌停', color: '#22C55E', icon: null });
    if (q.volume_ratio >= 3) labels.push({ text: '放量', color: '#F59E0B', icon: null });
    if (q.main_net_inflow >= 5e7) labels.push({ text: '主力流入', color: '#F43F5E', icon: null });
    else if (q.main_net_inflow <= -5e7) labels.push({ text: '主力流出', color: '#22C55E', icon: null });
    if (q.pct_5d >= 15) labels.push({ text: '5日强势', color: '#F43F5E', icon: null });
    else if (q.pct_5d <= -15) labels.push({ text: '5日弱势', color: '#22C55E', icon: null });
    if (q.pe_ttm > 0 && q.pe_ttm < 15 && q.roe >= 15) labels.push({ text: '低估优质', color: '#3B82F6', icon: 'crown' });
    return labels;
  }, []);

  const tableData: UnifiedStockRow[] = useMemo(() => {
    const quoteMap = new Map(quotes.map((q) => [q.code, q]));
    return stocks.map((s) => {
      const q = quoteMap.get(s.code);
      if (q) {
        const row = watchlistQuoteToUnified({ ...q, hasQuote: true });
        row.labels = computeLabels(q);
        return row;
      }
      return watchlistQuoteToUnified({
        ...emptyDefault, code: s.code, name: s.name,
        hasQuote: false,
      });
    });
  }, [stocks, quotes, computeLabels]);

  return (
    <div className="flex flex-col h-full overflow-hidden relative">
      {/* Top Toolbar */}
      <div className="flex items-center gap-3 px-4 py-2 bg-bg-elevated/50 border-b border-[#30363D] flex-shrink-0">
        {activeTab === 'detail' ? (
          <button
            onClick={handleBackToTable}
            className="flex items-center gap-1.5 text-xs text-txt-secondary hover:text-primary-gold transition-colors cursor-pointer"
          >
            <ArrowLeft size={14} />
            <span>返回列表</span>
          </button>
        ) : (
          <>
            <h2 className="text-sm font-bold text-primary-gold">自选盯盘</h2>
            <span className="text-xs text-txt-muted">{stocks.length} 只</span>
          </>
        )}

        {activeTab === 'detail' && analysis && (
          <div className="flex items-center gap-2">
            <span className="text-sm font-bold text-txt-primary">{analysis.name}</span>
            <span className="text-xs text-txt-muted font-mono">{analysis.code}</span>
            {analysis.kline_data.length > 0 && (() => {
              const last = analysis.kline_data[analysis.kline_data.length - 1];
              const prevClose = analysis.kline_data.length > 1
                ? analysis.kline_data[analysis.kline_data.length - 2].close
                : last.open;
              const changePct = ((last.close - prevClose) / prevClose) * 100;
              const changeColor = changePct > 0 ? 'text-functional-up' : changePct < 0 ? 'text-functional-down' : 'text-txt-primary';
              return (
                <>
                  <span className={`font-mono font-bold text-sm ${changeColor}`}>{last.close.toFixed(2)}</span>
                  <span className={`font-mono text-xs ${changeColor}`}>
                    {changePct > 0 ? '+' : ''}{changePct.toFixed(2)}%
                  </span>
                </>
              );
            })()}
          </div>
        )}

        <div className="flex-1" />

        {activeTab === 'table' && (
          <>
            <div ref={dropdownRef} className="relative">
              <button
                onClick={() => {
                  setShowAdd(!showAdd);
                  if (!showAdd) setTimeout(() => addInputRef.current?.focus(), 100);
                }}
                className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-xs font-medium text-primary-gold bg-primary-gold/10 border border-primary-gold/20 hover:bg-primary-gold/20 transition-all cursor-pointer"
              >
                <Plus size={13} />
                添加自选
              </button>

              {showAdd && (
                <div className="absolute right-0 top-full mt-1 w-80 bg-bg-card border border-[#30363D] rounded-lg shadow-xl z-50">
                  <div className="p-2.5 flex items-center gap-2">
                    <div className="relative flex-1">
                      <Search size={12} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-txt-muted" />
                      <input
                        ref={addInputRef}
                        type="text"
                        placeholder="输入代码、名称或拼音搜索..."
                        value={addKeyword}
                        onChange={(e) => handleAddKeywordChange(e.target.value)}
                        className="w-full pl-7 pr-3 py-1.5 rounded-lg bg-bg-base border border-[#30363D] text-xs text-txt-primary placeholder:text-txt-muted outline-none focus:border-primary-gold/50 transition-colors"
                      />
                    </div>
                    <button onClick={() => { setShowAdd(false); setAddKeyword(''); setSearchResults([]); }} className="p-1 rounded hover:bg-bg-elevated cursor-pointer">
                      <X size={13} className="text-txt-muted" />
                    </button>
                  </div>
                  {(searchResults.length > 0 || searching) && (
                    <div className="max-h-[240px] overflow-auto border-t border-[#30363D]">
                      {searching ? (
                        <div className="flex items-center justify-center py-4">
                          <Loader2 size={14} className="animate-spin text-txt-muted" />
                          <span className="ml-2 text-xs text-txt-muted">搜索中...</span>
                        </div>
                      ) : searchResults.map((r) => {
                        const alreadyAdded = stocks.some((s) => s.code === r.code);
                        return (
                          <button
                            key={r.code}
                            onClick={() => !alreadyAdded && handleSelectSearchResult(r)}
                            disabled={alreadyAdded}
                            className={`flex items-center w-full px-3 py-1.5 text-left transition-colors ${alreadyAdded ? 'opacity-40 cursor-not-allowed' : 'hover:bg-bg-elevated cursor-pointer'}`}
                          >
                            <span className="text-xs font-bold text-txt-primary">{r.name}</span>
                            <span className="ml-2 text-[10px] text-txt-muted font-mono">{r.code}</span>
                            <span className="ml-auto text-[10px] text-txt-muted">{r.market}</span>
                            {alreadyAdded && <span className="ml-2 text-[10px] text-primary-gold">已添加</span>}
                          </button>
                        );
                      })}
                    </div>
                  )}
                  {addKeyword.trim() && !searching && searchResults.length === 0 && (
                    <div className="py-3 text-center text-xs text-txt-muted border-t border-[#30363D]">未找到匹配的A股</div>
                  )}
                </div>
              )}
            </div>

            <button
              onClick={loadQuotes}
              disabled={quotesLoading || stocks.length === 0}
              className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-xs font-medium bg-bg-card text-txt-secondary border border-[#30363D] hover:border-[#484F58] hover:text-txt-primary transition-all cursor-pointer disabled:opacity-40"
            >
              <RefreshCw size={12} className={quotesLoading ? 'animate-spin' : ''} />
              刷新行情
            </button>

            <label className="flex items-center gap-1.5 text-xs text-txt-muted cursor-pointer select-none">
              <input
                type="checkbox"
                checked={autoRefresh}
                onChange={(e) => handleToggleAutoRefresh(e.target.checked)}
                className="accent-primary-gold w-3 h-3"
              />
              自动(15s)
            </label>
          </>
        )}

        {activeTab === 'detail' && analysis && (
          <button
            onClick={handleDiagnose}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-gradient-to-r from-red-600/80 to-red-500/80 text-white hover:from-red-500 hover:to-red-400 shadow-lg shadow-red-500/10 transition-all cursor-pointer"
          >
            <Activity size={13} />
            AI 诊断
          </button>
        )}
      </div>

      {/* ===== TABLE VIEW ===== */}
      {activeTab === 'table' && (
        <div className="flex-1 overflow-hidden">
          {stocks.length === 0 && !loading ? (
            <div className="flex flex-col items-center justify-center h-64 text-txt-muted">
              <Eye size={48} className="mb-3 opacity-15" />
              <p className="text-sm font-medium text-txt-primary mb-1">暂无自选股</p>
              <p className="text-xs opacity-60">点击右上角"添加自选"开始盯盘</p>
            </div>
          ) : (
            <StockTable
              mode="watchlist"
              data={tableData}
              loading={quotesLoading && tableData.length === 0}
              onRowClick={handleStockClick}
              onRemove={removeStock}
            />
          )}
        </div>
      )}

      {/* ===== DETAIL VIEW ===== */}
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

      {/* AI Diagnose Panel */}
      {showDiagnosePanel && (
        <WatchlistDiagnosePanel
          code={analysis?.code || ''}
          name={analysis?.name || ''}
          onClose={() => setShowDiagnosePanel(false)}
        />
      )}
    </div>
  );
}
