import { useEffect, useRef, useCallback } from 'react';
import { init, dispose } from 'klinecharts';
import type { Chart } from 'klinecharts';
import { KlineItem, TechnicalIndicators } from '../types';

interface KlineChartProps {
  klineData: KlineItem[];
  indicators: TechnicalIndicators;
  period: 'day' | 'week';
  onPeriodChange: (period: 'day' | 'week') => void;
  activeIndicators: string[];
  onIndicatorsChange: (indicators: string[]) => void;
}

const INDICATOR_OPTIONS = [
  { key: 'MA', label: 'MA' },
  { key: 'MACD', label: 'MACD' },
  { key: 'KDJ', label: 'KDJ' },
  { key: 'RSI', label: 'RSI' },
  { key: 'BOLL', label: 'BOLL' },
];

export default function KlineChart({
  klineData,
  period,
  onPeriodChange,
  activeIndicators,
  onIndicatorsChange,
}: KlineChartProps) {
  const chartRef = useRef<Chart | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const subPaneIds = useRef<Record<string, string>>({});

  const initChart = useCallback(() => {
    if (!containerRef.current) return;
    if (chartRef.current) {
      dispose(containerRef.current);
    }
    const chart = init(containerRef.current, {
      styles: {
        grid: {
          show: true,
          horizontal: { color: 'rgba(28,35,51,0.6)', style: 'dashed' as const },
          vertical: { color: 'rgba(28,35,51,0.6)', style: 'dashed' as const },
        },
        candle: {
          type: 'candle_solid' as const,
          bar: {
            upColor: '#E74C3C',
            downColor: '#2ECC71',
            upBorderColor: '#E74C3C',
            downBorderColor: '#2ECC71',
            upWickColor: '#E74C3C',
            downWickColor: '#2ECC71',
          },
          priceMark: {
            show: true,
            high: { show: true, color: '#E74C3C' },
            low: { show: true, color: '#2ECC71' },
            last: {
              show: true,
              upColor: '#E74C3C',
              downColor: '#2ECC71',
              noChangeColor: '#8B949E',
            },
          },
          tooltip: {
            text: { color: '#E6EDF3', marginLeft: 8, marginTop: 6, marginRight: 8, marginBottom: 0, size: 12 },
          },
        },
        indicator: {
          tooltip: {
            text: { color: '#E6EDF3', marginLeft: 8, marginTop: 6, marginRight: 8, marginBottom: 0, size: 12 },
          },
        },
        xAxis: {
          axisLine: { color: '#30363D' },
          tickLine: { color: '#30363D' },
          tickText: { color: '#8B949E', size: 11 },
        },
        yAxis: {
          axisLine: { color: '#30363D' },
          tickLine: { color: '#30363D' },
          tickText: { color: '#8B949E', size: 11 },
        },
        crosshair: {
          show: true,
          horizontal: { line: { color: '#484F58', style: 'dashed' as const } },
          vertical: { line: { color: '#484F58', style: 'dashed' as const } },
        },
        separator: { color: '#30363D' },
      },
    });
    chartRef.current = chart;
    return chart;
  }, []);

  // Initialize chart and apply data
  useEffect(() => {
    const chart = initChart();
    if (!chart || klineData.length === 0) return;

    const data = klineData.map((item) => ({
      timestamp: new Date(item.date).getTime(),
      open: item.open,
      high: item.high,
      low: item.low,
      close: item.close,
      volume: item.volume,
      turnover: item.amount,
    }));

    chart.applyNewData(data);

    // Always show volume
    chart.createIndicator('VOL', false, { id: 'candle_pane' });

    return () => {
      if (containerRef.current) {
        dispose(containerRef.current);
        chartRef.current = null;
      }
    };
  }, [klineData, initChart]);

  // Manage sub-pane indicators
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart) return;

    const subIndicators = ['MACD', 'KDJ', 'RSI'];
    const mainIndicators = ['MA', 'BOLL'];

    // Remove old sub-panes
    for (const key of Object.keys(subPaneIds.current)) {
      if (!activeIndicators.includes(key)) {
        chart.removeIndicator('candle_pane', key);
        const paneId = subPaneIds.current[key];
        if (paneId) {
          chart.removeIndicator(paneId, key);
        }
        delete subPaneIds.current[key];
      }
    }

    // Add main-pane indicators (MA, BOLL)
    for (const ind of mainIndicators) {
      if (activeIndicators.includes(ind) && !subPaneIds.current[ind]) {
        chart.createIndicator(ind, false, { id: 'candle_pane' });
        subPaneIds.current[ind] = 'candle_pane';
      }
    }

    // Add sub-pane indicators (MACD, KDJ, RSI)
    for (const ind of subIndicators) {
      if (activeIndicators.includes(ind) && !subPaneIds.current[ind]) {
        const paneId = chart.createIndicator(ind, true);
        if (paneId) subPaneIds.current[ind] = paneId as string;
      }
    }
  }, [activeIndicators]);

  const toggleIndicator = (key: string) => {
    const newIndicators = activeIndicators.includes(key)
      ? activeIndicators.filter((i) => i !== key)
      : [...activeIndicators, key];
    onIndicatorsChange(newIndicators);
  };

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center gap-3 px-3 py-2 border-b border-[#30363D]">
        {/* Period selector */}
        <div className="flex items-center gap-1">
          {(['day', 'week'] as const).map((p) => (
            <button
              key={p}
              onClick={() => onPeriodChange(p)}
              className={`px-2.5 py-1 text-xs font-medium rounded transition-all cursor-pointer ${
                period === p
                  ? 'bg-primary-gold/20 text-primary-gold border border-primary-gold/30'
                  : 'text-txt-secondary hover:text-txt-primary hover:bg-bg-elevated'
              }`}
            >
              {p === 'day' ? '日K' : '周K'}
            </button>
          ))}
        </div>

        <div className="w-px h-4 bg-[#30363D]" />

        {/* Indicator toggles */}
        <div className="flex items-center gap-1">
          {INDICATOR_OPTIONS.map((opt) => (
            <button
              key={opt.key}
              onClick={() => toggleIndicator(opt.key)}
              className={`px-2 py-1 text-xs font-medium rounded transition-all cursor-pointer ${
                activeIndicators.includes(opt.key)
                  ? 'bg-blue-600/20 text-blue-300 border border-blue-500/30'
                  : 'text-txt-secondary hover:text-txt-primary hover:bg-bg-elevated'
              }`}
            >
              {opt.label}
            </button>
          ))}
        </div>
      </div>

      {/* Chart container */}
      <div ref={containerRef} className="flex-1 min-h-0" />
    </div>
  );
}
