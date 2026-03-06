import { AreaChart, Area, ResponsiveContainer } from 'recharts';
import { IndexQuote, KlineItem } from '../../types';

interface Props {
  quote: IndexQuote;
  klineData: KlineItem[];
}

function formatAmount(val: number): string {
  if (val >= 1e12) return `${(val / 1e12).toFixed(2)}万亿`;
  if (val >= 1e8) return `${(val / 1e8).toFixed(0)}亿`;
  return `${val.toFixed(0)}`;
}

export default function IndexCard({ quote, klineData }: Props) {
  const isUp = quote.change_pct >= 0;
  const color = isUp ? '#E74C3C' : '#2ECC71';
  const bgGradient = isUp ? 'from-red-950/20 to-transparent' : 'from-green-950/20 to-transparent';

  const chartData = klineData.map(k => ({ close: k.close }));

  return (
    <div className={`rounded-xl bg-bg-card border border-[#30363D] p-3.5 hover:brightness-110 transition-all duration-200 bg-gradient-to-b ${bgGradient}`}>
      {/* 顶部：名称 + 涨跌幅标签 */}
      <div className="flex items-center justify-between mb-1">
        <span className="text-xs text-txt-secondary font-medium">{quote.name}</span>
        <span
          className={`text-xs font-din font-semibold px-2 py-0.5 rounded-full ${
            isUp ? 'bg-functional-up/15 text-functional-up' : 'bg-functional-down/15 text-functional-down'
          }`}
        >
          {isUp ? '+' : ''}{quote.change_pct.toFixed(2)}%
        </span>
      </div>

      {/* 最新价 */}
      <div className="flex items-baseline gap-2 mb-0.5">
        <span className={`text-2xl font-din font-bold ${isUp ? 'text-functional-up' : 'text-functional-down'}`}>
          {quote.price.toFixed(2)}
        </span>
        <span className={`text-xs font-din ${isUp ? 'text-functional-up/70' : 'text-functional-down/70'}`}>
          {isUp ? '+' : ''}{quote.change_amount.toFixed(2)}
        </span>
      </div>

      {/* 成交额 */}
      <div className="text-xs text-txt-muted mb-2">
        成交 {formatAmount(quote.amount)}
      </div>

      {/* 迷你走势图 */}
      {chartData.length > 0 && (
        <div className="h-[55px] -mx-1">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={chartData}>
              <defs>
                <linearGradient id={`grad-${quote.code}`} x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor={color} stopOpacity={0.3} />
                  <stop offset="100%" stopColor={color} stopOpacity={0.02} />
                </linearGradient>
              </defs>
              <Area
                type="monotone"
                dataKey="close"
                stroke={color}
                strokeWidth={1.5}
                fill={`url(#grad-${quote.code})`}
                isAnimationActive={false}
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      )}
    </div>
  );
}
