import { ArrowUp, ArrowDown, Sparkles, RefreshCw } from 'lucide-react';
import { MarketOverview } from '../../types';

interface Props {
  overview: MarketOverview | null;
  aiComment: string | null;
  aiCommentLoading: boolean;
  onRefreshAi: () => void;
}

function formatAmount(val: number): string {
  if (val >= 1e12) return `${(val / 1e12).toFixed(2)}万亿`;
  if (val >= 1e8) return `${(val / 1e8).toFixed(0)}亿`;
  return `${val.toFixed(0)}`;
}

const statusColor: Record<string, string> = {
  '交易中(上午)': 'bg-functional-down',
  '交易中(下午)': 'bg-functional-down',
  '竞价中': 'bg-functional-warn',
  '午间休市': 'bg-txt-muted',
  '已收盘': 'bg-txt-muted',
  '盘前': 'bg-functional-info',
  '休市(周末)': 'bg-txt-muted',
  '集合竞价结束': 'bg-functional-warn',
};

export default function MarketHeader({ overview, aiComment, aiCommentLoading, onRefreshAi }: Props) {
  if (!overview) return null;

  const isTrading = overview.market_status.includes('交易中');
  const dotColor = statusColor[overview.market_status] || 'bg-txt-muted';
  const vc = overview.volume_compare;
  const isUp = vc.diff > 0;

  return (
    <div className="space-y-2.5">
      {/* 状态条 */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          {/* 市场状态胶囊 */}
          <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-bg-elevated border border-[#30363D]">
            <span className={`w-2 h-2 rounded-full ${dotColor} ${isTrading ? 'animate-pulse' : ''}`} />
            <span className="text-xs font-medium text-txt-secondary">{overview.market_status}</span>
          </div>
          <div className="relative group">
            <span className="px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wider bg-amber-500/15 text-amber-400 rounded cursor-default">Beta</span>
            <div className="absolute left-1/2 -translate-x-1/2 top-full mt-2 px-3 py-2 rounded-lg bg-[#1C2128] border border-[#30363D] shadow-lg text-xs text-txt-secondary whitespace-nowrap opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-150 z-50">
              <div className="absolute left-1/2 -translate-x-1/2 -top-1 w-2 h-2 rotate-45 bg-[#1C2128] border-l border-t border-[#30363D]" />
              此功能仍在开发中，可能随时有较大调整
            </div>
          </div>

          {/* 两市总成交额 */}
          <div className="flex items-center gap-2">
            <span className="text-xs text-txt-muted">两市成交</span>
            <span className="text-lg font-din font-bold text-txt-primary">{formatAmount(overview.total_amount)}</span>
          </div>

          {/* 量能对比 */}
          {vc.yesterday_amount > 0 && (
            <div className={`flex items-center gap-1 text-xs ${isUp ? 'text-functional-up' : 'text-functional-down'}`}>
              {isUp ? <ArrowUp size={12} /> : <ArrowDown size={12} />}
              <span>VS昨日 {isUp ? '+' : ''}{((vc.ratio - 1) * 100).toFixed(1)}%</span>
              <span className="text-txt-muted">({isUp ? '放量' : '缩量'})</span>
            </div>
          )}

        </div>

        <span className="text-xs text-txt-muted font-din">{overview.update_time}</span>
      </div>

      {/* AI 解说卡片 */}
      {(aiComment || aiCommentLoading) && (
        <div className="flex items-start gap-2.5 px-3 py-2.5 rounded-lg bg-bg-elevated border-l-2 border-functional-info">
          <Sparkles size={14} className="text-functional-info mt-0.5 flex-shrink-0" />
          {aiCommentLoading ? (
            <div className="flex-1 space-y-1.5">
              <div className="h-3 bg-bg-card rounded animate-pulse w-3/4" />
              <div className="h-3 bg-bg-card rounded animate-pulse w-1/2" />
            </div>
          ) : (
            <p className="text-xs text-txt-secondary leading-relaxed flex-1">{aiComment}</p>
          )}
          <button
            onClick={onRefreshAi}
            disabled={aiCommentLoading}
            className="p-1 rounded hover:bg-bg-card transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed flex-shrink-0"
          >
            <RefreshCw size={12} className={`text-txt-muted ${aiCommentLoading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      )}
    </div>
  );
}
