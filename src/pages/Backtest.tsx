import { useState, useMemo, useRef, useCallback, useEffect } from 'react';
import { Play, Loader2, RotateCcw, X, TrendingUp, TrendingDown, Activity, BarChart2, Target, Award, Search } from 'lucide-react';
import { useBacktestStore } from '../stores/backtestStore';
import { safeInvoke } from '../hooks/useTauri';
import BacktestChart from '../components/BacktestChart';

interface SearchResult {
  code: string;
  name: string;
  market: string;
}

function MetricCard({ label, value, suffix, color, icon: Icon }: {
  label: string; value: string; suffix?: string; color: string; icon: typeof TrendingUp;
}) {
  return (
    <div className="flex flex-col p-3 rounded-lg bg-bg-elevated/50 border border-[#30363D]/50 hover:border-[#484F58]/50 transition-colors">
      <div className="flex items-center gap-1.5 mb-2">
        <Icon size={13} className="text-txt-muted" />
        <span className="text-[10px] text-txt-muted font-medium uppercase tracking-wider">{label}</span>
      </div>
      <div className="flex items-baseline gap-1">
        <span className={`text-xl font-bold font-mono ${color}`}>{value}</span>
        {suffix && <span className="text-xs text-txt-muted">{suffix}</span>}
      </div>
    </div>
  );
}

export default function Backtest() {
  const {
    config, result, running, error,
    updateConfig, setCodes, setDateRange, runBacktest, clearResult, resetConfig,
  } = useBacktestStore();

  const [showTrades, setShowTrades] = useState(false);
  const [searchKeyword, setSearchKeyword] = useState('');
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [searching, setSearching] = useState(false);
  const [showDropdown, setShowDropdown] = useState(false);
  const searchTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const doSearch = useCallback(async (kw: string) => {
    if (!kw.trim()) { setSearchResults([]); return; }
    setSearching(true);
    try {
      const results = await safeInvoke<SearchResult[]>('search_stocks', { keyword: kw.trim() });
      setSearchResults(results || []);
      setShowDropdown(true);
    } catch {
      setSearchResults([]);
    } finally {
      setSearching(false);
    }
  }, []);

  const handleSearchChange = useCallback((value: string) => {
    setSearchKeyword(value);
    if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
    if (!value.trim()) { setSearchResults([]); setShowDropdown(false); return; }
    searchTimerRef.current = setTimeout(() => doSearch(value), 300);
  }, [doSearch]);

  const handleSelectResult = useCallback((result: SearchResult) => {
    if (!config.codes.includes(result.code)) {
      setCodes([...config.codes, result.code]);
    }
    setSearchKeyword('');
    setSearchResults([]);
    setShowDropdown(false);
  }, [config.codes, setCodes]);

  const handleRemoveCode = (code: string) => {
    setCodes(config.codes.filter((c) => c !== code));
  };

  // Close dropdown on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setShowDropdown(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
    };
  }, []);

  const perf = result?.performance;

  const metrics = useMemo(() => {
    if (!perf) return [];
    return [
      { label: '年化收益率', value: (perf.annual_return * 100).toFixed(2), suffix: '%', color: perf.annual_return >= 0 ? 'text-primary-gold' : 'text-functional-up', icon: TrendingUp },
      { label: '最大回撤', value: (perf.max_drawdown * 100).toFixed(2), suffix: '%', color: 'text-functional-up', icon: TrendingDown },
      { label: '夏普比率', value: perf.sharpe_ratio.toFixed(2), suffix: '', color: perf.sharpe_ratio >= 1 ? 'text-functional-down' : 'text-txt-primary', icon: Activity },
      { label: '胜率', value: (perf.win_rate * 100).toFixed(1), suffix: '%', color: perf.win_rate >= 0.5 ? 'text-functional-down' : 'text-txt-primary', icon: Target },
      { label: '盈亏比', value: perf.profit_loss_ratio.toFixed(2), suffix: '', color: perf.profit_loss_ratio >= 1.5 ? 'text-functional-down' : 'text-txt-primary', icon: BarChart2 },
      { label: '总交易', value: String(perf.total_trades), suffix: '次', color: 'text-primary-blue', icon: Award },
    ];
  }, [perf]);

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Config bar */}
      <div className="flex-shrink-0 border-b border-[#30363D] bg-bg-card">
        <div className="flex items-center gap-3 px-4 py-2.5 flex-wrap">
          {/* Code input with fuzzy search */}
          <div className="flex items-center gap-1.5">
            <span className="text-xs text-txt-muted whitespace-nowrap">标的:</span>
            <div className="flex items-center gap-1 flex-wrap">
              {config.codes.map((code) => (
                <span key={code} className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-blue-600/20 text-blue-300 text-xs border border-blue-500/30">
                  {code}
                  <button onClick={() => handleRemoveCode(code)} className="hover:text-white cursor-pointer"><X size={10} /></button>
                </span>
              ))}
              <div ref={dropdownRef} className="relative">
                <div className="relative">
                  <Search size={11} className="absolute left-2 top-1/2 -translate-y-1/2 text-txt-muted" />
                  <input
                    ref={inputRef}
                    type="text"
                    placeholder="搜索代码/名称/拼音"
                    value={searchKeyword}
                    onChange={(e) => handleSearchChange(e.target.value)}
                    onFocus={() => { if (searchResults.length > 0) setShowDropdown(true); }}
                    className="w-40 pl-7 pr-2 py-0.5 rounded bg-bg-base border border-[#30363D] text-xs text-txt-primary placeholder:text-txt-muted outline-none focus:border-primary-gold/50 transition-colors"
                  />
                  {searching && <Loader2 size={11} className="absolute right-2 top-1/2 -translate-y-1/2 animate-spin text-txt-muted" />}
                </div>
                {showDropdown && (searchResults.length > 0 || (searchKeyword.trim() && !searching)) && (
                  <div className="absolute top-full left-0 mt-1 w-64 max-h-[240px] overflow-auto rounded-lg bg-bg-card border border-[#30363D] shadow-xl z-50">
                    {searchResults.length > 0 ? searchResults.map((r) => {
                      const alreadyAdded = config.codes.includes(r.code);
                      return (
                        <button
                          key={r.code}
                          onClick={() => !alreadyAdded && handleSelectResult(r)}
                          disabled={alreadyAdded}
                          className={`flex items-center w-full px-3 py-1.5 text-left transition-colors ${
                            alreadyAdded ? 'opacity-40 cursor-not-allowed' : 'hover:bg-bg-elevated cursor-pointer'
                          }`}
                        >
                          <span className="text-xs font-bold text-txt-primary">{r.name}</span>
                          <span className="ml-2 text-[10px] text-txt-muted font-mono">{r.code}</span>
                          <span className="ml-auto text-[10px] text-txt-muted">{r.market}</span>
                          {alreadyAdded && <span className="ml-2 text-[10px] text-primary-gold">已添加</span>}
                        </button>
                      );
                    }) : (
                      <div className="py-3 text-center text-xs text-txt-muted">未找到匹配的A股</div>
                    )}
                  </div>
                )}
              </div>
            </div>
          </div>

          <div className="w-px h-5 bg-[#30363D]" />

          {/* Date range */}
          <div className="flex items-center gap-1.5">
            <span className="text-xs text-txt-muted">时间:</span>
            <input
              type="date"
              value={config.start_date}
              onChange={(e) => setDateRange(e.target.value, config.end_date)}
              className="px-2 py-0.5 rounded bg-bg-base border border-[#30363D] text-xs text-txt-primary outline-none focus:border-primary-gold/50 transition-colors [color-scheme:dark]"
            />
            <span className="text-txt-muted text-xs">~</span>
            <input
              type="date"
              value={config.end_date}
              onChange={(e) => setDateRange(config.start_date, e.target.value)}
              className="px-2 py-0.5 rounded bg-bg-base border border-[#30363D] text-xs text-txt-primary outline-none focus:border-primary-gold/50 transition-colors [color-scheme:dark]"
            />
          </div>

          <div className="w-px h-5 bg-[#30363D]" />

          {/* Quick params */}
          <div className="flex items-center gap-2">
            <label className="flex items-center gap-1 text-xs text-txt-muted">
              买入≥
              <input
                type="number"
                value={config.buy_threshold}
                onChange={(e) => updateConfig({ buy_threshold: Number(e.target.value) })}
                className="w-12 px-1.5 py-0.5 rounded bg-bg-base border border-[#30363D] text-xs text-txt-primary outline-none text-center"
              />
            </label>
            <label className="flex items-center gap-1 text-xs text-txt-muted">
              卖出≤
              <input
                type="number"
                value={config.sell_threshold}
                onChange={(e) => updateConfig({ sell_threshold: Number(e.target.value) })}
                className="w-12 px-1.5 py-0.5 rounded bg-bg-base border border-[#30363D] text-xs text-txt-primary outline-none text-center"
              />
            </label>
            <label className="flex items-center gap-1 text-xs text-txt-muted">
              止损
              <input
                type="number"
                step="0.01"
                value={(config.stop_loss * 100).toFixed(0)}
                onChange={(e) => updateConfig({ stop_loss: Number(e.target.value) / 100 })}
                className="w-10 px-1 py-0.5 rounded bg-bg-base border border-[#30363D] text-xs text-txt-primary outline-none text-center"
              />
              %
            </label>
          </div>

          <div className="flex-1" />

          {/* Buttons */}
          <div className="flex items-center gap-2">
            <button
              onClick={resetConfig}
              className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-xs text-txt-secondary hover:text-txt-primary bg-bg-elevated hover:bg-bg-base transition-all cursor-pointer"
            >
              <RotateCcw size={12} />
              重置
            </button>
            <button
              onClick={runBacktest}
              disabled={running || config.codes.length === 0}
              className={`flex items-center gap-1.5 px-4 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer ${
                running
                  ? 'bg-bg-elevated text-txt-muted cursor-not-allowed'
                  : 'bg-gradient-to-r from-primary-gold to-yellow-600 text-bg-base hover:from-yellow-500 hover:to-yellow-600 shadow-lg shadow-yellow-500/20 disabled:opacity-40 disabled:cursor-not-allowed'
              }`}
            >
              {running ? <Loader2 size={13} className="animate-spin" /> : <Play size={13} />}
              {running ? '回测中...' : '开始回测'}
            </button>
          </div>
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="px-4 py-2 bg-red-500/10 border-b border-red-500/20 text-xs text-functional-up flex items-center justify-between">
          <span>{error}</span>
          <button onClick={clearResult} className="text-txt-muted hover:text-txt-primary cursor-pointer"><X size={14} /></button>
        </div>
      )}

      {/* Main content */}
      <div className="flex-1 min-h-0 overflow-auto">
        {!result && !running ? (
          /* Empty state */
          <div className="flex flex-col items-center justify-center h-full text-txt-muted">
            <div className="relative mb-4">
              <BarChart2 size={56} className="opacity-15" />
              <div className="absolute -bottom-1 -right-1 w-6 h-6 rounded-full bg-primary-gold/20 flex items-center justify-center">
                <Play size={12} className="text-primary-gold ml-0.5" />
              </div>
            </div>
            <p className="text-base font-bold text-txt-primary mb-1">策略回测系统</p>
            <p className="text-xs text-txt-muted mb-4">选择回测标的和时间范围，验证您的量化策略表现</p>
            <div className="flex gap-4 text-[10px] text-txt-muted">
              <span>📈 收益曲线对比</span>
              <span>📊 绩效指标分析</span>
              <span>📋 交易明细记录</span>
            </div>
          </div>
        ) : running ? (
          /* Loading */
          <div className="flex flex-col items-center justify-center h-full">
            <Loader2 size={36} className="animate-spin text-primary-gold mb-3" />
            <p className="text-sm text-txt-primary font-medium">正在执行回测...</p>
            <p className="text-xs text-txt-muted mt-1">拉取历史数据 → 计算技术指标 → 模拟交易</p>
          </div>
        ) : result ? (
          <div className="flex flex-col h-full">
            {/* Performance metrics */}
            <div className="grid grid-cols-6 gap-3 p-4">
              {metrics.map((m) => (
                <MetricCard key={m.label} {...m} />
              ))}
            </div>

            {/* Chart */}
            <div className="flex-1 min-h-[300px] px-4">
              <BacktestChart equityCurve={result.equity_curve} />
            </div>

            {/* Trades toggle */}
            <div className="border-t border-[#30363D]">
              <button
                onClick={() => setShowTrades(!showTrades)}
                className="flex items-center gap-2 px-4 py-2 text-xs text-txt-secondary hover:text-txt-primary transition-colors cursor-pointer w-full"
              >
                <span>{showTrades ? '▼' : '▶'}</span>
                交易明细 ({result.trades.length}笔)
              </button>

              {showTrades && (
                <div className="max-h-[280px] overflow-auto px-4 pb-3">
                  <table className="w-full text-xs">
                    <thead className="sticky top-0 bg-bg-card">
                      <tr className="text-txt-muted text-left border-b border-[#30363D]">
                        <th className="py-1.5 pr-2">#</th>
                        <th className="py-1.5 pr-2">代码</th>
                        <th className="py-1.5 pr-2">名称</th>
                        <th className="py-1.5 pr-2">方向</th>
                        <th className="py-1.5 pr-2">开仓日期</th>
                        <th className="py-1.5 pr-2">开仓价</th>
                        <th className="py-1.5 pr-2">平仓日期</th>
                        <th className="py-1.5 pr-2">平仓价</th>
                        <th className="py-1.5 pr-2">数量</th>
                        <th className="py-1.5 pr-2">盈亏</th>
                        <th className="py-1.5 pr-2">盈亏%</th>
                        <th className="py-1.5">持仓天数</th>
                      </tr>
                    </thead>
                    <tbody>
                      {result.trades.map((t) => {
                        const profitColor = t.profit > 0 ? 'text-functional-up' : t.profit < 0 ? 'text-functional-down' : 'text-txt-primary';
                        return (
                          <tr key={t.id} className="border-b border-[#30363D]/30 hover:bg-bg-elevated/30">
                            <td className="py-1.5 pr-2 text-txt-muted">{t.id}</td>
                            <td className="py-1.5 pr-2 font-mono text-txt-secondary">{t.code}</td>
                            <td className="py-1.5 pr-2 text-txt-primary">{t.name}</td>
                            <td className="py-1.5 pr-2">
                              <span className={`px-1.5 py-0.5 rounded text-[10px] font-medium ${
                                t.direction === 'buy' ? 'bg-red-500/20 text-functional-up' : 'bg-green-500/20 text-functional-down'
                              }`}>
                                {t.direction === 'buy' ? '买入' : '卖出'}
                              </span>
                            </td>
                            <td className="py-1.5 pr-2 text-txt-secondary">{t.open_date}</td>
                            <td className="py-1.5 pr-2 font-mono">{t.open_price.toFixed(2)}</td>
                            <td className="py-1.5 pr-2 text-txt-secondary">{t.close_date}</td>
                            <td className="py-1.5 pr-2 font-mono">{t.close_price.toFixed(2)}</td>
                            <td className="py-1.5 pr-2 font-mono text-txt-secondary">{t.shares}</td>
                            <td className={`py-1.5 pr-2 font-mono font-medium ${profitColor}`}>
                              {t.profit > 0 ? '+' : ''}{t.profit.toFixed(2)}
                            </td>
                            <td className={`py-1.5 pr-2 font-mono ${profitColor}`}>
                              {t.profit_pct > 0 ? '+' : ''}{(t.profit_pct * 100).toFixed(2)}%
                            </td>
                            <td className="py-1.5 text-txt-secondary">{t.holding_days}天</td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
}
