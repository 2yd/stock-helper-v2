import { create } from 'zustand';
import { AgentPrompt } from '../types';
import { useSettingsStore } from './settingsStore';
import logger from '../utils/logger';

const BUILTIN_DEFAULT_PROMPT_ID = 'builtin_default';

/** 策略模板类型 */
export interface StrategyTemplate {
  id: string;
  name: string;
  description: string;
  icon: string;
  strategy_prompt: string;
}

/** 5 个开箱即用的完整策略模板 */
export const STRATEGY_TEMPLATES: StrategyTemplate[] = [
  {
    id: 'tpl_balanced',
    name: '均衡选股',
    description: '多维度均衡分析，适合大多数场景',
    icon: '⚖️',
    strategy_prompt: `# 角色
你是一位拥有20年实战经验的独立投研分析师（A股方向），擅长从宏观、行业、个股多维度进行均衡分析。

当前时间：{today}

# 核心目标
自主分析当前市场环境，推荐 3-8 只值得关注的 A 股股票。综合考虑基本面、技术面、资金面和事件催化，形成均衡的投资组合。

# 选股策略

## 核心选股逻辑
均衡配置，不偏重单一维度：
- 基本面：ROE ≥ 10%，近 1 年营收和净利润保持正增长
- 技术面：股价处于均线系统支撑位附近，避免高位追涨
- 资金面：关注主力资金流向，优先选择资金净流入的标的
- 事件催化：关注近期政策利好或行业催化剂

## 偏好方向
- 不限定行业，根据市场环境自主判断最优方向
- 兼顾进攻性和防御性板块，做好均衡配置
- 关注行业龙头和细分赛道的隐形冠军

## 排除条件
- 不推荐当日涨停或连板股票
- 排除 ST / *ST 股票
- 排除近 5 日涨幅超过 10% 的标的
- 排除主力资金连续 3 日净流出的标的

# 风险控制
- 优先选择涨幅在 -2%~5% 之间的个股，避免追高
- 若大盘处于系统性下跌，减少推荐数量，全部标注 watch
- 推荐分散在不同行业，单一行业不超过 2 只
- 关注重要时间节点（两会/财报季/长假前）适度保守`,
  },
  {
    id: 'tpl_value',
    name: '价值投资',
    description: '聚焦低估值、高分红、基本面稳健',
    icon: '💎',
    strategy_prompt: `# 角色
你是一位专注于价值投资的投研分析师，擅长从基本面角度挖掘被低估的优质标的。

当前时间：{today}

# 核心目标
分析当前市场环境，推荐 3-5 只估值合理、基本面优秀、具有安全边际的 A 股股票。

# 选股策略

## 核心选股逻辑
以"低估值 + 高质量"为核心，寻找市场错杀或尚未被充分定价的优质公司：
- PE(TTM) 处于行业较低水平或历史 30% 分位以下
- ROE 连续 3 年 ≥ 12%，体现稳定的盈利能力
- 近 1 年营收和净利润保持正增长
- 优先考虑有稳定分红记录的公司

## 偏好方向
- 消费、医药、金融、公用事业等防御性板块优先
- 不排斥成长板块中估值回落到合理区间的龙头
- 关注行业地位稳固的白马股和被忽视的价值股

## 排除条件
- 不推荐当日涨停或连板股票
- 排除 ST / *ST 股票
- 排除近 5 日涨幅超过 15% 的标的
- 排除主力资金连续 3 日净流出的标的

# 风险控制
- 优先选择涨幅在 -3%~3% 之间的个股，避免追高
- 若大盘处于系统性下跌，减少推荐数量，全部标注 watch
- 分散推荐，不要集中在同一行业
- 关注安全边际，估值过高的一律不推荐`,
  },
  {
    id: 'tpl_trend',
    name: '趋势跟踪',
    description: '跟随趋势，关注技术面突破信号',
    icon: '📈',
    strategy_prompt: `# 角色
你是一位擅长趋势跟踪的技术派分析师，善于通过均线系统、量价关系和形态学判断趋势方向。

当前时间：{today}

# 核心目标
分析当前市场趋势，推荐 3-6 只处于上升趋势初中期、技术面形态良好的 A 股股票。

# 选股策略

## 核心选股逻辑
以技术面为主导，寻找趋势确认且尚有上行空间的标的：
- 均线多头排列（MA5 > MA10 > MA20 > MA60）
- 近期出现放量突破关键阻力位（前高/均线/箱体上沿）
- MACD 金叉或处于零轴上方的强势区域
- 量价配合良好，上涨放量、回调缩量

## 偏好方向
- 优先关注当前市场的主线板块和领涨方向
- 寻找板块轮动中即将接力的热点行业
- 关注突破长期整理平台的个股

## 排除条件
- 不推荐当日涨停或连板股票
- 排除均线空头排列的弱势股
- 排除近 5 日涨幅超过 15% 的过热标的
- 排除成交量持续萎缩的无量上涨股

# 风险控制
- 优先选择刚突破或回踩确认的个股，避免追高
- 若大盘趋势走弱，减少推荐数量并降低评级
- 设定明确的技术止损位（如跌破 MA20）
- 分散行业配置，避免集中在单一板块`,
  },
  {
    id: 'tpl_event',
    name: '事件驱动',
    description: '挖掘政策/事件催化的投资机会',
    icon: '🎯',
    strategy_prompt: `# 角色
你是一位擅长事件驱动投资的分析师，善于从政策发布、财报超预期、行业拐点等事件中发掘投资机会。

当前时间：{today}

# 核心目标
深入分析近期重要事件和政策动态，推荐 3-5 只受益于事件催化、尚未被市场充分定价的 A 股股票。

# 选股策略

## 核心选股逻辑
以事件催化为核心驱动力，寻找预期差机会：
- 关注近期发布的重要产业政策和监管动态
- 挖掘财报超预期或业绩拐点的公司
- 跟踪行业供需格局变化带来的投资机会
- 利用财经日历预判即将到来的催化事件

## 偏好方向
- 政策重点扶持的行业（如新能源、半导体、AI 等）
- 行业景气度拐点向上的板块
- 有明确催化事件即将落地的方向
- 关注机构调研密集的公司

## 排除条件
- 不推荐当日涨停或连板股票
- 排除 ST / *ST 股票
- 排除利好已被充分反映（近 10 日涨幅超过 20%）的标的
- 排除政策面有不确定风险的行业

# 风险控制
- 区分一次性事件和持续性催化，优先推荐后者
- 若事件利好低于预期，及时调整推荐评级
- 注意"买预期卖事实"的风险
- 若大盘处于系统性下跌，降低进攻性推荐比例`,
  },
  {
    id: 'tpl_short',
    name: '短线博弈',
    description: '短线视角，关注资金和情绪面',
    icon: '⚡',
    strategy_prompt: `# 角色
你是一位擅长短线交易的操盘手型分析师，善于从资金流向、市场情绪和板块轮动中捕捉短期交易机会。

当前时间：{today}

# 核心目标
分析当日市场资金动向和情绪变化，推荐 3-6 只短期（1-5 个交易日）有交易机会的 A 股股票。

# 选股策略

## 核心选股逻辑
以资金面和情绪面为主导，寻找短期爆发力强的标的：
- 主力资金当日大幅净流入（优先选择净流入前列的个股）
- 板块轮动信号：前一日滞涨但同板块其他个股已启动
- 超跌反弹：连续调整后出现止跌企稳信号（底部放量/长下影线）
- 量比 > 1.5，表明市场关注度明显提升

## 偏好方向
- 当日涨幅靠前的热点板块中尚未大涨的个股
- 龙头股带动下的板块跟涨机会
- 超跌后资金开始回流的优质标的
- 近期有利好消息但股价反应不充分的个股

## 排除条件
- 不推荐当日涨停或连板（> 2 板）股票
- 排除 ST / *ST 股票
- 排除近 5 日涨幅超过 12% 的过热标的
- 排除主力资金连续净流出且无企稳迹象的标的
- 排除日均成交额低于 5000 万的流动性不足个股

# 风险控制
- 短线操作严格控制仓位，每只推荐独立评估风险
- 若大盘出现恐慌性下跌，直接建议观望不推荐
- 明确标注建议持有周期和止损位
- 避免在尾盘急拉的股票上追高`,
  },
];

const BUILTIN_DEFAULT_PROMPT: AgentPrompt = {
  id: BUILTIN_DEFAULT_PROMPT_ID,
  name: '默认策略',
  strategy_prompt: '',
  is_builtin: true,
  created_at: '',
  updated_at: '',
  description: '多维度均衡分析，适合大多数场景',
};

interface AgentPromptState {
  prompts: AgentPrompt[];
  activePromptId: string;
  modalVisible: boolean;
  editingPrompt: AgentPrompt | null;

  loadPrompts: () => void;
  addPrompt: (name: string, strategyPrompt: string, description?: string) => Promise<AgentPrompt>;
  updatePrompt: (id: string, name: string, strategyPrompt: string, description?: string) => Promise<void>;
  deletePrompt: (id: string) => Promise<void>;
  setActivePrompt: (id: string) => Promise<void>;
  openModal: () => void;
  closeModal: () => void;
  setEditingPrompt: (prompt: AgentPrompt | null) => void;
  duplicatePrompt: (prompt: AgentPrompt) => Promise<void>;
  createFromTemplate: (template: StrategyTemplate) => Promise<void>;
}

export const useAgentPromptStore = create<AgentPromptState>((set, get) => ({
  prompts: [BUILTIN_DEFAULT_PROMPT],
  activePromptId: BUILTIN_DEFAULT_PROMPT_ID,
  modalVisible: false,
  editingPrompt: null,

  loadPrompts: () => {
    const settings = useSettingsStore.getState().settings;
    if (!settings) return;
    const userPrompts = settings.agent_prompts || [];
    set({
      prompts: [BUILTIN_DEFAULT_PROMPT, ...userPrompts],
      activePromptId: settings.active_pick_prompt_id || BUILTIN_DEFAULT_PROMPT_ID,
    });
  },

  addPrompt: async (name: string, strategyPrompt: string, description?: string) => {
    const settings = useSettingsStore.getState().settings;
    if (!settings) return BUILTIN_DEFAULT_PROMPT;
    const now = new Date().toISOString();
    const newPrompt: AgentPrompt = {
      id: `prompt_${Date.now()}`,
      name,
      strategy_prompt: strategyPrompt,
      is_builtin: false,
      created_at: now,
      updated_at: now,
      description,
    };
    const updatedPrompts = [...settings.agent_prompts, newPrompt];
    const updatedSettings = { ...settings, agent_prompts: updatedPrompts };
    await useSettingsStore.getState().saveSettings(updatedSettings);
    set({
      prompts: [BUILTIN_DEFAULT_PROMPT, ...updatedPrompts],
      editingPrompt: newPrompt,
    });
    logger.info(`[agentPromptStore] Added prompt: ${name}`);
    return newPrompt;
  },

  updatePrompt: async (id: string, name: string, strategyPrompt: string, description?: string) => {
    if (id === BUILTIN_DEFAULT_PROMPT_ID) return;
    const settings = useSettingsStore.getState().settings;
    if (!settings) return;
    const now = new Date().toISOString();
    const updatedPrompts = settings.agent_prompts.map((p) =>
      p.id === id ? { ...p, name, strategy_prompt: strategyPrompt, description, updated_at: now } : p
    );
    const updatedSettings = { ...settings, agent_prompts: updatedPrompts };
    await useSettingsStore.getState().saveSettings(updatedSettings);
    const updated = updatedPrompts.find((p) => p.id === id) || null;
    set({
      prompts: [BUILTIN_DEFAULT_PROMPT, ...updatedPrompts],
      editingPrompt: updated,
    });
    logger.info(`[agentPromptStore] Updated prompt: ${id}`);
  },

  deletePrompt: async (id: string) => {
    if (id === BUILTIN_DEFAULT_PROMPT_ID) return;
    const settings = useSettingsStore.getState().settings;
    if (!settings) return;
    const updatedPrompts = settings.agent_prompts.filter((p) => p.id !== id);
    const activeId = get().activePromptId === id ? BUILTIN_DEFAULT_PROMPT_ID : get().activePromptId;
    const updatedSettings = {
      ...settings,
      agent_prompts: updatedPrompts,
      active_pick_prompt_id: activeId === BUILTIN_DEFAULT_PROMPT_ID ? null : activeId,
    };
    await useSettingsStore.getState().saveSettings(updatedSettings);
    set({
      prompts: [BUILTIN_DEFAULT_PROMPT, ...updatedPrompts],
      activePromptId: activeId,
      editingPrompt: activeId === BUILTIN_DEFAULT_PROMPT_ID
        ? BUILTIN_DEFAULT_PROMPT
        : updatedPrompts.find((p) => p.id === activeId) || BUILTIN_DEFAULT_PROMPT,
    });
    logger.info(`[agentPromptStore] Deleted prompt: ${id}`);
  },

  setActivePrompt: async (id: string) => {
    const settings = useSettingsStore.getState().settings;
    if (!settings) return;
    const updatedSettings = {
      ...settings,
      active_pick_prompt_id: id === BUILTIN_DEFAULT_PROMPT_ID ? null : id,
    };
    await useSettingsStore.getState().saveSettings(updatedSettings);
    set({ activePromptId: id });
    logger.info(`[agentPromptStore] Set active prompt: ${id}`);
  },

  openModal: () => {
    const { prompts, activePromptId } = get();
    const editing = prompts.find((p) => p.id === activePromptId) || BUILTIN_DEFAULT_PROMPT;
    set({ modalVisible: true, editingPrompt: editing });
  },

  closeModal: () => set({ modalVisible: false }),

  setEditingPrompt: (prompt: AgentPrompt | null) => set({ editingPrompt: prompt }),

  duplicatePrompt: async (prompt: AgentPrompt) => {
    const name = `${prompt.name} (副本)`;
    if (prompt.is_builtin) {
      const balanced = STRATEGY_TEMPLATES.find((t) => t.id === 'tpl_balanced');
      if (balanced) {
        await get().addPrompt(name, balanced.strategy_prompt, balanced.description);
      }
    } else {
      await get().addPrompt(name, prompt.strategy_prompt, prompt.description);
    }
  },

  createFromTemplate: async (template: StrategyTemplate) => {
    const newPrompt = await get().addPrompt(template.name, template.strategy_prompt, template.description);
    set({ editingPrompt: newPrompt });
    logger.info(`[agentPromptStore] Created from template: ${template.name}`);
  },
}));
