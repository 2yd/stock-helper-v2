import { useEffect, useRef, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { X, Copy, Check, RefreshCw, Database, BarChart3, TrendingUp, DollarSign, TrendingDown, AlertTriangle, Newspaper, FileText, Search } from 'lucide-react';
import { useTrackingStore } from '../stores/trackingStore';

const toolIcons: Record<string, typeof Database> = {
  get_stock_quote: DollarSign,
  get_kline_data: BarChart3,
  get_technical_indicators: TrendingUp,
  get_fund_flow: Database,
  get_market_news: Newspaper,
  search_stock_news: Search,
  get_stock_notices: FileText,
  get_industry_report: FileText,
  search_concept_boards: Search,
  get_economic_data: BarChart3,
  get_global_indexes: TrendingUp,
  batch_get_stock_quotes: DollarSign,
};

export default function LossAnalysisPanel() {
  const {
    lossAnalyzing,
    lossAnalysisContent,
    lossAnalysisDate,
    lossToolCalls,
    lossThinkingSteps,
    lossError,
    lossDone,
    closeLossAnalysis,
  } = useTrackingStore();

  const [copied, setCopied] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (contentRef.current) {
      contentRef.current.scrollTop = contentRef.current.scrollHeight;
    }
  }, [lossAnalysisContent, lossToolCalls, lossThinkingSteps]);

  const handleCopy = () => {
    navigator.clipboard.writeText(lossAnalysisContent);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const showToolProgress = lossAnalyzing && lossToolCalls.length > 0 && !lossAnalysisContent;

  return (
    <div className="fixed right-0 top-12 bottom-0 w-[480px] bg-bg-card border-l border-[#30363D] flex flex-col z-50 shadow-2xl shadow-black/50 animate-in slide-in-from-right">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-[#30363D]">
        <div className="flex items-center gap-2">
          <TrendingDown size={16} className="text-red-400" />
          <span className="font-bold text-txt-primary text-sm">败因分析</span>
          <span className="text-xs text-txt-muted">{lossAnalysisDate}</span>
        </div>
        <button onClick={closeLossAnalysis} className="p-1 rounded hover:bg-bg-elevated transition-colors cursor-pointer">
          <X size={18} className="text-txt-secondary" />
        </button>
      </div>

      {/* Thinking Steps */}
      {lossThinkingSteps.length > 0 && !lossAnalysisContent && (
        <div className="px-4 py-2 border-b border-[#30363D] bg-bg-elevated/30 max-h-[120px] overflow-auto">
          <div className="flex items-center gap-1.5 mb-1.5">
            <AlertTriangle size={12} className="text-amber-400" />
            <span className="text-[10px] font-medium text-amber-400 uppercase tracking-wider">AI 思考过程</span>
          </div>
          <div className="space-y-1">
            {lossThinkingSteps.map((step, i) => (
              <p key={i} className="text-[11px] text-txt-muted leading-relaxed">{step.content}</p>
            ))}
          </div>
        </div>
      )}

      {/* Tool Calls Progress */}
      {lossToolCalls.length > 0 && (
        <div className="px-4 py-2 border-b border-[#30363D] bg-bg-elevated/30">
          <div className="flex items-center gap-1.5 mb-1.5">
            <Database size={12} className="text-primary-gold" />
            <span className="text-[10px] font-medium text-primary-gold uppercase tracking-wider">数据获取</span>
            <span className="text-[10px] text-txt-muted ml-auto">
              {lossToolCalls.filter(t => t.done).length}/{lossToolCalls.length}
            </span>
          </div>
          <div className="flex flex-col gap-1 max-h-[100px] overflow-auto">
            {lossToolCalls.map((tool, i) => {
              const Icon = toolIcons[tool.name] || Database;
              return (
                <div key={i} className="flex items-center gap-1.5">
                  {tool.done ? (
                    <Check size={10} className="text-green-400 flex-shrink-0" />
                  ) : (
                    <RefreshCw size={10} className="text-primary-gold animate-spin flex-shrink-0" />
                  )}
                  <Icon size={10} className={tool.done ? 'text-txt-muted' : 'text-primary-gold'} />
                  <span className={`text-[11px] ${tool.done ? 'text-txt-muted' : 'text-txt-secondary'}`}>
                    {tool.done ? `✓ ${tool.label}` : `${tool.label}...`}
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Content */}
      <div ref={contentRef} className="flex-1 overflow-auto px-4 py-3">
        {lossError ? (
          <div className="flex flex-col items-center justify-center h-full text-txt-muted">
            <AlertTriangle size={24} className="mb-3 text-red-400" />
            <p className="text-sm text-red-400">{lossError}</p>
          </div>
        ) : lossAnalysisContent ? (
          <div className="prose prose-invert prose-sm max-w-none">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{lossAnalysisContent}</ReactMarkdown>
            {lossAnalyzing && <span className="inline-block w-2 h-4 bg-red-400 animate-pulse ml-0.5" />}
          </div>
        ) : showToolProgress ? (
          <div className="flex flex-col items-center justify-center h-full text-txt-muted">
            <RefreshCw size={24} className="animate-spin mb-3 text-red-400" />
            <p className="text-sm">正在获取股票数据...</p>
            <p className="text-[11px] mt-1 opacity-60">AI 正在调用工具获取亏损股相关数据</p>
          </div>
        ) : lossAnalyzing ? (
          <div className="flex flex-col items-center justify-center h-full text-txt-muted">
            <RefreshCw size={24} className="animate-spin mb-3 text-red-400" />
            <p className="text-sm">正在启动败因分析...</p>
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-txt-muted">
            <p className="text-sm">AI 将自动获取数据进行败因归因分析</p>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2 px-4 py-3 border-t border-[#30363D]">
        {lossDone && lossAnalysisContent && (
          <button
            onClick={handleCopy}
            className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm bg-bg-elevated text-txt-secondary hover:text-txt-primary transition-colors cursor-pointer"
          >
            {copied ? <Check size={14} className="text-green-400" /> : <Copy size={14} />}
            {copied ? '已复制' : '复制'}
          </button>
        )}
        {lossAnalyzing && (
          <span className="text-xs text-txt-muted ml-auto">
            {lossToolCalls.length > 0 && !lossAnalysisContent
              ? `获取数据中 (${lossToolCalls.filter(t => t.done).length}/${lossToolCalls.length})...`
              : '分析中...'
            }
          </span>
        )}
      </div>
    </div>
  );
}
