import { useEffect, useRef } from 'react';
import { createChart, LineStyle } from 'lightweight-charts';
import type { IChartApi } from 'lightweight-charts';
import { EquityPoint } from '../types';

interface BacktestChartProps {
  equityCurve: EquityPoint[];
}

export default function BacktestChart({ equityCurve }: BacktestChartProps) {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const drawdownContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const drawdownChartRef = useRef<IChartApi | null>(null);

  useEffect(() => {
    if (!chartContainerRef.current || !drawdownContainerRef.current || equityCurve.length === 0) return;

    // Main chart - Equity curve
    const mainChart = createChart(chartContainerRef.current, {
      layout: {
        background: { color: 'transparent' },
        textColor: '#8B949E',
        fontSize: 11,
      },
      grid: {
        vertLines: { color: 'rgba(28,35,51,0.6)' },
        horzLines: { color: 'rgba(28,35,51,0.6)' },
      },
      crosshair: {
        vertLine: { color: '#484F58', style: 2 },
        horzLine: { color: '#484F58', style: 2 },
      },
      rightPriceScale: {
        borderColor: '#30363D',
      },
      timeScale: {
        borderColor: '#30363D',
        timeVisible: false,
      },
      autoSize: true,
    });

    const strategySeries = mainChart.addLineSeries({
      color: '#FFD700',
      lineWidth: 2,
      title: '策略净值',
    });

    const benchmarkSeries = mainChart.addLineSeries({
      color: '#8B949E',
      lineWidth: 1,
      lineStyle: LineStyle.Dashed,
      title: '沪深300',
    });

    const strategyData = equityCurve.map((p) => ({
      time: p.date as string,
      value: p.equity,
    }));

    const benchmarkData = equityCurve.map((p) => ({
      time: p.date as string,
      value: p.benchmark,
    }));

    strategySeries.setData(strategyData);
    benchmarkSeries.setData(benchmarkData);
    mainChart.timeScale().fitContent();
    chartRef.current = mainChart;

    // Drawdown chart
    const ddChart = createChart(drawdownContainerRef.current, {
      layout: {
        background: { color: 'transparent' },
        textColor: '#8B949E',
        fontSize: 11,
      },
      grid: {
        vertLines: { color: 'rgba(28,35,51,0.6)' },
        horzLines: { color: 'rgba(28,35,51,0.6)' },
      },
      crosshair: {
        vertLine: { color: '#484F58', style: 2 },
        horzLine: { color: '#484F58', style: 2 },
      },
      rightPriceScale: {
        borderColor: '#30363D',
      },
      timeScale: {
        borderColor: '#30363D',
        timeVisible: false,
      },
      autoSize: true,
    });

    const drawdownSeries = ddChart.addAreaSeries({
      topColor: 'rgba(231, 76, 60, 0.3)',
      bottomColor: 'rgba(231, 76, 60, 0.05)',
      lineColor: '#E74C3C',
      lineWidth: 1,
      title: '回撤',
    });

    const drawdownData = equityCurve.map((p) => ({
      time: p.date as string,
      value: -Math.abs(p.drawdown) * 100,
    }));

    drawdownSeries.setData(drawdownData);
    ddChart.timeScale().fitContent();
    drawdownChartRef.current = ddChart;

    // Sync time scales
    mainChart.timeScale().subscribeVisibleLogicalRangeChange((range) => {
      if (range) ddChart.timeScale().setVisibleLogicalRange(range);
    });
    ddChart.timeScale().subscribeVisibleLogicalRangeChange((range) => {
      if (range) mainChart.timeScale().setVisibleLogicalRange(range);
    });

    return () => {
      mainChart.remove();
      ddChart.remove();
      chartRef.current = null;
      drawdownChartRef.current = null;
    };
  }, [equityCurve]);

  if (equityCurve.length === 0) return null;

  return (
    <div className="flex flex-col h-full">
      <div ref={chartContainerRef} className="flex-[7] min-h-0" />
      <div className="border-t border-[#30363D]">
        <span className="text-[10px] text-txt-muted px-3 py-1 inline-block">回撤曲线</span>
      </div>
      <div ref={drawdownContainerRef} className="flex-[3] min-h-0" />
    </div>
  );
}
