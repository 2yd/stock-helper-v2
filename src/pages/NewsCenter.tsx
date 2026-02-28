import { useState, useEffect, useCallback, useRef } from 'react';
import {
  Loader2, RefreshCw, Zap, Newspaper, FileText, BookOpen,
  ExternalLink, Search, ChevronDown, Clock, Building2, Star,
  Tag, Filter,
} from 'lucide-react';
import type { NewsCategory } from '../types';
import { Tooltip } from 'antd';
import { open } from '@tauri-apps/plugin-shell';
import { useNewsStore, type NewsTab } from '../stores/newsStore';

const TAB_CONFIG: { key: NewsTab; label: string; icon: React.ReactNode; color: string }[] = [
  { key: 'telegraph', label: '实时快讯', icon: <Zap size={13} />, color: 'text-yellow-400' },
  { key: 'news', label: '财经要闻', icon: <Newspaper size={13} />, color: 'text-blue-400' },
  { key: 'announcement', label: '公司公告', icon: <FileText size={13} />, color: 'text-green-400' },
  { key: 'report', label: '研究报告', icon: <BookOpen size={13} />, color: 'text-purple-400' },
];

export default function NewsCenter() {
  const {
    activeTab, setActiveTab,
    telegraphs, telegraphLoading, fetchTelegraphs,
    newsList, newsLoading, fetchNews,
    announcements, announcementLoading, fetchAnnouncements,
    reports, reportLoading, fetchReports,
    stockKeyword, stockNews, stockNewsLoading, fetchStockNews, setStockKeyword,
    startAutoRefresh, stopAutoRefresh,
  } = useNewsStore();

  const [autoRefresh, setAutoRefresh] = useState(false);
  const [annStockFilter, setAnnStockFilter] = useState('');
  const [reportStockFilter, setReportStockFilter] = useState('');
  const searchTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // 初始加载
  useEffect(() => {
    fetchTelegraphs();
    return () => { stopAutoRefresh(); };
  }, []);

  // Tab 切换时加载对应数据
  useEffect(() => {
    switch (activeTab) {
      case 'telegraph':
        if (telegraphs.length === 0) fetchTelegraphs();
        break;
      case 'news':
        if (newsList.length === 0) fetchNews();
        break;
      case 'announcement':
        if (announcements.length === 0) fetchAnnouncements();
        break;
      case 'report':
        if (reports.length === 0) fetchReports();
        break;
    }
  }, [activeTab]);

  const handleAutoRefresh = useCallback((checked: boolean) => {
    setAutoRefresh(checked);
    if (checked) {
      startAutoRefresh(30);
    } else {
      stopAutoRefresh();
    }
  }, [startAutoRefresh, stopAutoRefresh]);

  const handleRefresh = useCallback(() => {
    switch (activeTab) {
      case 'telegraph': fetchTelegraphs(); break;
      case 'news': fetchNews(); break;
      case 'announcement': fetchAnnouncements(annStockFilter || undefined); break;
      case 'report': fetchReports(reportStockFilter || undefined); break;
    }
  }, [activeTab, annStockFilter, reportStockFilter]);

  const handleStockSearch = useCallback((value: string) => {
    setStockKeyword(value);
    if (searchTimerRef.current) clearTimeout(searchTimerRef.current);
    if (!value.trim()) return;
    searchTimerRef.current = setTimeout(() => fetchStockNews(value), 500);
  }, [fetchStockNews, setStockKeyword]);

  const isLoading = activeTab === 'telegraph' ? telegraphLoading
    : activeTab === 'news' ? newsLoading
    : activeTab === 'announcement' ? announcementLoading
    : reportLoading;

  const openUrl = (url: string) => {
    if (url) open(url).catch(console.error);
  };

  const formatTime = (t: string) => {
    if (!t) return '';
    // 如果是完整日期时间，只取时间部分
    if (t.includes(' ')) {
      const parts = t.split(' ');
      const today = new Date().toISOString().slice(0, 10);
      if (parts[0] === today) return parts[1]?.slice(0, 5) || t;
      return parts[0]?.slice(5) + ' ' + (parts[1]?.slice(0, 5) || '');
    }
    return t;
  };

  const importanceBadge = (level: number) => {
    if (level >= 2) return <span className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-bold bg-red-500/20 text-red-400 border border-red-500/30">重要</span>;
    if (level >= 1) return <span className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-bold bg-orange-500/20 text-orange-400 border border-orange-500/30">关注</span>;
    return null;
  };

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Top Toolbar */}
      <div className="flex items-center gap-3 px-4 py-2 bg-bg-elevated/50 border-b border-[#30363D] flex-shrink-0">
        <h2 className="text-sm font-bold text-primary-gold flex items-center gap-1.5">
          <Newspaper size={15} />
          资讯中心
        </h2>

        {/* Tab 切换 */}
        <div className="flex items-center gap-0.5 ml-3 bg-bg-base rounded-lg p-0.5 border border-[#30363D]">
          {TAB_CONFIG.map(({ key, label, icon, color }) => (
            <button
              key={key}
              onClick={() => setActiveTab(key)}
              className={`flex items-center gap-1 px-2.5 py-1 text-xs font-medium rounded-md transition-all cursor-pointer ${
                activeTab === key
                  ? `bg-bg-card ${color} border border-[#484F58]`
                  : 'text-txt-muted hover:text-txt-secondary'
              }`}
            >
              {icon}
              {label}
            </button>
          ))}
        </div>

        <div className="flex-1" />

        {/* 个股新闻搜索 */}
        <div className="relative w-48">
          <Search size={12} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-txt-muted" />
          <input
            type="text"
            placeholder="搜索个股新闻..."
            value={stockKeyword}
            onChange={(e) => handleStockSearch(e.target.value)}
            className="w-full pl-7 pr-3 py-1.5 rounded-lg bg-bg-base border border-[#30363D] text-xs text-txt-primary placeholder:text-txt-muted outline-none focus:border-primary-gold/50 transition-colors"
          />
        </div>

        {/* 自动刷新 */}
        {activeTab === 'telegraph' && (
          <label className="flex items-center gap-1.5 text-xs text-txt-muted cursor-pointer select-none">
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => handleAutoRefresh(e.target.checked)}
              className="accent-primary-gold w-3 h-3"
            />
            自动(30s)
          </label>
        )}

        <button
          onClick={handleRefresh}
          disabled={isLoading}
          className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-xs font-medium bg-bg-card text-txt-secondary border border-[#30363D] hover:border-[#484F58] hover:text-txt-primary transition-all cursor-pointer disabled:opacity-40"
        >
          <RefreshCw size={12} className={isLoading ? 'animate-spin' : ''} />
          刷新
        </button>
      </div>

      {/* 个股新闻搜索结果 (浮动) */}
      {stockKeyword.trim() && (
        <div className="border-b border-[#30363D] bg-bg-elevated/80 max-h-60 overflow-auto">
          <div className="px-4 py-2 flex items-center gap-2">
            <Search size={12} className="text-primary-gold" />
            <span className="text-xs text-txt-secondary">
              个股搜索: <span className="text-txt-primary font-bold">{stockKeyword}</span>
            </span>
            {stockNewsLoading && <Loader2 size={12} className="animate-spin text-txt-muted" />}
            <button
              onClick={() => { setStockKeyword(''); }}
              className="ml-auto text-xs text-txt-muted hover:text-txt-primary cursor-pointer"
            >
              关闭
            </button>
          </div>
          {stockNews.length > 0 ? (
            <div className="px-4 pb-2 space-y-1">
              {stockNews.map((item) => (
                <div
                  key={item.id}
                  className="flex items-start gap-2 py-1.5 px-2 rounded hover:bg-bg-card/50 cursor-pointer group"
                  onClick={() => openUrl(item.url)}
                >
                  <span className="text-[10px] text-txt-muted whitespace-nowrap mt-0.5">{formatTime(item.publish_time)}</span>
                  <span className="text-xs text-txt-primary group-hover:text-primary-gold transition-colors line-clamp-1 flex-1">{item.title}</span>
                  <span className="text-[10px] text-txt-muted whitespace-nowrap">{item.source}</span>
                  <ExternalLink size={10} className="text-txt-muted opacity-0 group-hover:opacity-100 transition-opacity mt-0.5 flex-shrink-0" />
                </div>
              ))}
            </div>
          ) : !stockNewsLoading ? (
            <div className="px-4 pb-3 text-xs text-txt-muted">未找到相关新闻</div>
          ) : null}
        </div>
      )}

      {/* Main Content */}
      <div className="flex-1 min-h-0 overflow-auto">
        {/* 实时快讯 */}
        {activeTab === 'telegraph' && (
          <TelegraphPanel
            items={telegraphs}
            loading={telegraphLoading}
            formatTime={formatTime}
            importanceBadge={importanceBadge}
            openUrl={openUrl}
          />
        )}

        {/* 财经要闻 */}
        {activeTab === 'news' && (
          <NewsPanel
            items={newsList}
            loading={newsLoading}
            formatTime={formatTime}
            openUrl={openUrl}
            onLoadMore={() => fetchNews(true)}
          />
        )}

        {/* 公司公告 */}
        {activeTab === 'announcement' && (
          <AnnouncementPanel
            items={announcements}
            loading={announcementLoading}
            stockFilter={annStockFilter}
            onStockFilterChange={(v) => { setAnnStockFilter(v); fetchAnnouncements(v || undefined); }}
            openUrl={openUrl}
            onLoadMore={() => fetchAnnouncements(annStockFilter || undefined, true)}
          />
        )}

        {/* 研报 */}
        {activeTab === 'report' && (
          <ReportPanel
            items={reports}
            loading={reportLoading}
            stockFilter={reportStockFilter}
            onStockFilterChange={(v) => { setReportStockFilter(v); fetchReports(v || undefined); }}
            openUrl={openUrl}
            onLoadMore={() => fetchReports(reportStockFilter || undefined, true)}
          />
        )}
      </div>
    </div>
  );
}

// ============================================================
// 来源配色和快讯来源配置
// ============================================================

type TelegraphSource = 'all' | 'ClsTelegraph' | 'Sina7x24' | 'WallStreetCn';

const TELEGRAPH_SOURCE_CONFIG: { key: TelegraphSource; label: string; color: string; dotColor: string; bgColor: string; borderColor: string }[] = [
  { key: 'all', label: '全部', color: 'text-txt-secondary', dotColor: '', bgColor: '', borderColor: '' },
  { key: 'ClsTelegraph', label: '财联社', color: 'text-yellow-400', dotColor: 'bg-yellow-400', bgColor: 'bg-yellow-500/10', borderColor: 'border-yellow-500/20' },
  { key: 'Sina7x24', label: '新浪7x24', color: 'text-orange-400', dotColor: 'bg-orange-400', bgColor: 'bg-orange-500/10', borderColor: 'border-orange-500/20' },
  { key: 'WallStreetCn', label: '华尔街见闻', color: 'text-cyan-400', dotColor: 'bg-cyan-400', bgColor: 'bg-cyan-500/10', borderColor: 'border-cyan-500/20' },
];

function getSourceStyle(category: string) {
  switch (category) {
    case 'ClsTelegraph': return { label: '财联社', color: 'text-yellow-400', bg: 'bg-yellow-500/10', border: 'border-yellow-500/20', dotColor: 'bg-yellow-400' };
    case 'Sina7x24': return { label: '新浪7x24', color: 'text-orange-400', bg: 'bg-orange-500/10', border: 'border-orange-500/20', dotColor: 'bg-orange-400' };
    case 'WallStreetCn': return { label: '华尔街见闻', color: 'text-cyan-400', bg: 'bg-cyan-500/10', border: 'border-cyan-500/20', dotColor: 'bg-cyan-400' };
    default: return { label: '未知', color: 'text-txt-muted', bg: 'bg-bg-elevated', border: 'border-[#484F58]', dotColor: 'bg-[#484F58]' };
  }
}

function TelegraphPanel({ items, loading, formatTime, importanceBadge, openUrl }: {
  items: { id: string; title: string; summary: string; publish_time: string; importance: number; related_stocks: string[]; url: string; source: string; category: NewsCategory }[];
  loading: boolean;
  formatTime: (t: string) => string;
  importanceBadge: (level: number) => React.ReactNode;
  openUrl: (url: string) => void;
}) {
  const [sourceFilter, setSourceFilter] = useState<TelegraphSource>('all');

  const filteredItems = sourceFilter === 'all'
    ? items
    : items.filter((item) => item.category === sourceFilter);

  // 各来源计数
  const counts: Record<TelegraphSource, number> = {
    all: items.length,
    ClsTelegraph: items.filter(i => i.category === 'ClsTelegraph').length,
    Sina7x24: items.filter(i => i.category === 'Sina7x24').length,
    WallStreetCn: items.filter(i => i.category === 'WallStreetCn').length,
  };

  if (loading && items.length === 0) {
    return (
      <div className="flex items-center justify-center h-40">
        <Loader2 size={20} className="animate-spin text-primary-gold" />
        <span className="ml-2 text-xs text-txt-muted">加载快讯中...</span>
      </div>
    );
  }

  return (
    <div className="relative">
      {/* 来源筛选栏 */}
      <div className="flex items-center gap-2 px-4 py-2 border-b border-[#30363D] bg-bg-elevated/30 flex-shrink-0 sticky top-0 z-10">
        <Filter size={11} className="text-txt-muted flex-shrink-0" />
        <div className="flex items-center gap-1">
          {TELEGRAPH_SOURCE_CONFIG.map(({ key, label, color, dotColor }) => (
            <button
              key={key}
              onClick={() => setSourceFilter(key)}
              className={`flex items-center gap-1 px-2 py-0.5 text-[11px] rounded-md transition-all cursor-pointer ${
                sourceFilter === key
                  ? `${color} bg-bg-card border border-[#484F58] font-medium`
                  : 'text-txt-muted hover:text-txt-secondary'
              }`}
            >
              {dotColor && <span className={`w-1.5 h-1.5 rounded-full ${dotColor} flex-shrink-0`} />}
              {label}
              <span className="text-[9px] opacity-60">{counts[key]}</span>
            </button>
          ))}
        </div>
        <span className="text-[10px] text-txt-muted ml-auto">
          共 {filteredItems.length} 条
        </span>
      </div>

      {/* 时间线 */}
      <div className="relative">
        <div className="absolute left-[72px] top-0 bottom-0 w-px bg-[#30363D]" />

        {filteredItems.map((item) => {
          const srcStyle = getSourceStyle(item.category);
          return (
            <div
              key={item.id}
              className={`flex gap-3 px-4 py-2.5 hover:bg-bg-elevated/50 transition-colors group cursor-pointer ${
                item.importance >= 2 ? 'bg-red-500/5' : item.importance >= 1 ? 'bg-orange-500/5' : ''
              }`}
              onClick={() => openUrl(item.url)}
            >
              {/* 时间 */}
              <div className="w-14 flex-shrink-0 text-right">
                <span className="text-[11px] text-txt-muted font-mono tabular-nums">
                  {formatTime(item.publish_time)}
                </span>
              </div>

              {/* 时间线节点 - 用来源色标 */}
              <div className="relative flex-shrink-0 w-2 flex items-start pt-1.5">
                <div className={`w-2 h-2 rounded-full ${
                  item.importance >= 2 ? 'bg-red-500 ring-2 ring-red-500/30' :
                  item.importance >= 1 ? 'bg-orange-400 ring-2 ring-orange-400/30' :
                  srcStyle.dotColor
                }`} />
              </div>

              {/* 内容 */}
              <div className="flex-1 min-w-0">
                <div className="flex items-start gap-1.5">
                  {/* 来源小标签 */}
                  <span className={`inline-flex items-center px-1 py-0 rounded text-[9px] leading-[16px] flex-shrink-0 mt-px border ${srcStyle.bg} ${srcStyle.color} ${srcStyle.border}`}>
                    {srcStyle.label}
                  </span>
                  {importanceBadge(item.importance)}
                  <p className={`text-xs leading-relaxed ${
                    item.importance >= 2 ? 'text-txt-primary font-semibold' :
                    item.importance >= 1 ? 'text-txt-primary font-medium' :
                    'text-txt-secondary'
                  } group-hover:text-txt-primary transition-colors`}>
                    {item.title || item.summary}
                  </p>
                </div>
                {item.title && item.summary && item.title !== item.summary && (
                  <p className="text-[11px] text-txt-muted mt-1 line-clamp-2 leading-relaxed ml-[calc(2rem+6px)]">
                    {item.summary}
                  </p>
                )}
                {/* 关联股票标签 */}
                {item.related_stocks.length > 0 && (
                  <div className="flex items-center gap-1 mt-1.5 flex-wrap ml-[calc(2rem+6px)]">
                    {item.related_stocks.slice(0, 5).map((code) => (
                      <span
                        key={code}
                        className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] bg-blue-500/10 text-blue-400 border border-blue-500/20"
                        onClick={(e) => e.stopPropagation()}
                      >
                        {code}
                      </span>
                    ))}
                    {item.related_stocks.length > 5 && (
                      <span className="text-[10px] text-txt-muted">+{item.related_stocks.length - 5}</span>
                    )}
                  </div>
                )}
              </div>

              <ExternalLink size={11} className="text-txt-muted opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0 mt-0.5" />
            </div>
          );
        })}

        {filteredItems.length === 0 && !loading && (
          <div className="flex items-center justify-center h-32 text-xs text-txt-muted">
            该来源暂无快讯
          </div>
        )}

        {loading && items.length > 0 && (
          <div className="flex items-center justify-center py-4">
            <Loader2 size={14} className="animate-spin text-txt-muted" />
          </div>
        )}
      </div>
    </div>
  );
}

function NewsPanel({ items, loading, formatTime, openUrl, onLoadMore }: {
  items: { id: string; title: string; summary: string; publish_time: string; source: string; url: string; category: string }[];
  loading: boolean;
  formatTime: (t: string) => string;
  openUrl: (url: string) => void;
  onLoadMore: () => void;
}) {
  if (loading && items.length === 0) {
    return (
      <div className="flex items-center justify-center h-40">
        <Loader2 size={20} className="animate-spin text-primary-gold" />
        <span className="ml-2 text-xs text-txt-muted">加载新闻中...</span>
      </div>
    );
  }

  const sourceColor = (cat: string) => {
    if (cat === 'EastmoneyNews') return 'text-blue-400 bg-blue-500/10 border-blue-500/20';
    return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20';
  };

  return (
    <div>
      {items.map((item) => (
        <div
          key={item.id}
          className="flex items-start gap-3 px-4 py-3 border-b border-[#30363D]/50 hover:bg-bg-elevated/50 transition-colors cursor-pointer group"
          onClick={() => openUrl(item.url)}
        >
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className={`inline-flex items-center px-1.5 py-0.5 rounded text-[10px] border ${sourceColor(item.category)}`}>
                {item.source || (item.category === 'EastmoneyNews' ? '东方财富' : '新浪财经')}
              </span>
              <span className="text-[10px] text-txt-muted flex items-center gap-1">
                <Clock size={9} />
                {formatTime(item.publish_time)}
              </span>
            </div>
            <h3 className="text-xs font-medium text-txt-primary group-hover:text-primary-gold transition-colors line-clamp-1">
              {item.title}
            </h3>
            {item.summary && (
              <p className="text-[11px] text-txt-muted mt-1 line-clamp-2 leading-relaxed">{item.summary}</p>
            )}
          </div>
          <ExternalLink size={11} className="text-txt-muted opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0 mt-1" />
        </div>
      ))}

      {items.length > 0 && (
        <button
          onClick={(e) => { e.stopPropagation(); onLoadMore(); }}
          disabled={loading}
          className="w-full py-3 text-xs text-txt-muted hover:text-primary-gold transition-colors flex items-center justify-center gap-1 cursor-pointer"
        >
          {loading ? <Loader2 size={12} className="animate-spin" /> : <ChevronDown size={12} />}
          {loading ? '加载中...' : '加载更多'}
        </button>
      )}
    </div>
  );
}

function AnnouncementPanel({ items, loading, stockFilter, onStockFilterChange, openUrl, onLoadMore }: {
  items: { id: string; title: string; stock_code: string; stock_name: string; notice_date: string; category: string; url: string }[];
  loading: boolean;
  stockFilter: string;
  onStockFilterChange: (v: string) => void;
  openUrl: (url: string) => void;
  onLoadMore: () => void;
}) {
  return (
    <div>
      {/* 筛选栏 */}
      <div className="flex items-center gap-2 px-4 py-2 border-b border-[#30363D] bg-bg-elevated/30">
        <Building2 size={12} className="text-txt-muted" />
        <input
          type="text"
          placeholder="按股票代码筛选公告..."
          value={stockFilter}
          onChange={(e) => onStockFilterChange(e.target.value)}
          className="w-40 px-2 py-1 rounded bg-bg-base border border-[#30363D] text-xs text-txt-primary placeholder:text-txt-muted outline-none focus:border-primary-gold/50"
        />
        {stockFilter && (
          <button onClick={() => onStockFilterChange('')} className="text-xs text-txt-muted hover:text-txt-primary cursor-pointer">清除</button>
        )}
        <span className="text-[10px] text-txt-muted ml-auto">{items.length} 条公告</span>
      </div>

      {loading && items.length === 0 ? (
        <div className="flex items-center justify-center h-40">
          <Loader2 size={20} className="animate-spin text-primary-gold" />
          <span className="ml-2 text-xs text-txt-muted">加载公告中...</span>
        </div>
      ) : (
        <>
          {items.map((item) => (
            <div
              key={item.id}
              className="flex items-start gap-3 px-4 py-2.5 border-b border-[#30363D]/50 hover:bg-bg-elevated/50 transition-colors cursor-pointer group"
              onClick={() => openUrl(item.url)}
            >
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  {item.stock_name && (
                    <span className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-green-500/10 text-green-400 border border-green-500/20">
                      {item.stock_name}
                    </span>
                  )}
                  {item.stock_code && (
                    <span className="text-[10px] text-txt-muted font-mono">{item.stock_code}</span>
                  )}
                  {item.category && (
                    <Tooltip title={item.category}>
                      <span className="inline-flex items-center gap-0.5 text-[10px] text-txt-muted">
                        <Tag size={8} />
                        {item.category.length > 8 ? item.category.slice(0, 8) + '...' : item.category}
                      </span>
                    </Tooltip>
                  )}
                  <span className="text-[10px] text-txt-muted ml-auto flex items-center gap-1">
                    <Clock size={9} />
                    {item.notice_date?.slice(0, 10)}
                  </span>
                </div>
                <h3 className="text-xs text-txt-primary group-hover:text-primary-gold transition-colors line-clamp-2">
                  {item.title}
                </h3>
              </div>
              <ExternalLink size={11} className="text-txt-muted opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0 mt-1" />
            </div>
          ))}

          {items.length > 0 && (
            <button
              onClick={onLoadMore}
              disabled={loading}
              className="w-full py-3 text-xs text-txt-muted hover:text-primary-gold transition-colors flex items-center justify-center gap-1 cursor-pointer"
            >
              {loading ? <Loader2 size={12} className="animate-spin" /> : <ChevronDown size={12} />}
              {loading ? '加载中...' : '加载更多'}
            </button>
          )}
        </>
      )}
    </div>
  );
}

function ReportPanel({ items, loading, stockFilter, onStockFilterChange, openUrl, onLoadMore }: {
  items: { title: string; stock_code: string; stock_name: string; org_name: string; publish_date: string; rating: string; researcher: string; industry: string; url: string }[];
  loading: boolean;
  stockFilter: string;
  onStockFilterChange: (v: string) => void;
  openUrl: (url: string) => void;
  onLoadMore: () => void;
}) {
  const ratingColor = (r: string) => {
    if (r === '买入' || r === '强烈推荐') return 'text-red-400 bg-red-500/10 border-red-500/20';
    if (r === '增持' || r === '推荐') return 'text-orange-400 bg-orange-500/10 border-orange-500/20';
    if (r === '持有' || r === '中性') return 'text-yellow-400 bg-yellow-500/10 border-yellow-500/20';
    if (r === '减持' || r === '卖出') return 'text-green-400 bg-green-500/10 border-green-500/20';
    return 'text-txt-muted bg-bg-elevated border-[#484F58]';
  };

  return (
    <div>
      {/* 筛选栏 */}
      <div className="flex items-center gap-2 px-4 py-2 border-b border-[#30363D] bg-bg-elevated/30">
        <Search size={12} className="text-txt-muted" />
        <input
          type="text"
          placeholder="按股票代码筛选研报..."
          value={stockFilter}
          onChange={(e) => onStockFilterChange(e.target.value)}
          className="w-40 px-2 py-1 rounded bg-bg-base border border-[#30363D] text-xs text-txt-primary placeholder:text-txt-muted outline-none focus:border-primary-gold/50"
        />
        {stockFilter && (
          <button onClick={() => onStockFilterChange('')} className="text-xs text-txt-muted hover:text-txt-primary cursor-pointer">清除</button>
        )}
        <span className="text-[10px] text-txt-muted ml-auto">{items.length} 篇研报</span>
      </div>

      {loading && items.length === 0 ? (
        <div className="flex items-center justify-center h-40">
          <Loader2 size={20} className="animate-spin text-primary-gold" />
          <span className="ml-2 text-xs text-txt-muted">加载研报中...</span>
        </div>
      ) : (
        <>
          {items.map((item, idx) => (
            <div
              key={`${item.title}-${idx}`}
              className="flex items-start gap-3 px-4 py-2.5 border-b border-[#30363D]/50 hover:bg-bg-elevated/50 transition-colors cursor-pointer group"
              onClick={() => openUrl(item.url)}
            >
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1 flex-wrap">
                  {item.stock_name && (
                    <span className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-purple-500/10 text-purple-400 border border-purple-500/20">
                      {item.stock_name}
                    </span>
                  )}
                  {item.rating && (
                    <span className={`inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded text-[10px] font-bold border ${ratingColor(item.rating)}`}>
                      <Star size={8} />
                      {item.rating}
                    </span>
                  )}
                  <span className="text-[10px] text-txt-muted flex items-center gap-1">
                    <Building2 size={9} />
                    {item.org_name}
                  </span>
                  {item.industry && (
                    <span className="text-[10px] text-txt-muted">{item.industry}</span>
                  )}
                  <span className="text-[10px] text-txt-muted ml-auto">
                    {item.publish_date?.slice(0, 10)}
                  </span>
                </div>
                <h3 className="text-xs text-txt-primary group-hover:text-primary-gold transition-colors line-clamp-2">
                  {item.title}
                </h3>
                {item.researcher && (
                  <p className="text-[10px] text-txt-muted mt-0.5">分析师: {item.researcher}</p>
                )}
              </div>
              <ExternalLink size={11} className="text-txt-muted opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0 mt-1" />
            </div>
          ))}

          {items.length > 0 && (
            <button
              onClick={onLoadMore}
              disabled={loading}
              className="w-full py-3 text-xs text-txt-muted hover:text-primary-gold transition-colors flex items-center justify-center gap-1 cursor-pointer"
            >
              {loading ? <Loader2 size={12} className="animate-spin" /> : <ChevronDown size={12} />}
              {loading ? '加载中...' : '加载更多'}
            </button>
          )}
        </>
      )}
    </div>
  );
}
