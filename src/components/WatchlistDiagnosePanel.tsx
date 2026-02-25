import { useEffect, useRef } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { X, Copy, Check, RefreshCw, Database, BarChart3, TrendingUp, DollarSign } from 'lucide-react';
import { useState } from 'react';
import { useWatchlistStore } from '../stores/watchlistStore';

interface Props {
  code: string;
  name: string;
  onClose: () => void;
}

const toolIcons: Record<string, typeof Database> = {
  get_stock_quote: DollarSign,
  get_kline_data: BarChart3,
  get_technical_indicators: TrendingUp,
  get_fund_flow: Database,
};

const toolLabels: Record<string, string> = {
  get_stock_quote: '实时行情',
  get_kline_data: 'K线数据',
  get_technical_indicators: '技术指标',
  get_fund_flow: '资金流向',
};

export default function WatchlistDiagnosePanel({ code, name, onClose }: Props) {
  const { diagnosing, diagnoseContent, diagnoseDone, diagnoseToolCalls } = useWatchlistStore();
  const [copied, setCopied] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (contentRef.current) {
      contentRef.current.scrollTop = contentRef.current.scrollHeight;
    }
  }, [diagnoseContent, diagnoseToolCalls]);

  const handleCopy = () => {
    navigator.clipboard.writeText(diagnoseContent);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const showToolProgress = diagnosing && diagnoseToolCalls.length > 0 && !diagnoseContent;

  return (
    <div className="fixed right-0 top-12 bottom-0 w-[420px] bg-bg-card border-l border-[#30363D] flex flex-col z-50 shadow-2xl shadow-black/50 animate-in slide-in-from-right">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-[#30363D]">
        <div className="flex items-center gap-2">
          <span className="font-bold text-txt-primary text-sm">AI 技术诊断</span>
          <span className="text-xs text-txt-muted">{name}</span>
          <span className="text-[10px] text-txt-muted font-mono">{code}</span>
        </div>
        <button onClick={onClose} className="p-1 rounded hover:bg-bg-elevated transition-colors cursor-pointer">
          <X size={18} className="text-txt-secondary" />
        </button>
      </div>

      {/* Tool Calls Progress */}
      {diagnoseToolCalls.length > 0 && (
        <div className="px-4 py-2 border-b border-[#30363D] bg-bg-elevated/30">
          <div className="flex items-center gap-1.5 mb-1.5">
            <Database size={12} className="text-primary-gold" />
            <span className="text-[10px] font-medium text-primary-gold uppercase tracking-wider">数据获取</span>
          </div>
          <div className="flex flex-col gap-1">
            {diagnoseToolCalls.map((tool, i) => {
              const Icon = toolIcons[tool.name] || Database;
              const label = toolLabels[tool.name] || tool.name;
              return (
                <div key={i} className="flex items-center gap-1.5">
                  {tool.done ? (
                    <Check size={10} className="text-functional-down flex-shrink-0" />
                  ) : (
                    <RefreshCw size={10} className="text-primary-gold animate-spin flex-shrink-0" />
                  )}
                  <Icon size={10} className={tool.done ? 'text-txt-muted' : 'text-primary-gold'} />
                  <span className={`text-[11px] ${tool.done ? 'text-txt-muted' : 'text-txt-secondary'}`}>
                    {tool.done ? `✓ 已获取${label}` : `正在获取${label}...`}
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Content */}
      <div ref={contentRef} className="flex-1 overflow-auto px-4 py-3">
        {diagnoseContent ? (
          <div className="prose prose-invert prose-sm max-w-none">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{diagnoseContent}</ReactMarkdown>
            {diagnosing && <span className="inline-block w-2 h-4 bg-primary-gold animate-pulse ml-0.5" />}
          </div>
        ) : showToolProgress ? (
          <div className="flex flex-col items-center justify-center h-full text-txt-muted">
            <RefreshCw size={24} className="animate-spin mb-3 text-primary-gold" />
            <p className="text-sm">正在获取股票数据...</p>
            <p className="text-[11px] mt-1 opacity-60">AI 正在调用工具获取实时数据</p>
          </div>
        ) : diagnosing ? (
          <div className="flex flex-col items-center justify-center h-full text-txt-muted">
            <RefreshCw size={24} className="animate-spin mb-3 text-primary-gold" />
            <p className="text-sm">正在启动 AI 分析...</p>
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-txt-muted">
            <p className="text-sm">AI 将自动获取实时数据进行深度技术分析</p>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2 px-4 py-3 border-t border-[#30363D]">
        {diagnoseDone && diagnoseContent && (
          <button
            onClick={handleCopy}
            className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm bg-bg-elevated text-txt-secondary hover:text-txt-primary transition-colors cursor-pointer"
          >
            {copied ? <Check size={14} className="text-functional-down" /> : <Copy size={14} />}
            {copied ? '已复制' : '复制'}
          </button>
        )}
        {diagnosing && (
          <span className="text-xs text-txt-muted ml-auto">
            {diagnoseToolCalls.length > 0 && !diagnoseContent
              ? `获取数据中 (${diagnoseToolCalls.length}/4)...`
              : '分析中...'
            }
          </span>
        )}
      </div>
    </div>
  );
}
