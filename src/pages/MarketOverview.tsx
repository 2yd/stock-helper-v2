import { useEffect, useRef } from 'react';
import { useMarketStore } from '../stores/marketStore';
import { useSettingsStore } from '../stores/settingsStore';
import MarketHeader from '../components/dashboard/MarketHeader';
import IndexCard from '../components/dashboard/IndexCard';
import SentimentMeter from '../components/dashboard/SentimentMeter';
import SectorRank from '../components/dashboard/SectorRank';
import { Loader2 } from 'lucide-react';

export default function MarketOverview() {
  const {
    overview, aiComment, aiCommentLoading, loading, error,
    indexKlines,
    fetchOverview, generateAiComment, fetchIndexKlines,
    startAutoRefresh, stopAutoRefresh,
  } = useMarketStore();

  const { settings, loadSettings } = useSettingsStore();
  const initialLoadDone = useRef(false);

  // 初始化：加载设置 + 数据
  useEffect(() => {
    if (!initialLoadDone.current) {
      initialLoadDone.current = true;
      loadSettings();
      fetchOverview();
      fetchIndexKlines();
    }
  }, []);

  // 数据加载完成后尝试生成 AI 解说
  useEffect(() => {
    if (overview && settings) {
      const hasAi = settings.ai_configs.some(c => c.enabled);
      if (hasAi && !aiComment && !aiCommentLoading) {
        generateAiComment();
      }
    }
  }, [overview, settings]);

  // 自动刷新
  useEffect(() => {
    if (overview?.market_status.includes('交易中') || overview?.market_status === '竞价中') {
      startAutoRefresh();
    } else {
      stopAutoRefresh();
    }
    return () => stopAutoRefresh();
  }, [overview?.market_status]);

  const hasAiConfig = settings?.ai_configs?.some(c => c.enabled) ?? false;

  if (loading && !overview) {
    return (
      <div className="h-full flex items-center justify-center">
        <Loader2 size={24} className="animate-spin text-functional-info" />
        <span className="ml-2 text-sm text-txt-secondary">加载大盘数据...</span>
      </div>
    );
  }

  if (error && !overview) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center">
          <p className="text-sm text-functional-up mb-2">数据加载失败</p>
          <p className="text-xs text-txt-muted mb-3">{error}</p>
          <button
            onClick={fetchOverview}
            className="px-3 py-1.5 text-xs bg-functional-info/20 text-functional-info rounded-md hover:bg-functional-info/30 transition-colors cursor-pointer"
          >
            重试
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto p-4 space-y-3">
      {/* 顶部信息条 + AI解说 */}
      <MarketHeader
        overview={overview}
        aiComment={hasAiConfig ? aiComment : null}
        aiCommentLoading={hasAiConfig && aiCommentLoading}
        onRefreshAi={generateAiComment}
      />

      {/* 第一行：三大指数 + 情绪仪表盘 */}
      {overview && (
        <div className="grid grid-cols-4 gap-3">
          {overview.indexes.map((idx) => (
            <IndexCard
              key={idx.code}
              quote={idx}
              klineData={indexKlines[idx.code] || []}
            />
          ))}
          <SentimentMeter
            sentiment={overview.sentiment}
            stats={overview.market_stats}
            volumeCompare={overview.volume_compare}
          />
        </div>
      )}

      {/* 第二行：板块排行 + 全球指数 */}
      {overview && (
        <SectorRank
          sectorTop={overview.sector_top}
          sectorBottom={overview.sector_bottom}
          globalIndexes={overview.global_indexes}
        />
      )}
    </div>
  );
}
