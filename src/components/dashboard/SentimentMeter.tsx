import { PieChart, Pie, Cell } from 'recharts';
import { SentimentInfo, MarketStats, VolumeCompare } from '../../types';

interface Props {
  sentiment: SentimentInfo;
  stats: MarketStats;
  volumeCompare: VolumeCompare;
}

const levelColors: Record<string, string> = {
  '极强': '#E74C3C',
  '偏强': '#F39C12',
  '中性': '#3498DB',
  '偏弱': '#8B949E',
  '极弱': '#2ECC71',
};

export default function SentimentMeter({ sentiment, stats, volumeCompare }: Props) {
  const total = stats.rise_count + stats.fall_count + stats.flat_count;
  const risePct = total > 0 ? (stats.rise_count / total) * 100 : 0;
  const flatPct = total > 0 ? (stats.flat_count / total) * 100 : 0;
  const fallPct = total > 0 ? (stats.fall_count / total) * 100 : 0;

  // 弧形仪表盘数据（180度半圆）
  const gaugeData = [
    { value: sentiment.score },
    { value: 100 - sentiment.score },
  ];
  const gaugeColor = levelColors[sentiment.level] || '#3498DB';

  return (
    <div className="rounded-xl bg-bg-card border border-[#30363D] p-3.5 hover:brightness-110 transition-all duration-200 flex flex-col">
      {/* 弧形仪表盘 */}
      <div className="flex flex-col items-center -mt-1">
        <div className="relative w-[140px] h-[75px] overflow-hidden">
          <PieChart width={140} height={140}>
            <Pie
              data={gaugeData}
              cx={70}
              cy={70}
              startAngle={180}
              endAngle={0}
              innerRadius={48}
              outerRadius={62}
              paddingAngle={0}
              dataKey="value"
              isAnimationActive={false}
            >
              <Cell fill={gaugeColor} />
              <Cell fill="#1C2333" />
            </Pie>
          </PieChart>
          {/* 中央文字 */}
          <div className="absolute inset-x-0 bottom-0 flex flex-col items-center">
            <span className="text-2xl font-din font-bold text-txt-primary leading-none">{sentiment.score.toFixed(0)}</span>
            <span className="text-xs font-medium mt-0.5" style={{ color: gaugeColor }}>{sentiment.level}</span>
          </div>
        </div>
        <div className="text-xs text-txt-muted mt-0.5">
          赚钱效应 <span className="font-din text-txt-secondary">{sentiment.money_effect.toFixed(1)}%</span>
        </div>
      </div>

      {/* 涨跌分布条 */}
      <div className="mt-3">
        <div className="flex items-center justify-between text-xs mb-1">
          <span className="text-functional-up font-din">{stats.rise_count}</span>
          <span className="text-txt-muted font-din">{stats.flat_count}</span>
          <span className="text-functional-down font-din">{stats.fall_count}</span>
        </div>
        <div className="flex h-2 rounded-full overflow-hidden bg-bg-elevated">
          <div className="bg-functional-up transition-all duration-500" style={{ width: `${risePct}%` }} />
          <div className="bg-txt-muted transition-all duration-500" style={{ width: `${flatPct}%` }} />
          <div className="bg-functional-down transition-all duration-500" style={{ width: `${fallPct}%` }} />
        </div>
        <div className="flex items-center justify-between text-[10px] text-txt-muted mt-0.5">
          <span>涨</span>
          <span>平</span>
          <span>跌</span>
        </div>
      </div>

      {/* 关键统计 */}
      <div className="grid grid-cols-2 gap-1.5 mt-3">
        <StatCard label="涨跌比" value={`${risePct.toFixed(1)}%`} color="#E74C3C" />
        <StatCard label="量比" value={`${volumeCompare.ratio.toFixed(2)}`} color="#3498DB" />
      </div>
    </div>
  );
}

function StatCard({ label, value, color }: { label: string; value: string; color: string }) {
  return (
    <div className="rounded-md bg-bg-elevated p-1.5 text-center">
      <div className="h-0.5 rounded-full mb-1" style={{ backgroundColor: color }} />
      <div className="text-sm font-din font-bold text-txt-primary">{value}</div>
      <div className="text-[10px] text-txt-muted">{label}</div>
    </div>
  );
}
