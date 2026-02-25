import { useState, useCallback, useEffect, useRef } from 'react';
import { Input, Table, App, Spin } from 'antd';
import { Search, Sparkles, TrendingUp, Flame } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { safeInvoke as invoke, safeListen } from '../hooks/useTauri';
import { HotStrategyItem, SmartStockColumn, AIStreamEvent } from '../types';

const { Search: AntSearch } = Input;

interface SearchResultData {
  columns: SmartStockColumn[];
  dataList: Record<string, unknown>[];
  traceInfo?: string;
}

export default function SmartStock() {
  const { message } = App.useApp();

  const [keyword, setKeyword] = useState('');
  const [loading, setLoading] = useState(false);
  const [hotStrategies, setHotStrategies] = useState<HotStrategyItem[]>([]);
  const [hotLoading, setHotLoading] = useState(false);
  const [resultData, setResultData] = useState<SearchResultData | null>(null);

  // AI smart pick state
  const [aiPicking, setAiPicking] = useState(false);
  const [aiContent, setAiContent] = useState('');
  const [aiResultData, setAiResultData] = useState<SearchResultData | null>(null);
  const aiContentRef = useRef('');
  const aiStreamPanelRef = useRef<HTMLDivElement>(null);

  // Load hot strategies on mount
  useEffect(() => {
    loadHotStrategies();
  }, []);

  const loadHotStrategies = useCallback(async () => {
    setHotLoading(true);
    try {
      const data = await invoke<HotStrategyItem[]>('get_hot_strategies');
      setHotStrategies(data || []);
      // Auto-search with first hot strategy
      if (data && data.length > 0 && !keyword) {
        setKeyword(data[0].question);
        doSearch(data[0].question);
      }
    } catch (e) {
      console.error('Failed to load hot strategies:', e);
    } finally {
      setHotLoading(false);
    }
  }, []);

  const doSearch = useCallback(async (searchKeyword: string) => {
    if (!searchKeyword.trim()) {
      message.warning('请输入选股条件');
      return;
    }
    setLoading(true);
    setResultData(null);
    try {
      const resp = await invoke<{
        code: number;
        msg?: string;
        message?: string;
        data?: {
          result: {
            columns: SmartStockColumn[];
            dataList: Record<string, unknown>[];
          };
          traceInfo?: { showText: string };
        };
      }>('smart_search_stock', { keyword: searchKeyword, pageSize: 50 });

      if (resp.code === 100 && resp.data) {
        setResultData({
          columns: resp.data.result.columns,
          dataList: resp.data.result.dataList,
          traceInfo: resp.data.traceInfo?.showText,
        });
      } else {
        const errMsg = resp.msg || resp.message || '搜索失败';
        message.error(errMsg);
      }
    } catch (e: unknown) {
      message.error(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const handleSearch = useCallback(() => {
    doSearch(keyword);
  }, [keyword, doSearch]);

  const handleHotClick = useCallback((question: string) => {
    setKeyword(question);
    doSearch(question);
  }, [doSearch]);

  // AI Smart Pick
  const handleAiPick = useCallback(async () => {
    setAiPicking(true);
    setAiContent('');
    setAiResultData(null);
    aiContentRef.current = '';

    // Listen for stream events
    const unlisten = await safeListen<AIStreamEvent>('ai-smart-pick-stream', (event) => {
      const data = event.payload;
      if (data.event_type === 'content' && data.content) {
        aiContentRef.current += data.content;
        setAiContent(aiContentRef.current);
        // Auto-scroll
        if (aiStreamPanelRef.current) {
          aiStreamPanelRef.current.scrollTop = aiStreamPanelRef.current.scrollHeight;
        }
      } else if (data.event_type === 'search_result' && data.content) {
        try {
          const searchResp = JSON.parse(data.content);
          if (searchResp.code === 100 && searchResp.data) {
            setAiResultData({
              columns: searchResp.data.result.columns,
              dataList: searchResp.data.result.dataList,
              traceInfo: searchResp.data.trace_info?.show_text,
            });
          }
        } catch {}
      } else if (data.event_type === 'done') {
        setAiPicking(false);
        unlisten();
      }
    });

    try {
      await invoke('ai_smart_pick');
    } catch (e: unknown) {
      message.error(String(e));
      setAiPicking(false);
      unlisten();
    }
  }, []);

  // Build dynamic Ant Table columns from API response
  const buildTableColumns = useCallback((cols: SmartStockColumn[]) => {
    const antColumns: Array<{
      title: string;
      dataIndex: string;
      key: string;
      width?: number;
      fixed?: 'left' | 'right';
      sorter?: (a: Record<string, unknown>, b: Record<string, unknown>) => number;
      render?: (val: unknown) => React.ReactNode;
      children?: Array<{
        title: string;
        dataIndex: string;
        key: string;
        width?: number;
        sorter?: (a: Record<string, unknown>, b: Record<string, unknown>) => number;
        render?: (val: unknown) => React.ReactNode;
      }>;
    }> = [];

    for (const col of cols) {
      if (col.hidden_need) continue;
      if (col.key === 'MARKET_SHORT_NAME') continue;

      const title = col.title + (col.unit ? `[${col.unit}]` : '');

      if (col.children && col.children.length > 0) {
        const children = col.children
          .filter(c => !c.hidden_need)
          .map(child => ({
            title: child.date_msg || child.title,
            dataIndex: child.key,
            key: child.key,
            width: 100,
            sorter: (a: Record<string, unknown>, b: Record<string, unknown>) => {
              const va = Number(a[child.key]);
              const vb = Number(b[child.key]);
              if (isNaN(va) || isNaN(vb)) return 0;
              return va - vb;
            },
            render: (val: unknown) => renderCellValue(val),
          }));
        antColumns.push({ title, dataIndex: col.key, key: col.key, width: 200, children });
      } else {
        const isCode = col.key === 'SECURITY_CODE' || col.key === 'SERIAL';
        const isName = col.key === 'SECURITY_SHORT_NAME';

        antColumns.push({
          title,
          dataIndex: col.key,
          key: col.key,
          width: isName ? 100 : isCode ? 80 : 120,
          fixed: isCode || isName ? 'left' as const : undefined,
          sorter: isCode || isName ? undefined : (a: Record<string, unknown>, b: Record<string, unknown>) => {
            const va = Number(a[col.key]);
            const vb = Number(b[col.key]);
            if (isNaN(va) || isNaN(vb)) return 0;
            return va - vb;
          },
          render: isName
            ? (val: unknown) => <span className="text-blue-400 font-medium">{String(val ?? '')}</span>
            : isCode
            ? (val: unknown) => <span className="text-txt-muted font-mono text-xs">{String(val ?? '')}</span>
            : (val: unknown) => renderCellValue(val),
        });
      }
    }

    return antColumns;
  }, []);

  const renderCellValue = (val: unknown) => {
    if (val === null || val === undefined || val === '-') {
      return <span className="text-txt-muted">-</span>;
    }
    const num = Number(val);
    if (!isNaN(num) && val !== '') {
      let color = 'text-blue-400';
      if (num < 0) color = 'text-green-400';
      else if (num > 5) color = 'text-red-400';
      else if (num >= 0 && num <= 5) color = 'text-orange-400';
      return <span className={color}>{String(val)}</span>;
    }
    return <span className="text-txt-secondary">{String(val)}</span>;
  };

  const tableColumns = resultData ? buildTableColumns(resultData.columns) : [];
  const aiTableColumns = aiResultData ? buildTableColumns(aiResultData.columns) : [];

  return (
    <div className="flex h-full overflow-hidden">
      {/* Left: Hot Strategies Sidebar */}
      <div className="w-56 border-r border-[#30363D] flex flex-col flex-shrink-0">
        <div className="flex items-center gap-2 px-3 py-2.5 border-b border-[#30363D]">
          <Flame size={14} className="text-orange-400" />
          <span className="text-xs font-bold text-txt-primary">热门策略</span>
          <button
            onClick={loadHotStrategies}
            className="ml-auto text-[10px] text-txt-muted hover:text-txt-primary cursor-pointer"
          >
            刷新
          </button>
        </div>
        <div className="flex-1 overflow-auto">
          {hotLoading ? (
            <div className="flex items-center justify-center py-8">
              <Spin size="small" />
            </div>
          ) : (
            <div className="py-1">
              {hotStrategies.map((item, index) => (
                <button
                  key={item.rank}
                  onClick={() => handleHotClick(item.question)}
                  className={`w-full text-left px-3 h-14 hover:bg-bg-elevated transition-colors cursor-pointer group flex items-center ${
                    index < hotStrategies.length - 1 ? 'border-b border-[#30363D]/50' : ''
                  }`}
                >
                  <div className="flex items-start gap-2 w-full">
                    <span className="text-[10px] text-blue-400 font-mono mt-0.5 flex-shrink-0 w-5 text-right">
                      #{item.rank}
                    </span>
                    <span className="text-xs text-txt-secondary group-hover:text-orange-300 line-clamp-2 leading-relaxed">
                      {item.question}
                    </span>
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Right: Main Content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Search Bar */}
        <div className="flex items-center gap-2 px-4 py-2.5 border-b border-[#30363D] bg-bg-elevated/50 flex-shrink-0">
          <AntSearch
            placeholder="输入选股条件，如：换手率>3%，PE<30，ROE>10%，不要ST股..."
            value={keyword}
            onChange={e => setKeyword(e.target.value)}
            onSearch={handleSearch}
            enterButton={
              <span className="flex items-center gap-1">
                <Search size={14} />
                搜索A股
              </span>
            }
            loading={loading}
            size="middle"
            style={{ flex: 1 }}
          />
          <button
            onClick={handleAiPick}
            disabled={aiPicking}
            className="flex items-center gap-1.5 px-4 py-1.5 text-xs font-medium bg-gradient-to-r from-purple-600 to-indigo-500 text-white rounded-md hover:from-purple-500 hover:to-indigo-400 transition-all cursor-pointer shadow-lg shadow-purple-500/20 disabled:opacity-50 flex-shrink-0"
          >
            <Sparkles size={14} className={aiPicking ? 'animate-pulse' : ''} />
            {aiPicking ? 'AI选股中...' : 'AI智能选股'}
          </button>
        </div>

        {/* Trace Info */}
        {resultData?.traceInfo && (
          <div className="px-4 py-1.5 text-xs border-b border-[#30363D] bg-bg-card/50 flex-shrink-0">
            <span className="text-txt-muted">选股条件：</span>
            <span className="text-orange-300">{resultData.traceInfo}</span>
            <span className="text-txt-muted ml-3">共 {resultData.dataList.length} 只</span>
          </div>
        )}

        {/* Content area - split between table and AI panel */}
        <div className="flex-1 flex overflow-hidden">
          {/* Table Area */}
          <div className={`flex-1 overflow-auto ${(aiContent || aiPicking) ? 'border-r border-[#30363D]' : ''}`}>
            {!resultData && !loading && !aiContent && !aiPicking && (
              <div className="flex-1 flex items-center justify-center h-full">
                <div className="text-center space-y-4">
                  <div className="text-5xl opacity-15">
                    <TrendingUp size={64} className="mx-auto" />
                  </div>
                  <div className="space-y-2">
                    <p className="text-txt-primary text-base font-medium">智能选股</p>
                    <p className="text-txt-muted text-sm max-w-md">
                      输入自然语言选股条件，东方财富服务端智能筛选。
                      <br />
                      或点击「AI智能选股」让AI自动分析市场热点，构造选股条件。
                    </p>
                  </div>
                  <div className="flex gap-3 justify-center">
                    <button
                      onClick={() => {
                        if (hotStrategies.length > 0) {
                          handleHotClick(hotStrategies[0].question);
                        }
                      }}
                      disabled={hotStrategies.length === 0}
                      className="px-6 py-2 text-sm font-medium bg-gradient-to-r from-blue-600 to-indigo-500 text-white rounded-lg hover:from-blue-500 hover:to-indigo-400 transition-all cursor-pointer shadow-xl shadow-blue-500/30 disabled:opacity-40"
                    >
                      <Search size={14} className="inline mr-1.5 -mt-0.5" />
                      试试热门策略
                    </button>
                    <button
                      onClick={handleAiPick}
                      disabled={aiPicking}
                      className="px-6 py-2 text-sm font-medium bg-gradient-to-r from-purple-600 to-indigo-500 text-white rounded-lg hover:from-purple-500 hover:to-indigo-400 transition-all cursor-pointer shadow-xl shadow-purple-500/30 disabled:opacity-40"
                    >
                      <Sparkles size={14} className="inline mr-1.5 -mt-0.5" />
                      AI智能选股
                    </button>
                  </div>
                </div>
              </div>
            )}

            {(resultData || loading) && (
              <Table
                columns={tableColumns}
                dataSource={resultData?.dataList || []}
                loading={loading}
                rowKey={(record) => String(record['SECURITY_CODE'] || record['SERIAL'] || Math.random())}
                size="small"
                pagination={{ pageSize: 20, showSizeChanger: true, pageSizeOptions: ['10', '20', '50'] }}
                scroll={{ x: Math.max(tableColumns.length * 120, 1200), y: 'calc(100vh - 220px)' }}
                className="smart-stock-table"
              />
            )}
          </div>

          {/* AI Stream Panel - shows when AI is working */}
          {(aiContent || aiPicking) && (
            <div className="w-[420px] flex flex-col flex-shrink-0 bg-bg-card">
              <div className="flex items-center gap-2 px-3 py-2 border-b border-[#30363D]">
                <Sparkles size={14} className="text-purple-400" />
                <span className="text-xs font-bold text-txt-primary">AI 智能选股分析</span>
                {aiPicking && <Spin size="small" className="ml-auto" />}
                {!aiPicking && (
                  <button
                    onClick={() => { setAiContent(''); setAiResultData(null); }}
                    className="ml-auto text-[10px] text-txt-muted hover:text-txt-primary cursor-pointer"
                  >
                    关闭
                  </button>
                )}
              </div>
              <div
                ref={aiStreamPanelRef}
                className="flex-1 overflow-auto px-4 py-3 prose prose-invert prose-sm max-w-none"
              >
                <ReactMarkdown remarkPlugins={[remarkGfm]}>
                  {aiContent}
                </ReactMarkdown>
              </div>

              {/* AI search result table */}
              {aiResultData && (
                <div className="border-t border-[#30363D] max-h-[300px] overflow-auto">
                  <div className="px-3 py-1.5 text-xs text-txt-muted border-b border-[#30363D]">
                    AI筛选出 {aiResultData.dataList.length} 只股票
                  </div>
                  <Table
                    columns={aiTableColumns}
                    dataSource={aiResultData.dataList}
                    rowKey={(record) => String(record['SECURITY_CODE'] || Math.random())}
                    size="small"
                    pagination={false}
                    scroll={{ x: Math.max(aiTableColumns.length * 100, 800), y: 250 }}
                  />
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
