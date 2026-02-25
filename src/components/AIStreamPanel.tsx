import { useState, useEffect, useRef } from 'react';
import { safeInvoke as invoke, safeListen } from '../hooks/useTauri';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { X, RefreshCw, Copy, Check, History } from 'lucide-react';
import { StrategyResultRow, AIAnalysisResult, AIStreamEvent } from '../types';

interface AIStreamPanelProps {
  stock: StrategyResultRow | null;
  onClose: () => void;
}

export default function AIStreamPanel({ stock, onClose }: AIStreamPanelProps) {
  const [content, setContent] = useState('');
  const [streaming, setStreaming] = useState(false);
  const [copied, setCopied] = useState(false);
  const [history, setHistory] = useState<AIAnalysisResult[]>([]);
  const [showHistory, setShowHistory] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!stock) return;
    setContent('');
    setStreaming(false);
    loadHistory();
  }, [stock?.code]);

  const loadHistory = async () => {
    if (!stock) return;
    try {
      const h = await invoke<AIAnalysisResult[]>('get_analysis_history', {
        code: stock.code,
        limit: 10,
      });
      setHistory(h);
    } catch (e) {
      console.error('Failed to load history:', e);
    }
  };

  const startAnalysis = async () => {
    if (!stock || streaming) return;
    setContent('');
    setStreaming(true);

    const contextData = `当前价: ${stock.price}, 涨跌: ${stock.change_pct.toFixed(2)}%, 得分: ${stock.score}, PE(TTM): ${stock.pe_ttm.toFixed(1)}, PB: ${stock.pb.toFixed(2)}, ROE: ${stock.roe.toFixed(1)}%, 营收增速: ${stock.revenue_yoy.toFixed(1)}%, 市值: ${stock.total_market_cap.toFixed(0)}亿, 换手: ${stock.turnover_rate.toFixed(2)}%, 量比: ${stock.volume_ratio.toFixed(2)}, 主力净流入: ${stock.main_net_inflow.toFixed(0)}万, 5日涨幅: ${stock.pct_5d.toFixed(1)}%, 20日涨幅: ${stock.pct_20d.toFixed(1)}%, 标签: ${stock.labels.map(l => l.text).join(',')}`;

    let unlisten: (() => void) | undefined;
    try {
      unlisten = await safeListen<AIStreamEvent>(`ai-stream-${stock.code}`, (event) => {
        if (event.payload.content) {
          setContent(prev => prev + event.payload.content);
        }
        if (event.payload.done) {
          setStreaming(false);
        }
      });
    } catch {
      // Not in Tauri runtime
    }

    try {
      await invoke('analyze_stock', {
        code: stock.code,
        name: stock.name,
        contextData,
      });
    } catch (e) {
      console.error('Analysis failed:', e);
      setContent('分析失败: ' + String(e));
      setStreaming(false);
    }

    return () => unlisten?.();
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  useEffect(() => {
    if (contentRef.current) {
      contentRef.current.scrollTop = contentRef.current.scrollHeight;
    }
  }, [content]);

  if (!stock) return null;

  const changePct = stock.change_pct;
  const changeColor = changePct > 0 ? 'text-functional-up' : changePct < 0 ? 'text-functional-down' : 'text-txt-primary';

  return (
    <div className="fixed right-0 top-12 bottom-0 w-[420px] bg-bg-card border-l border-[#30363D] flex flex-col z-50 shadow-2xl shadow-black/50">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-[#30363D]">
        <div className="flex items-center gap-3">
          <div>
            <span className="font-bold text-txt-primary text-base">{stock.name}</span>
            <span className="text-txt-muted text-xs ml-2">{stock.code.replace(/^(sh|sz|bj)/, '')}</span>
          </div>
          <div className="flex items-center gap-1">
            <span className={`font-mono font-bold ${changeColor}`}>{stock.price.toFixed(2)}</span>
            <span className={`font-mono text-xs ${changeColor}`}>
              {changePct > 0 ? '+' : ''}{changePct.toFixed(2)}%
            </span>
          </div>
        </div>
        <button onClick={onClose} className="p-1 rounded hover:bg-bg-elevated transition-colors cursor-pointer">
          <X size={18} className="text-txt-secondary" />
        </button>
      </div>

      {/* Content */}
      <div ref={contentRef} className="flex-1 overflow-auto px-4 py-3">
        {content ? (
          <div className="prose prose-invert prose-sm max-w-none">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
            {streaming && <span className="inline-block w-2 h-4 bg-primary-gold animate-pulse ml-0.5" />}
          </div>
        ) : showHistory && history.length > 0 ? (
          <div className="space-y-3">
            {history.map(h => (
              <div key={h.id} className="p-3 rounded-lg bg-bg-elevated border border-[#30363D]">
                <div className="flex justify-between text-xs text-txt-muted mb-2">
                  <span>{h.model_name}</span>
                  <span>{h.created_at}</span>
                </div>
                <div className="prose prose-invert prose-xs max-w-none text-xs">
                  <ReactMarkdown remarkPlugins={[remarkGfm]}>
                    {h.content.length > 200 ? h.content.slice(0, 200) + '...' : h.content}
                  </ReactMarkdown>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-txt-muted">
            <p className="text-sm">点击下方按钮开始 AI 分析</p>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2 px-4 py-3 border-t border-[#30363D]">
        <button
          onClick={startAnalysis}
          disabled={streaming}
          className={`flex items-center gap-1.5 px-4 py-1.5 rounded-lg text-sm font-medium transition-all cursor-pointer ${
            streaming
              ? 'bg-bg-elevated text-txt-muted cursor-not-allowed'
              : 'bg-gradient-to-r from-red-600 to-red-500 text-white hover:from-red-500 hover:to-red-400 shadow-lg shadow-red-500/20'
          }`}
        >
          <RefreshCw size={14} className={streaming ? 'animate-spin' : ''} />
          {streaming ? '分析中...' : '开始分析'}
        </button>

        <button
          onClick={() => setShowHistory(!showHistory)}
          className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm bg-bg-elevated text-txt-secondary hover:text-txt-primary transition-colors cursor-pointer"
        >
          <History size={14} />
          历史
        </button>

        {content && (
          <button
            onClick={handleCopy}
            className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm bg-bg-elevated text-txt-secondary hover:text-txt-primary transition-colors cursor-pointer"
          >
            {copied ? <Check size={14} className="text-functional-down" /> : <Copy size={14} />}
            {copied ? '已复制' : '复制'}
          </button>
        )}
      </div>
    </div>
  );
}
