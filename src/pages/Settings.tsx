import { useEffect, useState } from 'react';
import { Slider, Switch, Select, Input, InputNumber, App } from 'antd';
import { Plus, Trash2, Bot, Database, Sliders, Filter, Fingerprint, Loader2, CheckCircle, XCircle, Zap } from 'lucide-react';
import { useSettingsStore } from '../stores/settingsStore';
import { AIConfig } from '../types';

export default function Settings() {
  const { message } = App.useApp();
  const { settings, loadSettings, saveSettings, addAIConfig, removeAIConfig, updateAIConfig, setActiveAIConfig, updateStrategy, testAIConfig, testingConfigId } = useSettingsStore();

  const [testResults, setTestResults] = useState<Record<string, { ok: boolean; msg: string }>>({});

  useEffect(() => {
    loadSettings();
  }, []);

  if (!settings) {
    return <div className="flex items-center justify-center text-txt-muted" style={{ height: 'calc(100vh - 48px)' }}>加载中...</div>;
  }

  const handleAddModel = () => {
    const newConfig: AIConfig = {
      id: crypto.randomUUID(),
      name: `模型 ${settings.ai_configs.length + 1}`,
      base_url: 'https://api.openai.com/v1',
      api_key: '',
      model_name: 'gpt-4o-mini',
      max_tokens: 2048,
      temperature: 0.3,
      pick_temperature: 0.7,
      timeout_secs: 300,
      enabled: true,
    };
    addAIConfig(newConfig);
  };

  const handleDeleteConfig = (id: string) => {
    removeAIConfig(id);
    setTestResults(prev => { const n = { ...prev }; delete n[id]; return n; });
    message.success('模型已删除');
  };

  const handleTestConfig = async (config: AIConfig) => {
    if (!config.api_key || !config.base_url) {
      message.warning('请先填写 API 地址和 API Key');
      return;
    }
    setTestResults(prev => ({ ...prev, [config.id]: { ok: false, msg: '' } }));
    try {
      const result = await testAIConfig(config);
      setTestResults(prev => ({ ...prev, [config.id]: { ok: true, msg: result } }));
      message.success('连接测试成功');
    } catch (e: unknown) {
      const errMsg = e instanceof Error ? e.message : String(e);
      setTestResults(prev => ({ ...prev, [config.id]: { ok: false, msg: errMsg } }));
      message.error(`测试失败: ${errMsg}`);
    }
  };

  const activeStrategy = settings.strategies.find(s => s.id === settings.active_strategy_id);

  const inputStyle = { background: '#0D1117', borderColor: '#30363D', color: '#E6EDF3' };

  return (
    <div className="overflow-y-auto p-6 space-y-6" style={{ height: 'calc(100vh - 48px)' }}>
      {/* 东财用户标识 — 放最前面 */}
      <section>
        <div className="flex items-center gap-2 mb-4">
          <Fingerprint size={18} className="text-orange-400" />
          <h2 className="text-base font-bold text-txt-primary">东财用户标识</h2>
        </div>

        <div className="p-4 rounded-lg border border-[#30363D] bg-bg-card space-y-3">
          <div>
            <label className="text-xs text-txt-muted block mb-1">
              qgqp_b_id（智能选股必需）
            </label>
            <Input
              value={settings.qgqp_b_id || ''}
              onChange={e => saveSettings({ ...settings, qgqp_b_id: e.target.value.trim() })}
              placeholder="请粘贴东财 Cookie 中的 qgqp_b_id 值"
              style={inputStyle}
            />
          </div>
          <div className="text-[11px] text-txt-muted leading-relaxed">
            获取方式：打开浏览器访问{' '}
            <span className="text-blue-400">https://xuangu.eastmoney.com</span>
            {' '}→ F12 开发者工具 → 网络面板 → 随便点开一个请求 → 复制 Cookie 中 <code className="text-orange-300">qgqp_b_id</code> 的值
          </div>
        </div>
      </section>

      {/* AI Model Configuration */}
      <section>
        <div className="flex items-center gap-2 mb-4">
          <Bot size={18} className="text-primary-gold" />
          <h2 className="text-base font-bold text-txt-primary">AI 模型配置</h2>
        </div>

        <div className="grid gap-4">
          {settings.ai_configs.map(config => (
            <div
              key={config.id}
              className={`p-4 rounded-lg border transition-all ${
                config.id === settings.active_ai_config_id
                  ? 'border-primary-gold/50 bg-bg-elevated shadow-lg shadow-yellow-500/5'
                  : 'border-[#30363D] bg-bg-card hover:border-[#484F58]'
              }`}
            >
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                  <input
                    className="bg-transparent border-none text-txt-primary font-semibold text-sm outline-none w-32"
                    value={config.name}
                    onChange={e => updateAIConfig({ ...config, name: e.target.value })}
                  />
                  {config.id === settings.active_ai_config_id && (
                    <span className="px-2 py-0.5 rounded text-[10px] bg-primary-gold/20 text-primary-gold font-medium">
                      活跃
                    </span>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  {config.id !== settings.active_ai_config_id && (
                    <button
                      onClick={() => setActiveAIConfig(config.id)}
                      className="text-xs text-functional-info hover:underline cursor-pointer"
                    >
                      设为活跃
                    </button>
                  )}
                  <button
                    onClick={() => handleTestConfig(config)}
                    disabled={testingConfigId === config.id}
                    className="flex items-center gap-1 text-xs text-cyan-400 hover:text-cyan-300 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {testingConfigId === config.id ? (
                      <Loader2 size={12} className="animate-spin" />
                    ) : (
                      <Zap size={12} />
                    )}
                    {testingConfigId === config.id ? '测试中...' : '测试连接'}
                  </button>
                  <button
                    onClick={() => handleDeleteConfig(config.id)}
                    className="p-1 rounded hover:bg-red-900/30 transition-colors cursor-pointer"
                  >
                    <Trash2 size={14} className="text-red-400" />
                  </button>
                </div>
              </div>

              <div className="grid grid-cols-3 gap-3">
                <div>
                  <label className="text-xs text-txt-muted block mb-1">API 地址</label>
                  <Input
                    size="small"
                    value={config.base_url}
                    onChange={e => updateAIConfig({ ...config, base_url: e.target.value })}
                    style={inputStyle}
                  />
                </div>
                <div>
                  <label className="text-xs text-txt-muted block mb-1">API Key</label>
                  <Input.Password
                    size="small"
                    value={config.api_key}
                    onChange={e => updateAIConfig({ ...config, api_key: e.target.value })}
                    style={inputStyle}
                  />
                </div>
                <div>
                  <label className="text-xs text-txt-muted block mb-1">模型名称</label>
                  <Input
                    size="small"
                    value={config.model_name}
                    onChange={e => updateAIConfig({ ...config, model_name: e.target.value })}
                    placeholder="gpt-4o-mini / deepseek-chat"
                    style={inputStyle}
                  />
                </div>
                <div>
                  <label className="text-xs text-txt-muted block mb-1">
                    分析 Temperature: {config.temperature.toFixed(1)}
                  </label>
                  <Slider
                    min={0}
                    max={1}
                    step={0.1}
                    value={config.temperature}
                    onChange={v => updateAIConfig({ ...config, temperature: v })}
                  />
                </div>
                <div>
                  <label className="text-xs text-txt-muted block mb-1">
                    选股 Temperature: {(config.pick_temperature ?? 0.7).toFixed(1)}
                  </label>
                  <Slider
                    min={0}
                    max={1}
                    step={0.1}
                    value={config.pick_temperature ?? 0.7}
                    onChange={v => updateAIConfig({ ...config, pick_temperature: v })}
                  />
                </div>
                <div>
                  <label className="text-xs text-txt-muted block mb-1">单次最大 Tokens</label>
                  <InputNumber
                    size="small"
                    min={256}
                    max={128000}
                    step={256}
                    value={config.max_tokens}
                    onChange={v => updateAIConfig({ ...config, max_tokens: v || 2048 })}
                    style={{ ...inputStyle, width: '100%' }}
                  />
                </div>
                <div>
                  <label className="text-xs text-txt-muted block mb-1">超时时间(秒)</label>
                  <InputNumber
                    size="small"
                    min={30}
                    max={600}
                    step={30}
                    value={config.timeout_secs}
                    onChange={v => updateAIConfig({ ...config, timeout_secs: v || 300 })}
                    style={{ ...inputStyle, width: '100%' }}
                  />
                </div>
              </div>

              {/* Test Result */}
              {testResults[config.id] && testResults[config.id].msg && (
                <div className={`mt-3 px-3 py-2 rounded-md text-[11px] leading-relaxed flex items-start gap-2 ${
                  testResults[config.id].ok
                    ? 'bg-green-500/10 border border-green-500/20 text-green-400'
                    : 'bg-red-500/10 border border-red-500/20 text-red-400'
                }`}>
                  {testResults[config.id].ok ? (
                    <CheckCircle size={13} className="flex-shrink-0 mt-0.5" />
                  ) : (
                    <XCircle size={13} className="flex-shrink-0 mt-0.5" />
                  )}
                  <span className="break-all">{testResults[config.id].msg}</span>
                </div>
              )}
            </div>
          ))}

          <button
            onClick={handleAddModel}
            className="flex items-center justify-center gap-2 p-4 rounded-lg border border-dashed border-[#30363D] text-txt-secondary hover:border-[#484F58] hover:text-txt-primary transition-all cursor-pointer"
          >
            <Plus size={16} />
            <span className="text-sm">添加 AI 模型</span>
          </button>
        </div>
      </section>

      {/* General Configuration */}
      <section>
        <div className="flex items-center gap-2 mb-4">
          <Database size={18} className="text-functional-info" />
          <h2 className="text-base font-bold text-txt-primary">通用配置</h2>
        </div>

        <div className="p-4 rounded-lg border border-[#30363D] bg-bg-card space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <span className="text-sm text-txt-primary">自动刷新间隔</span>
              <span className="text-xs text-txt-muted ml-2">
                {settings.refresh_interval_secs}秒
              </span>
            </div>
            <div className="w-48">
              <Slider
                min={30}
                max={300}
                step={30}
                value={settings.refresh_interval_secs}
                onChange={v => saveSettings({ ...settings, refresh_interval_secs: v })}
              />
            </div>
          </div>

          <div className="flex items-center justify-between">
            <span className="text-sm text-txt-primary">AI 指令自动生成</span>
            <Switch
              checked={settings.ai_instruction_enabled}
              onChange={v => saveSettings({ ...settings, ai_instruction_enabled: v })}
            />
          </div>
        </div>
      </section>

      {/* Factor Weights */}
      {activeStrategy && (
        <section>
          <div className="flex items-center gap-2 mb-4">
            <Sliders size={18} className="text-primary-gold" />
            <h2 className="text-base font-bold text-txt-primary">因子权重配置</h2>
            <span className="text-xs text-txt-muted ml-2">
              总计: {activeStrategy.weights.value + activeStrategy.weights.quality + activeStrategy.weights.momentum + activeStrategy.weights.capital + activeStrategy.weights.risk + (activeStrategy.weights.sentiment || 0)}%
            </span>
          </div>

          <div className="p-4 rounded-lg border border-[#30363D] bg-bg-card space-y-3">
            {[
              { key: 'value', label: '价值因子（PE/PB综合）', color: 'text-green-400', desc: 'PE越低、PB越低得分越高' },
              { key: 'quality', label: '质量因子（ROE/营收增长）', color: 'text-blue-400', desc: 'ROE越高、营收增速越快得分越高' },
              { key: 'momentum', label: '动量因子（涨幅/量比）', color: 'text-orange-400', desc: '近期适度上涨趋势，但惩罚过热' },
              { key: 'capital', label: '资金因子（主力净流入/换手）', color: 'text-red-400', desc: '主力资金流入越多、换手率适中得分越高' },
              { key: 'risk', label: '风险因子（市值/波动率）', color: 'text-purple-400', desc: '市值适中、波动适中最优' },
              { key: 'sentiment', label: '消息因子（新闻/主题热度）', color: 'text-yellow-400', desc: '根据新闻热点匹配概念板块，捕捉消息面驱动的行情' },
            ].map(item => (
              <div key={item.key}>
                <div className="flex items-center justify-between mb-1">
                  <div>
                    <span className={`text-sm font-medium ${item.color}`}>{item.label}</span>
                    <span className="text-[10px] text-txt-muted ml-2">{item.desc}</span>
                  </div>
                  <span className="text-xs text-txt-muted w-10 text-right font-mono">
                    {(activeStrategy.weights as unknown as Record<string, number>)[item.key]}%
                  </span>
                </div>
                <Slider
                  min={0}
                  max={50}
                  step={5}
                  value={(activeStrategy.weights as unknown as Record<string, number>)[item.key]}
                  onChange={v => {
                    const newWeights = { ...activeStrategy.weights, [item.key]: v };
                    updateStrategy({ ...activeStrategy, weights: newWeights });
                  }}
                />
              </div>
            ))}

            <div className="flex items-center justify-between pt-2 border-t border-[#30363D]">
              <span className="text-sm text-txt-primary">输出 Top N</span>
              <InputNumber
                size="small"
                min={10}
                max={200}
                step={10}
                value={activeStrategy.top_n}
                onChange={v => updateStrategy({ ...activeStrategy, top_n: v || 50 })}
                style={{ ...inputStyle, width: 80 }}
              />
            </div>
          </div>
        </section>
      )}

      {/* Stock Filters */}
      {activeStrategy && (
        <section>
          <div className="flex items-center gap-2 mb-4">
            <Filter size={18} className="text-functional-info" />
            <h2 className="text-base font-bold text-txt-primary">筛选条件</h2>
          </div>

          <div className="p-4 rounded-lg border border-[#30363D] bg-bg-card grid grid-cols-2 gap-4">
            <div className="flex items-center justify-between">
              <span className="text-sm text-txt-primary">排除 ST</span>
              <Switch
                checked={activeStrategy.filters.exclude_st}
                onChange={v => updateStrategy({ ...activeStrategy, filters: { ...activeStrategy.filters, exclude_st: v } })}
              />
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-txt-primary">排除次新股(天)</span>
              <InputNumber
                size="small"
                min={0}
                max={365}
                value={activeStrategy.filters.exclude_new_stock_days}
                onChange={v => updateStrategy({ ...activeStrategy, filters: { ...activeStrategy.filters, exclude_new_stock_days: v || 60 } })}
                style={{ ...inputStyle, width: 80 }}
              />
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-txt-primary">最低市值(亿)</span>
              <InputNumber
                size="small"
                min={0}
                max={10000}
                value={activeStrategy.filters.min_market_cap}
                onChange={v => updateStrategy({ ...activeStrategy, filters: { ...activeStrategy.filters, min_market_cap: v || 0 } })}
                style={{ ...inputStyle, width: 80 }}
              />
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-txt-primary">最高市值(亿)</span>
              <InputNumber
                size="small"
                min={0}
                max={100000}
                value={activeStrategy.filters.max_market_cap}
                onChange={v => updateStrategy({ ...activeStrategy, filters: { ...activeStrategy.filters, max_market_cap: v || 0 } })}
                style={{ ...inputStyle, width: 80 }}
                placeholder="0=不限"
              />
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-txt-primary">最低股价(元)</span>
              <InputNumber
                size="small"
                min={0}
                max={1000}
                step={0.5}
                value={activeStrategy.filters.min_price}
                onChange={v => updateStrategy({ ...activeStrategy, filters: { ...activeStrategy.filters, min_price: v || 0 } })}
                style={{ ...inputStyle, width: 80 }}
              />
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-txt-primary">最低成交额(万)</span>
              <InputNumber
                size="small"
                min={0}
                max={1000000}
                step={1000}
                value={activeStrategy.filters.min_amount}
                onChange={v => updateStrategy({ ...activeStrategy, filters: { ...activeStrategy.filters, min_amount: v || 0 } })}
                style={{ ...inputStyle, width: 100 }}
              />
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-txt-primary">PE 上限</span>
              <InputNumber
                size="small"
                min={0}
                max={1000}
                value={activeStrategy.filters.pe_max}
                onChange={v => updateStrategy({ ...activeStrategy, filters: { ...activeStrategy.filters, pe_max: v || 0 } })}
                style={{ ...inputStyle, width: 80 }}
                placeholder="0=不限"
              />
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-txt-primary">PE 下限</span>
              <InputNumber
                size="small"
                min={-100}
                max={100}
                value={activeStrategy.filters.pe_min}
                onChange={v => updateStrategy({ ...activeStrategy, filters: { ...activeStrategy.filters, pe_min: v ?? 0 } })}
                style={{ ...inputStyle, width: 80 }}
              />
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-txt-primary">PB 上限</span>
              <InputNumber
                size="small"
                min={0}
                max={100}
                step={0.5}
                value={activeStrategy.filters.pb_max}
                onChange={v => updateStrategy({ ...activeStrategy, filters: { ...activeStrategy.filters, pb_max: v || 0 } })}
                style={{ ...inputStyle, width: 80 }}
                placeholder="0=不限"
              />
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-txt-primary">ROE 下限(%)</span>
              <InputNumber
                size="small"
                min={0}
                max={100}
                step={1}
                value={activeStrategy.filters.roe_min}
                onChange={v => updateStrategy({ ...activeStrategy, filters: { ...activeStrategy.filters, roe_min: v || 0 } })}
                style={{ ...inputStyle, width: 80 }}
              />
            </div>
          </div>
        </section>
      )}
    </div>
  );
}
