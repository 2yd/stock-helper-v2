import { useState, useEffect, useRef, useCallback } from 'react';
import { Sparkles, ArrowLeft, Activity, Loader2, Check, RefreshCw, Brain, Zap, ChevronRight, ChevronDown, ChevronUp, Star, TrendingUp, X, Search, Users, MessageSquare, Eye, Square } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import KlineChart from '../components/KlineChart';
import TechnicalPanel from '../components/TechnicalPanel';
import WatchlistDiagnosePanel from '../components/WatchlistDiagnosePanel';
import { useAIPickStore } from '../stores/aiPickStore';
import { useWatchlistStore } from '../stores/watchlistStore';
import { useTrackingStore } from '../stores/trackingStore';
import { AIPickRecommendation } from '../types';
import { safeInvoke as invoke } from '../hooks/useTauri';

type ViewMode = 'list' | 'detail';

const RATING_CONFIG: Record<string, { label: string; color: string; bg: string; border: string }> = {
  strong_buy: { label: '强烈推荐', color: 'text-red-400', bg: 'bg-red-500/10', border: 'border-red-500/20' },
  buy: { label: '推荐买入', color: 'text-orange-400', bg: 'bg-orange-500/10', border: 'border-orange-500/20' },
  watch: { label: '建议关注', color: 'text-cyan-400', bg: 'bg-cyan-500/10', border: 'border-cyan-500/20' },
};

export default function AIPick() {
  const {
    picking, aiContent, recommendations, toolCalls, thinkingSteps, error, tokenUsage,
    startPick, stopPick, loadCachedPicks, reset,
    similarLoading, similarTarget, similarContent, similarPicks, similarToolCalls, similarThinkingSteps, similarError,
    findSimilarStocks, closeSimilar,
  } = useAIPickStore();

  const {
    analysis, analysisLoading, analysisPeriod,
    showDiagnosePanel,
    loadAnalysis, setPeriod, startDiagnosis, setShowDiagnosePanel, resetDiagnosis,
  } = useWatchlistStore();

  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [activeIndicators, setActiveIndicators] = useState<string[]>(['MA']);
  const [selectedStock, setSelectedStock] = useState<AIPickRecommendation | null>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const unlistenRef = useRef<(() => void) | null>(null);
  const autoScrollRef = useRef(true);

  useEffect(() => {
    loadCachedPicks();
    return () => {
      if (unlistenRef.current) unlistenRef.current();
    };
  }, []);

  useEffect(() => {
    if (contentRef.current && picking && autoScrollRef.current) {
      contentRef.current.scrollTop = contentRef.current.scrollHeight;
    }
  }, [aiContent, toolCalls, picking]);

  useEffect(() => {
    if (!picking) {
      autoScrollRef.current = true;
    }
  }, [picking]);

  const handleLeftPanelScroll = useCallback(() => {
    const el = contentRef.current;
    if (!el) return;
    const distanceToBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    autoScrollRef.current = distanceToBottom < 48;
  }, []);

  const handleStartPick = useCallback(async () => {
    if (picking) return;
    reset();
    await startPick();
  }, [picking, reset, startPick]);

  const handleStopPick = useCallback(async () => {
    await stopPick();
  }, [stopPick]);

  const handleStockClick = useCallback((rec: AIPickRecommendation) => {
    setSelectedStock(rec);
    loadAnalysis(rec.code, rec.name);
    setViewMode('detail');
  }, [loadAnalysis]);

  const handleBackToList = useCallback(() => {
    setViewMode('list');
    setSelectedStock(null);
  }, []);

  const handleDiagnose = useCallback(async () => {
    if (!analysis) return;
    if (unlistenRef.current) unlistenRef.current();
    resetDiagnosis();
    const unlisten = await startDiagnosis(analysis.code, analysis.name);
    unlistenRef.current = unlisten;
  }, [analysis, startDiagnosis, resetDiagnosis]);

  const handleFindSimilar = useCallback((e: React.MouseEvent, rec: AIPickRecommendation) => {
    e.stopPropagation();
    if (similarLoading) return;
    findSimilarStocks(rec.code, rec.name, rec.sector || '');
  }, [similarLoading, findSimilarStocks]);

  const { addTracking, trackingStocks } = useTrackingStore();

  const handleAddTracking = useCallback(async (e: React.MouseEvent, rec: AIPickRecommendation) => {
    e.stopPropagation();
    let price = rec.price || 0;
    // 如果 AI 没返回价格，通过行情接口获取实时价格
    if (!price) {
      try {
        const snapshots = await invoke<{ code: string; price: number }[]>('get_watchlist_enriched', { codes: [rec.code] });
        if (snapshots && snapshots.length > 0 && snapshots[0].price > 0) {
          price = snapshots[0].price;
        }
      } catch { /* ignore */ }
    }
    await addTracking(
      rec.code,
      rec.name,
      price,
      rec.rating,
      rec.reason,
      rec.sector || '',
    );
  }, [addTracking]);

  const displayContent = aiContent
    .replace(/<PICKS>[\s\S]*?<\/PICKS>/g, '')  // 清理完整闭合的 <PICKS> 块
    .replace(/<PICKS>[\s\S]*/g, '')             // 清理未闭合的 <PICKS>（输出截断时）
    .replace(/<[｜\uff5c][^>]*>[\s\S]*/g, '')   // 清理 DSML 标记
    .trim();
  const hasContent = !!(aiContent || toolCalls.length > 0 || picking);

  return (
    <div className="flex flex-col h-full overflow-hidden relative">
      {/* Top Toolbar */}
      <div className="flex items-center gap-3 px-4 py-2 bg-bg-elevated/50 border-b border-[#30363D] flex-shrink-0">
        {viewMode === 'detail' ? (
          <>
            <button
              onClick={handleBackToList}
              className="flex items-center gap-1.5 text-xs text-txt-secondary hover:text-cyan-400 transition-colors cursor-pointer"
            >
              <ArrowLeft size={14} />
              <span>返回列表</span>
            </button>
            {selectedStock && (
              <div className="flex items-center gap-2">
                <span className="text-sm font-bold text-txt-primary">{selectedStock.name}</span>
                <span className="text-xs text-txt-muted font-mono">{selectedStock.code}</span>
                {selectedStock.change_pct !== undefined && (
                  <span className={`font-mono text-xs ${selectedStock.change_pct > 0 ? 'text-functional-up' : selectedStock.change_pct < 0 ? 'text-functional-down' : 'text-txt-primary'}`}>
                    {selectedStock.change_pct > 0 ? '+' : ''}{selectedStock.change_pct.toFixed(2)}%
                  </span>
                )}
              </div>
            )}
            <div className="flex-1" />
            {analysis && (
              <button
                onClick={handleDiagnose}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-gradient-to-r from-cyan-600/80 to-indigo-600/80 text-white hover:from-cyan-500 hover:to-indigo-500 shadow-lg shadow-cyan-500/10 transition-all cursor-pointer"
              >
                <Activity size={13} />
                AI 诊断
              </button>
            )}
          </>
        ) : (
          <>
            <Brain size={15} className="text-cyan-400" />
            <h2 className="text-sm font-semibold text-txt-primary">AI 自主选股</h2>
            {recommendations.length > 0 && (
              <span className="text-[11px] px-2 py-0.5 rounded-full bg-cyan-500/10 text-cyan-400 border border-cyan-500/15">
                {recommendations.length} 只推荐
              </span>
            )}
            <div className="flex-1" />
            {tokenUsage && (
              <span className="text-[10px] text-txt-muted font-mono opacity-50">
                {tokenUsage.toLocaleString()} tokens
              </span>
            )}
            <button
              onClick={handleStartPick}
              disabled={picking}
              className={`flex items-center gap-1.5 px-4 py-1.5 rounded-md text-xs font-medium transition-all cursor-pointer ${
                picking
                  ? 'bg-cyan-900/30 text-cyan-400/70 border border-cyan-700/30 cursor-not-allowed'
                  : 'bg-cyan-600 text-white hover:bg-cyan-500 active:bg-cyan-700'
              }`}
            >
              {picking ? (
                <Loader2 size={12} className="animate-spin" />
              ) : (
                <Sparkles size={12} />
              )}
              {picking ? '分析中...' : '开始选股'}
            </button>
            {picking && (
              <button
                onClick={handleStopPick}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium bg-red-600/80 text-white hover:bg-red-500 active:bg-red-700 transition-all cursor-pointer"
              >
                <Square size={10} fill="currentColor" />
                停止
              </button>
            )}
          </>
        )}
      </div>

      {/* ===== LIST VIEW ===== */}
      {viewMode === 'list' && (
        <div className="flex-1 flex overflow-hidden">
          {/* Left: AI Analysis Panel — independent scrollable area */}
          {hasContent && (
            <LeftAnalysisPanel
              contentRef={contentRef}
              onScroll={handleLeftPanelScroll}
              toolCalls={toolCalls}
              thinkingSteps={thinkingSteps}
              displayContent={displayContent}
              picking={picking}
            />
          )}

          {/* Right: Recommendations / Empty State — independent scrollable area */}
          <RightRecommendationPanel
            recommendations={recommendations}
            error={error}
            similarLoading={similarLoading}
            similarTarget={similarTarget}
            onStockClick={handleStockClick}
            onFindSimilar={handleFindSimilar}
            onStartPick={handleStartPick}
            onAddTracking={handleAddTracking}
            trackingStocks={trackingStocks}
          />
        </div>
      )}

      {/* ===== DETAIL VIEW ===== */}
      {viewMode === 'detail' && (
        <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
          {analysisLoading ? (
            <div className="flex-1 flex items-center justify-center gap-2">
              <Loader2 size={20} className="animate-spin text-cyan-400" />
              <span className="text-xs text-txt-muted">加载K线和技术指标...</span>
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
              <p className="text-sm text-txt-muted">加载中...</p>
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

      {/* Similar Stocks Panel */}
      {similarTarget && (
        <SimilarStocksPanel
          target={similarTarget}
          loading={similarLoading}
          content={similarContent}
          picks={similarPicks}
          toolCalls={similarToolCalls}
          thinkingSteps={similarThinkingSteps}
          error={similarError}
          onClose={closeSimilar}
          onStockClick={handleStockClick}
        />
      )}
    </div>
  );
}

/* ========== Thinking Steps Card (collapsible, show last 3) ========== */

function ThinkingStepsCard({ steps, accentColor = 'cyan' }: { steps: { content: string; timestamp: number }[]; accentColor?: 'cyan' | 'indigo' }) {
  const [expanded, setExpanded] = useState(false);
  if (steps.length === 0) return null;

  const displaySteps = expanded ? steps : steps.slice(-3);
  const hasMore = steps.length > 3;
  const colors = accentColor === 'cyan'
    ? { border: 'border-cyan-500/20', bg: 'bg-cyan-500/5', icon: 'text-cyan-400', text: 'text-cyan-300/80', toggle: 'text-cyan-400/60 hover:text-cyan-400' }
    : { border: 'border-indigo-500/20', bg: 'bg-indigo-500/5', icon: 'text-indigo-400', text: 'text-indigo-300/80', toggle: 'text-indigo-400/60 hover:text-indigo-400' };

  return (
    <div className={`mx-3 my-2 rounded-lg border ${colors.border} ${colors.bg} overflow-hidden`}>
      <div className="flex items-center gap-1.5 px-3 py-1.5">
        <MessageSquare size={11} className={colors.icon} />
        <span className={`text-[10px] font-medium ${colors.icon}`}>AI 思考过程</span>
        <span className="text-[10px] text-txt-muted/50 ml-auto">{steps.length} 步</span>
      </div>
      <div className="px-3 pb-2 space-y-1.5">
        {!expanded && hasMore && (
          <button onClick={() => setExpanded(true)} className={`text-[10px] ${colors.toggle} transition-colors flex items-center gap-1 cursor-pointer`}>
            <ChevronUp size={10} />
            查看全部 {steps.length} 步
          </button>
        )}
        {displaySteps.map((step, i) => (
          <div key={step.timestamp + i} className="text-[11px] text-txt-secondary leading-relaxed">
            <span className={`${colors.text} font-medium`}>#{expanded ? i + 1 : steps.length - displaySteps.length + i + 1}:</span>{' '}
            {step.content}
          </div>
        ))}
        {expanded && hasMore && (
          <button onClick={() => setExpanded(false)} className={`text-[10px] ${colors.toggle} transition-colors flex items-center gap-1 cursor-pointer`}>
            <ChevronDown size={10} />
            收起
          </button>
        )}
      </div>
    </div>
  );
}

/* ========== Tool Call Item (expandable) ========== */

function ToolCallItem({ tool }: { tool: { name: string; label: string; status: string; summary?: string } }) {
  const [expanded, setExpanded] = useState(false);
  const hasSummary = tool.status === 'done' && tool.summary;

  return (
    <div>
      <div
        className={`flex items-center gap-2 py-1 ${hasSummary ? 'cursor-pointer hover:bg-bg-elevated/30 -mx-1 px-1 rounded' : ''}`}
        onClick={() => hasSummary && setExpanded(!expanded)}
      >
        {tool.status === 'done' ? (
          <Check size={11} className="text-green-400 flex-shrink-0" />
        ) : (
          <Loader2 size={11} className="text-cyan-400 animate-spin flex-shrink-0" />
        )}
        <span className={`text-[11px] flex-1 ${tool.status === 'done' ? 'text-txt-muted' : 'text-cyan-400'}`}>
          {tool.label}
        </span>
        {hasSummary && (
          <ChevronDown
            size={11}
            className={`text-txt-muted/40 flex-shrink-0 transition-transform ${expanded ? 'rotate-180' : ''}`}
          />
        )}
      </div>
      {expanded && tool.summary && (
        <div className="ml-5 mb-1.5 px-2 py-1.5 rounded bg-bg-elevated/40 border-l-2 border-cyan-500/20">
          <pre className="text-[10px] text-txt-muted leading-relaxed whitespace-pre-wrap font-mono break-all">
            {tool.summary}
          </pre>
        </div>
      )}
    </div>
  );
}

/* ========== Empty State ========== */

function EmptyState({ onStart }: { onStart: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center h-full text-txt-muted px-8">
      <div className="mb-6 relative">
        <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-cyan-500/5 to-indigo-500/5 border border-[#30363D] flex items-center justify-center">
          <Brain size={28} className="text-cyan-400/30" />
        </div>
      </div>
      <h3 className="text-base font-semibold text-txt-primary mb-2">AI 自主选股</h3>
      <p className="text-xs text-txt-muted text-center max-w-xs leading-relaxed mb-6">
        AI 将自主阅读市场新闻，研判热点板块方向，综合基本面筛选优质个股
      </p>
      <div className="flex items-center gap-6 text-[11px] text-txt-muted/60 mb-6">
        <div className="flex items-center gap-1.5">
          <Zap size={11} />
          <span>消息面分析</span>
        </div>
        <div className="flex items-center gap-1.5">
          <TrendingUp size={11} />
          <span>板块研判</span>
        </div>
        <div className="flex items-center gap-1.5">
          <Star size={11} />
          <span>个股精选</span>
        </div>
      </div>
      <button
        onClick={onStart}
        className="flex items-center gap-2 px-5 py-2 rounded-md text-sm font-medium bg-cyan-600 text-white hover:bg-cyan-500 active:bg-cyan-700 transition-colors cursor-pointer"
      >
        <Sparkles size={14} />
        开始选股
      </button>
    </div>
  );
}

/* ========== Left Analysis Panel (independent scroll) ========== */

function LeftAnalysisPanel({
  contentRef,
  onScroll,
  toolCalls,
  thinkingSteps,
  displayContent,
  picking,
}: {
  contentRef: React.RefObject<HTMLDivElement | null>;
  onScroll: () => void;
  toolCalls: { name: string; label: string; status: string; summary?: string }[];
  thinkingSteps: { content: string; timestamp: number }[];
  displayContent: string;
  picking: boolean;
}) {
  return (
    <div
      ref={contentRef as React.RefObject<HTMLDivElement>}
      onScroll={onScroll}
      className="w-[380px] h-full flex-shrink-0 border-r border-[#30363D] overflow-y-auto"
    >
      {/* Tool Call Status */}
      {toolCalls.length > 0 && (
        <div className="px-3 py-2 border-b border-[#30363D]/60">
          <div className="space-y-0.5">
            {toolCalls.map((tool, i) => (
              <ToolCallItem key={i} tool={tool} />
            ))}
          </div>
        </div>
      )}

      {/* Thinking Steps */}
      <ThinkingStepsCard steps={thinkingSteps} accentColor="cyan" />

      {/* Streaming Markdown Content */}
      <div className="px-3 py-3">
        {displayContent ? (
          <div className="prose prose-invert prose-sm max-w-none
            prose-headings:text-txt-primary prose-headings:font-semibold prose-headings:mb-2 prose-headings:mt-4
            prose-h2:text-[15px] prose-h3:text-[13px]
            prose-p:text-txt-secondary prose-p:text-[12px] prose-p:leading-relaxed prose-p:my-1.5
            prose-strong:text-cyan-300 prose-strong:font-medium
            prose-li:text-txt-secondary prose-li:text-[12px]
            prose-ul:my-1 prose-ol:my-1
            prose-code:text-cyan-300 prose-code:text-[11px] prose-code:bg-cyan-900/20 prose-code:px-1 prose-code:py-0.5 prose-code:rounded
            prose-table:text-[11px]
            prose-th:text-txt-primary prose-th:font-medium prose-th:py-1 prose-th:px-2
            prose-td:text-txt-secondary prose-td:py-1 prose-td:px-2
          ">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{displayContent}</ReactMarkdown>
            {picking && <span className="inline-block w-1.5 h-3.5 bg-cyan-400/80 animate-pulse ml-0.5 rounded-sm" />}
          </div>
        ) : picking ? (
          <div className="flex items-center gap-2 py-2 text-txt-muted">
            <Loader2 size={14} className="animate-spin text-cyan-400" />
            <span className="text-xs">AI 正在获取数据并分析...</span>
          </div>
        ) : null}
      </div>
    </div>
  );
}

/* ========== Right Recommendation Panel (independent scroll) ========== */

function RightRecommendationPanel({
  recommendations,
  error,
  similarLoading,
  similarTarget,
  onStockClick,
  onFindSimilar,
  onStartPick,
  onAddTracking,
  trackingStocks,
}: {
  recommendations: AIPickRecommendation[];
  error: string | null;
  similarLoading: boolean;
  similarTarget: { code: string; name: string; sector: string } | null;
  onStockClick: (rec: AIPickRecommendation) => void;
  onFindSimilar: (e: React.MouseEvent, rec: AIPickRecommendation) => void;
  onStartPick: () => void;
  onAddTracking: (e: React.MouseEvent, rec: AIPickRecommendation) => void;
  trackingStocks: { code: string; added_date: string }[];
}) {
  const today = new Date().toISOString().slice(0, 10);
  return (
    <div className="flex-1 h-full min-w-0 overflow-y-auto">
      {recommendations.length > 0 ? (
        <div className="p-3">
          <div className="space-y-2">
            {recommendations.map((rec, i) => {
              const alreadyTracked = trackingStocks.some(
                (t) => t.code === rec.code && t.added_date === today,
              );
              return (
                <RecommendationCard
                  key={rec.code || i}
                  rec={rec}
                  index={i}
                  onClick={() => onStockClick(rec)}
                  onFindSimilar={(e) => onFindSimilar(e, rec)}
                  similarLoading={similarLoading && similarTarget?.code === rec.code}
                  onAddTracking={(e) => onAddTracking(e, rec)}
                  alreadyTracked={alreadyTracked}
                />
              );
            })}
          </div>
        </div>
      ) : error ? (
        <div className="flex flex-col items-center justify-center h-full text-txt-muted px-8">
          <div className="w-10 h-10 rounded-full bg-red-500/10 flex items-center justify-center mb-3">
            <Sparkles size={18} className="text-red-400" />
          </div>
          <p className="text-sm font-medium text-red-400 mb-1">分析失败</p>
          <p className="text-[11px] opacity-60 max-w-sm text-center leading-relaxed">{error}</p>
          <button
            onClick={onStartPick}
            className="mt-4 flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs text-cyan-400 bg-cyan-500/10 hover:bg-cyan-500/20 transition-colors cursor-pointer"
          >
            <RefreshCw size={11} />
            重试
          </button>
        </div>
      ) : (
        <EmptyState onStart={onStartPick} />
      )}
    </div>
  );
}

/* ========== Recommendation Card ========== */

function RecommendationCard({
  rec,
  index,
  onClick,
  onFindSimilar,
  similarLoading,
  onAddTracking,
  alreadyTracked,
}: {
  rec: AIPickRecommendation;
  index: number;
  onClick: () => void;
  onFindSimilar: (e: React.MouseEvent) => void;
  similarLoading?: boolean;
  onAddTracking: (e: React.MouseEvent) => void;
  alreadyTracked?: boolean;
}) {
  const ratingCfg = RATING_CONFIG[rec.rating] || RATING_CONFIG.watch;
  const changePct = rec.change_pct;
  const changeColor = changePct !== undefined
    ? changePct > 0 ? 'text-functional-up' : changePct < 0 ? 'text-functional-down' : 'text-txt-primary'
    : 'text-txt-muted';

  return (
    <div
      onClick={onClick}
      className="flex items-start gap-3 px-3 py-2.5 rounded-lg border border-[#30363D]/60 hover:border-[#30363D] hover:bg-bg-elevated/30 transition-all cursor-pointer group"
    >
      {/* Index */}
      <div className="w-5 h-5 rounded flex items-center justify-center bg-bg-elevated/60 text-[10px] font-mono text-txt-muted flex-shrink-0 mt-0.5">
        {index + 1}
      </div>

      {/* Main Info */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <span className="text-sm font-bold text-txt-primary">{rec.name}</span>
          <span className="text-[11px] font-mono text-txt-muted">{rec.code}</span>
          {rec.price !== undefined && (
            <span className="text-[11px] font-mono text-txt-secondary ml-auto">
              {rec.price.toFixed(2)}
            </span>
          )}
          {changePct !== undefined && (
            <span className={`text-[11px] font-mono ${changeColor}`}>
              {changePct > 0 ? '+' : ''}{changePct.toFixed(2)}%
            </span>
          )}
        </div>

        <p className="text-[11px] text-txt-secondary leading-relaxed line-clamp-2 mb-1.5">
          {rec.reason}
        </p>

        <div className="flex items-center gap-1.5 flex-wrap">
          <span className={`inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium ${ratingCfg.color} ${ratingCfg.bg} border ${ratingCfg.border}`}>
            {ratingCfg.label}
          </span>
          {rec.sector && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-[#282E36] text-[#9CA3AF] border border-[#3B424D]">
              {rec.sector}
            </span>
          )}
          {rec.highlights?.map((h, hi) => (
            <span key={hi} className="text-[10px] px-1.5 py-0.5 rounded bg-cyan-500/5 text-cyan-400/70 border border-cyan-500/10">
              {h}
            </span>
          ))}
          {/* Find Similar Button */}
          <button
            onClick={onFindSimilar}
            disabled={similarLoading}
            className="ml-auto inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-medium text-indigo-400 bg-indigo-500/10 border border-indigo-500/20 hover:bg-indigo-500/20 transition-colors opacity-0 group-hover:opacity-100 cursor-pointer disabled:opacity-50"
          >
            {similarLoading ? <Loader2 size={9} className="animate-spin" /> : <Users size={9} />}
            找相似股
          </button>
          {/* Add to Tracking Button */}
          <button
            onClick={onAddTracking}
            disabled={alreadyTracked}
            className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-medium transition-colors cursor-pointer ${
              alreadyTracked
                ? 'text-green-400 bg-green-500/10 border border-green-500/20 opacity-70'
                : 'text-amber-400 bg-amber-500/10 border border-amber-500/20 hover:bg-amber-500/20 opacity-0 group-hover:opacity-100'
            }`}
          >
            {alreadyTracked ? <Check size={9} /> : <Eye size={9} />}
            {alreadyTracked ? '已盯盘' : '加入盯盘'}
          </button>
        </div>
      </div>

      {/* Arrow */}
      <ChevronRight size={14} className="text-txt-muted/30 group-hover:text-txt-muted flex-shrink-0 mt-1 transition-colors" />
    </div>
  );
}

/* ========== Similar Stocks Panel ========== */

function SimilarStocksPanel({
  target,
  loading,
  content,
  picks,
  toolCalls,
  thinkingSteps,
  error,
  onClose,
  onStockClick,
}: {
  target: { code: string; name: string; sector: string };
  loading: boolean;
  content: string;
  picks: AIPickRecommendation[];
  toolCalls: { name: string; label: string; status: string; summary?: string }[];
  thinkingSteps: { content: string; timestamp: number }[];
  error: string | null;
  onClose: () => void;
  onStockClick: (rec: AIPickRecommendation) => void;
}) {
  const panelRef = useRef<HTMLDivElement>(null);

  const displayContent = content
    .replace(/<PICKS>[\s\S]*?<\/PICKS>/g, '')
    .replace(/<[｜\uff5c][^>]*>[\s\S]*/g, '')
    .trim();

  useEffect(() => {
    if (panelRef.current && loading) {
      panelRef.current.scrollTop = panelRef.current.scrollHeight;
    }
  }, [content, toolCalls, loading]);

  return (
    <div className="absolute inset-0 z-50 flex">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/40" onClick={onClose} />

      {/* Panel slides in from right */}
      <div className="absolute right-0 top-0 bottom-0 w-[480px] bg-bg-card border-l border-[#30363D] flex flex-col shadow-2xl animate-in slide-in-from-right">
        {/* Header */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-[#30363D] flex-shrink-0">
          <Users size={15} className="text-indigo-400" />
          <div className="flex-1 min-w-0">
            <h3 className="text-sm font-semibold text-txt-primary">
              找相似股 — {target.name}
            </h3>
            <p className="text-[10px] text-txt-muted mt-0.5">
              {target.code} {target.sector ? `· ${target.sector}` : ''}
            </p>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-bg-elevated transition-colors cursor-pointer"
          >
            <X size={14} className="text-txt-muted" />
          </button>
        </div>

        {/* Tool Calls */}
        {toolCalls.length > 0 && (
          <div className="px-4 py-2 border-b border-[#30363D]/60 flex-shrink-0">
            <div className="space-y-0.5">
              {toolCalls.map((tool, i) => (
                <ToolCallItem key={i} tool={tool} />
              ))}
            </div>
          </div>
        )}

        {/* Thinking Steps */}
        {thinkingSteps.length > 0 && (
          <div className="flex-shrink-0">
            <ThinkingStepsCard steps={thinkingSteps} accentColor="indigo" />
          </div>
        )}

        {/* Content */}
        <div ref={panelRef} className="flex-1 overflow-y-auto min-h-0">
          {/* Similar Picks */}
          {picks.length > 0 && (
            <div className="px-4 py-3 border-b border-[#30363D]/40">
              <h4 className="text-xs font-semibold text-indigo-400 mb-2 flex items-center gap-1.5">
                <Search size={11} />
                补涨机会 · {picks.length} 只
              </h4>
              <div className="space-y-2">
                {picks.map((rec, i) => {
                  const ratingCfg = RATING_CONFIG[rec.rating] || RATING_CONFIG.watch;
                  const pct = rec.change_pct;
                  const pctColor = pct !== undefined
                    ? pct > 0 ? 'text-functional-up' : pct < 0 ? 'text-functional-down' : 'text-txt-primary'
                    : 'text-txt-muted';
                  return (
                    <div
                      key={rec.code || i}
                      onClick={() => onStockClick(rec)}
                      className="px-3 py-2 rounded-lg border border-indigo-500/15 hover:border-indigo-500/30 hover:bg-indigo-500/5 transition-all cursor-pointer"
                    >
                      <div className="flex items-center gap-2 mb-1">
                        <span className="text-[10px] font-mono text-indigo-400/60 w-4">{i + 1}</span>
                        <span className="text-sm font-bold text-txt-primary">{rec.name}</span>
                        <span className="text-[11px] font-mono text-txt-muted">{rec.code}</span>
                        {rec.price !== undefined && (
                          <span className="text-[11px] font-mono text-txt-secondary ml-auto">{rec.price.toFixed(2)}</span>
                        )}
                        {pct !== undefined && (
                          <span className={`text-[11px] font-mono ${pctColor}`}>
                            {pct > 0 ? '+' : ''}{pct.toFixed(2)}%
                          </span>
                        )}
                      </div>
                      <p className="text-[11px] text-txt-secondary leading-relaxed line-clamp-2 ml-6 mb-1">
                        {rec.reason}
                      </p>
                      <div className="flex items-center gap-1.5 flex-wrap ml-6">
                        <span className={`inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium ${ratingCfg.color} ${ratingCfg.bg} border ${ratingCfg.border}`}>
                          {ratingCfg.label}
                        </span>
                        {rec.highlights?.map((h, hi) => (
                          <span key={hi} className="text-[10px] px-1.5 py-0.5 rounded bg-indigo-500/5 text-indigo-400/70 border border-indigo-500/10">
                            {h}
                          </span>
                        ))}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* AI Analysis Content */}
          {displayContent && (
            <div className="px-4 py-3">
              <div className="prose prose-invert prose-sm max-w-none
                prose-headings:text-txt-primary prose-headings:font-semibold prose-headings:mb-2 prose-headings:mt-3
                prose-h2:text-[14px] prose-h3:text-[12px]
                prose-p:text-txt-secondary prose-p:text-[11px] prose-p:leading-relaxed prose-p:my-1
                prose-strong:text-indigo-300 prose-strong:font-medium
                prose-li:text-txt-secondary prose-li:text-[11px]
                prose-ul:my-1 prose-ol:my-1
              ">
                <ReactMarkdown remarkPlugins={[remarkGfm]}>{displayContent}</ReactMarkdown>
                {loading && <span className="inline-block w-1.5 h-3.5 bg-indigo-400/80 animate-pulse ml-0.5 rounded-sm" />}
              </div>
            </div>
          )}

          {/* Loading state */}
          {loading && !displayContent && toolCalls.length === 0 && (
            <div className="flex items-center justify-center gap-2 py-8">
              <Loader2 size={16} className="animate-spin text-indigo-400" />
              <span className="text-xs text-txt-muted">正在分析相似个股...</span>
            </div>
          )}

          {/* Error */}
          {error && (
            <div className="px-4 py-6 text-center">
              <p className="text-xs text-red-400">{error}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
