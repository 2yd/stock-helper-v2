import { useStockStore } from '../stores/stockStore';
import { Activity, Clock, Zap, BarChart3 } from 'lucide-react';

export default function StatusBar() {
  const { lastRefreshTime, marketStatus, tokenUsageToday, results } = useStockStore();

  const statusColor = marketStatus.includes('交易中') || marketStatus.includes('竞价中')
    ? 'text-functional-down'
    : 'text-txt-muted';

  const statusDot = marketStatus.includes('交易中') || marketStatus.includes('竞价中')
    ? 'bg-green-400 animate-pulse'
    : 'bg-gray-500';

  const avgScore = results.length > 0
    ? Math.round(results.reduce((sum, r) => sum + r.score, 0) / results.length)
    : 0;

  const highScoreCount = results.filter(r => r.score >= 80).length;

  return (
    <div className="h-8 flex items-center px-4 bg-bg-card/70 border-t border-[#30363D] text-xs flex-shrink-0">
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-1.5">
          <div className={`w-2 h-2 rounded-full ${statusDot}`} />
          <span className={statusColor}>{marketStatus}</span>
        </div>
        <div className="flex items-center gap-1 text-txt-secondary">
          <Clock size={12} />
          <span>刷新: {lastRefreshTime || '--:--:--'}</span>
        </div>
      </div>

      <div className="flex-1 flex items-center justify-center gap-4 text-txt-secondary">
        <span>
          筛出 <span className="text-txt-primary font-semibold">{results.length}</span> 只
        </span>
        <span>
          ≥80分: <span className="text-primary-red font-semibold">{highScoreCount}</span> 只
        </span>
        <span>
          均分: <span className="text-primary-orange font-semibold">{avgScore}</span>
        </span>
      </div>

      <div className="flex items-center gap-1 text-txt-muted">
        <Zap size={12} />
        <span>Token: {tokenUsageToday.toLocaleString()}</span>
      </div>
    </div>
  );
}
