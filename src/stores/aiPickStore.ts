import { create } from 'zustand';
import { safeInvoke, safeListen } from '../hooks/useTauri';
import { AIPickRecommendation, AIStreamEvent } from '../types';

interface ToolCallStatus {
  name: string;
  label: string;
  status: 'pending' | 'loading' | 'done';
  summary?: string;
}

interface ThinkingStep {
  content: string;
  timestamp: number;
}

interface AIPickState {
  // State
  picking: boolean;
  aiContent: string;
  recommendations: AIPickRecommendation[];
  toolCalls: ToolCallStatus[];
  thinkingSteps: ThinkingStep[];
  error: string | null;
  cachedContent: string | null;
  tokenUsage: number | null;

  // Similar stocks state
  similarLoading: boolean;
  similarTarget: { code: string; name: string; sector: string } | null;
  similarContent: string;
  similarPicks: AIPickRecommendation[];
  similarToolCalls: ToolCallStatus[];
  similarThinkingSteps: ThinkingStep[];
  similarError: string | null;

  // Actions
  startPick: () => Promise<void>;
  stopPick: () => Promise<void>;
  loadCachedPicks: () => Promise<void>;
  reset: () => void;
  findSimilarStocks: (code: string, name: string, sector: string) => Promise<void>;
  closeSimilar: () => void;
}

function parseRecommendations(content: string): AIPickRecommendation[] {
  const picksMatch = content.match(/<PICKS>\s*([\s\S]*?)\s*<\/PICKS>/);
  if (!picksMatch) return [];
  try {
    const jsonStr = picksMatch[1].trim();
    const parsed = JSON.parse(jsonStr);
    if (!Array.isArray(parsed)) {
      console.warn('[AI Pick] PICKS 标签内容不是 JSON 数组:', jsonStr.slice(0, 200));
      return [];
    }
    const valid = parsed.filter(
      (item: AIPickRecommendation) => item.code && item.name && item.reason && item.rating,
    );
    if (valid.length === 0 && parsed.length > 0) {
      console.warn('[AI Pick] PICKS 数组中无有效推荐项，缺少必要字段(code/name/reason/rating)');
    }
    return valid;
  } catch (e) {
    console.warn('[AI Pick] PICKS JSON 解析失败:', e, '原始内容:', picksMatch[1]?.slice(0, 300));
    return [];
  }
}

const TOOL_LABELS: Record<string, string> = {
  get_market_news: '获取市场新闻',
  get_economic_data: '宏观经济数据',
  get_global_indexes: '全球指数',
  get_financial_calendar: '财经日历',
  search_stocks_by_condition: 'NLP智能选股',
  search_concept_boards: 'NLP板块搜索',
  batch_get_stock_quotes: '批量查看行情',
  get_stock_quote: '查看个股行情',
  get_fund_flow: '查看资金流向',
  get_kline_data: '获取K线数据',
  get_technical_indicators: '获取技术指标',
  search_stock_news: '个股新闻搜索',
  get_stock_notices: '公司公告',
  get_industry_report: '研报摘要',
};

export const useAIPickStore = create<AIPickState>((set, get) => ({
  picking: false,
  aiContent: '',
  recommendations: [],
  toolCalls: [],
  thinkingSteps: [],
  error: null,
  cachedContent: null,
  tokenUsage: null,

  // Similar stocks
  similarLoading: false,
  similarTarget: null,
  similarContent: '',
  similarPicks: [],
  similarToolCalls: [],
  similarThinkingSteps: [],
  similarError: null,

  startPick: async () => {
    set({
      picking: true,
      aiContent: '',
      recommendations: [],
      toolCalls: [],
      thinkingSteps: [],
      error: null,
      tokenUsage: null,
    });

    const unlisten = await safeListen<AIStreamEvent>('ai-pick-stream', (event) => {
      const data = event.payload;
      const state = get();

      if (data.event_type === 'thinking') {
        set({
          thinkingSteps: [
            ...state.thinkingSteps,
            { content: data.content || '', timestamp: Date.now() },
          ],
        });
      } else if (data.event_type === 'content') {
        const newContent = state.aiContent + (data.content || '');
        set({ aiContent: newContent });
      } else if (data.event_type === 'tool_call') {
        const toolName = data.tool_name || '';
        const existing = state.toolCalls.filter((t) => t.name !== toolName);
        set({
          toolCalls: [
            ...existing,
            {
              name: toolName,
              label: TOOL_LABELS[toolName] || toolName,
              status: 'loading',
            },
          ],
        });
      } else if (data.event_type === 'tool_result') {
        const toolName = data.tool_name || '';
        const summary = data.content || '';
        set({
          toolCalls: state.toolCalls.map((t) =>
            t.name === toolName ? { ...t, status: 'done' as const, summary } : t,
          ),
        });
      } else if (data.event_type === 'done') {
        const fullContent = data.content || state.aiContent;
        let picks: AIPickRecommendation[] = [];
        try { picks = parseRecommendations(fullContent); } catch { /* ignore */ }
        const parseWarning = picks.length === 0 && fullContent.includes('<PICKS')
          ? 'AI 推荐结果解析异常，请查看原始分析报告'
          : null;
        set({
          picking: false,
          aiContent: fullContent || state.aiContent,
          recommendations: picks,
          tokenUsage: data.usage?.total_tokens || null,
          error: parseWarning,
        });
        unlisten();
      } else if (data.event_type === 'error') {
        set({
          picking: false,
          error: data.content || 'AI 选股失败',
        });
        unlisten();
      }
    });

    await safeInvoke('ai_pick_stocks').catch((e: Error) => {
      set({ picking: false, error: e.message });
      unlisten();
    });
  },

  stopPick: async () => {
    try {
      await safeInvoke('stop_ai_pick');
    } catch {
      // ignore — task may have already finished
    }
  },

  loadCachedPicks: async () => {
    const content = await safeInvoke<string | null>('get_cached_picks');
    if (content) {
      let picks: AIPickRecommendation[] = [];
      try { picks = parseRecommendations(content); } catch { /* ignore */ }
      const parseWarning = picks.length === 0 && content.includes('<PICKS')
        ? 'AI 推荐结果解析异常，请查看原始分析报告'
        : null;
      set({
        cachedContent: content,
        aiContent: content,
        recommendations: picks,
        error: parseWarning,
      });
    }
  },

  reset: () => {
    set({
      picking: false,
      aiContent: '',
      recommendations: [],
      toolCalls: [],
      thinkingSteps: [],
      error: null,
      tokenUsage: null,
    });
  },

  findSimilarStocks: async (code: string, name: string, sector: string) => {
    set({
      similarLoading: true,
      similarTarget: { code, name, sector },
      similarContent: '',
      similarPicks: [],
      similarToolCalls: [],
      similarThinkingSteps: [],
      similarError: null,
    });

    const eventName = `ai-similar-${code}`;
    const unlisten = await safeListen<AIStreamEvent>(eventName, (event) => {
      const data = event.payload;
      const state = get();

      if (data.event_type === 'thinking') {
        set({
          similarThinkingSteps: [
            ...state.similarThinkingSteps,
            { content: data.content || '', timestamp: Date.now() },
          ],
        });
      } else if (data.event_type === 'content') {
        set({ similarContent: state.similarContent + (data.content || '') });
      } else if (data.event_type === 'tool_call') {
        const toolName = data.tool_name || '';
        const existing = state.similarToolCalls.filter((t) => t.name !== toolName);
        set({
          similarToolCalls: [
            ...existing,
            { name: toolName, label: TOOL_LABELS[toolName] || toolName, status: 'loading' },
          ],
        });
      } else if (data.event_type === 'tool_result') {
        const toolName = data.tool_name || '';
        const summary = data.content || '';
        set({
          similarToolCalls: state.similarToolCalls.map((t) =>
            t.name === toolName ? { ...t, status: 'done' as const, summary } : t,
          ),
        });
      } else if (data.event_type === 'done') {
        const fullContent = data.content || state.similarContent;
        let picks: AIPickRecommendation[] = [];
        try { picks = parseRecommendations(fullContent); } catch { /* ignore */ }
        const parseWarning = picks.length === 0 && fullContent.includes('<PICKS')
          ? 'AI 推荐结果解析异常，请查看原始分析报告'
          : null;
        set({
          similarLoading: false,
          similarContent: fullContent || state.similarContent,
          similarPicks: picks,
          similarError: parseWarning,
        });
        unlisten();
      } else if (data.event_type === 'error') {
        set({
          similarLoading: false,
          similarError: data.content || '查找相似股失败',
        });
        unlisten();
      }
    });

    await safeInvoke('find_similar_stocks', { code, name, sector }).catch((e: Error) => {
      set({ similarLoading: false, similarError: e.message });
      unlisten();
    });
  },

  closeSimilar: () => {
    set({
      similarLoading: false,
      similarTarget: null,
      similarContent: '',
      similarPicks: [],
      similarToolCalls: [],
      similarThinkingSteps: [],
      similarError: null,
    });
  },
}));
