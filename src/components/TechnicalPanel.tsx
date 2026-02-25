import { TechnicalSignal, StockTechnicalAnalysis } from '../types';
import { TrendingUp, TrendingDown, Minus, Activity, BarChart2, Layers } from 'lucide-react';

interface TechnicalPanelProps {
  analysis: StockTechnicalAnalysis;
  onDiagnose: () => void;
}

function SignalDot({ direction }: { direction: string }) {
  const color =
    direction === 'bullish'
      ? 'bg-functional-down'
      : direction === 'bearish'
      ? 'bg-functional-up'
      : 'bg-txt-muted';
  return <span className={`inline-block w-2 h-2 rounded-full ${color}`} />;
}

function getVolumePriceLabel(relation: string): { label: string; color: string } {
  switch (relation) {
    case 'volume_up_price_up':
      return { label: '放量上涨', color: 'text-functional-down' };
    case 'volume_down_price_up':
      return { label: '缩量上涨', color: 'text-primary-gold' };
    case 'volume_up_price_down':
      return { label: '放量下跌', color: 'text-functional-up' };
    case 'volume_down_price_down':
      return { label: '缩量下跌', color: 'text-txt-muted' };
    case 'normal':
      return { label: '量价正常', color: 'text-primary-gold' };
    default:
      return { label: relation, color: 'text-primary-gold' };
  }
}

function getAlignmentInfo(alignment: string) {
  switch (alignment) {
    case 'bullish':
      return { label: '多头排列', color: 'text-functional-down', icon: TrendingUp, bg: 'bg-green-500/10 border-green-500/20' };
    case 'bearish':
      return { label: '空头排列', color: 'text-functional-up', icon: TrendingDown, bg: 'bg-red-500/10 border-red-500/20' };
    default:
      return { label: '均线纠缠', color: 'text-primary-gold', icon: Minus, bg: 'bg-yellow-500/10 border-yellow-500/20' };
  }
}

function SignalCard({ signal }: { signal: TechnicalSignal }) {
  const dirColor =
    signal.direction === 'bullish'
      ? 'text-functional-down'
      : signal.direction === 'bearish'
      ? 'text-functional-up'
      : 'text-txt-muted';

  return (
    <div className="flex items-start gap-2 px-2.5 py-2 rounded-lg bg-bg-base/50 border border-[#30363D]/50">
      <SignalDot direction={signal.direction} />
      <div className="flex-1 min-w-0">
        <div className={`text-xs font-medium ${dirColor} truncate`}>{signal.description}</div>
        <div className="flex items-center gap-2 mt-0.5">
          <span className="text-[10px] text-txt-muted">{signal.date}</span>
          <div className="flex gap-0.5">
            {Array.from({ length: 5 }).map((_, i) => (
              <span
                key={i}
                className={`w-1 h-2.5 rounded-sm ${
                  i < signal.strength ? (signal.direction === 'bullish' ? 'bg-functional-down' : 'bg-functional-up') : 'bg-[#30363D]'
                }`}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default function TechnicalPanel({ analysis, onDiagnose }: TechnicalPanelProps) {
  const maInfo = getAlignmentInfo(analysis.ma_alignment);
  const MaIcon = maInfo.icon;
  const vpInfo = getVolumePriceLabel(analysis.volume_price_relation);

  const bullishSignals = analysis.signals.filter((s) => s.direction === 'bullish');
  const bearishSignals = analysis.signals.filter((s) => s.direction === 'bearish');

  return (
    <div className="flex flex-col h-full overflow-auto">
      {/* Three-column grid */}
      <div className="grid grid-cols-3 gap-3 p-3">
        {/* Card 1: MA alignment */}
        <div className={`rounded-lg border p-3 ${maInfo.bg}`}>
          <div className="flex items-center gap-1.5 mb-2">
            <Layers size={13} className="text-txt-muted" />
            <span className="text-xs text-txt-muted font-medium">均线系统</span>
          </div>
          <div className="flex items-center gap-2">
            <MaIcon size={18} className={maInfo.color} />
            <span className={`text-sm font-bold ${maInfo.color}`}>{maInfo.label}</span>
          </div>
          <p className="text-[10px] text-txt-muted mt-1.5 leading-relaxed">
            MA5/10/20/60 {analysis.ma_alignment === 'bullish' ? '依次递增，趋势向上' : analysis.ma_alignment === 'bearish' ? '依次递减，趋势向下' : '交织缠绕，方向不明'}
          </p>
        </div>

        {/* Card 2: Key signals */}
        <div className="rounded-lg border border-[#30363D]/50 bg-bg-elevated/30 p-3">
          <div className="flex items-center gap-1.5 mb-2">
            <Activity size={13} className="text-txt-muted" />
            <span className="text-xs text-txt-muted font-medium">关键信号</span>
          </div>
          {analysis.signals.length > 0 ? (
            <div className="space-y-1.5 max-h-[120px] overflow-auto">
              {analysis.signals.slice(0, 4).map((s, i) => (
                <div key={i} className="flex items-center gap-1.5">
                  <SignalDot direction={s.direction} />
                  <span className={`text-xs truncate ${
                    s.direction === 'bullish' ? 'text-functional-down' : s.direction === 'bearish' ? 'text-functional-up' : 'text-txt-secondary'
                  }`}>
                    {s.description}
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-xs text-txt-muted">暂无明显信号</p>
          )}
        </div>

        {/* Card 3: Volume-price relation */}
        <div className="rounded-lg border border-[#30363D]/50 bg-bg-elevated/30 p-3">
          <div className="flex items-center gap-1.5 mb-2">
            <BarChart2 size={13} className="text-txt-muted" />
            <span className="text-xs text-txt-muted font-medium">量价关系</span>
          </div>
          <span className={`text-sm font-bold ${vpInfo.color}`}>{vpInfo.label}</span>
          <div className="flex items-center gap-3 mt-2">
            <div className="text-center">
              <span className="text-functional-down text-xs font-bold">{bullishSignals.length}</span>
              <p className="text-[10px] text-txt-muted">看多</p>
            </div>
            <div className="text-center">
              <span className="text-functional-up text-xs font-bold">{bearishSignals.length}</span>
              <p className="text-[10px] text-txt-muted">看空</p>
            </div>
          </div>
        </div>
      </div>

      {/* Signal details */}
      {analysis.signals.length > 0 && (
        <div className="px-3 pb-2">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs text-txt-muted font-medium">信号明细</span>
            <button
              onClick={onDiagnose}
              className="flex items-center gap-1 px-3 py-1 rounded-lg text-xs font-medium bg-gradient-to-r from-primary-gold/20 to-primary-gold/10 text-primary-gold border border-primary-gold/30 hover:from-primary-gold/30 hover:to-primary-gold/20 transition-all cursor-pointer"
            >
              <Activity size={12} />
              AI 诊断
            </button>
          </div>
          <div className="grid grid-cols-2 gap-2">
            {analysis.signals.map((signal, i) => (
              <SignalCard key={i} signal={signal} />
            ))}
          </div>
        </div>
      )}

      {/* Summary */}
      {analysis.summary && (
        <div className="mx-3 mb-3 p-3 rounded-lg bg-bg-base/50 border border-[#30363D]/50">
          <span className="text-xs text-txt-muted font-medium block mb-1">技术摘要</span>
          <p className="text-xs text-txt-secondary leading-relaxed">{analysis.summary}</p>
        </div>
      )}
    </div>
  );
}
