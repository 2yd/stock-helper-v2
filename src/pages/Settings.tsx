import { useEffect, useState } from 'react';
import { Slider, Switch, Select, Input, InputNumber, App } from 'antd';
import { Plus, Trash2, Bot, Database, Fingerprint, Loader2, CheckCircle, XCircle, Zap, FileDown, RefreshCw, Info } from 'lucide-react';
import { getVersion } from '@tauri-apps/api/app';
import { safeInvoke as invoke, isTauri } from '../hooks/useTauri';
import { useSettingsStore } from '../stores/settingsStore';
import { AIConfig } from '../types';
import UpdateModal from '../components/UpdateModal';
import type { UpdateInfo } from '../components/UpdateModal';

export default function Settings() {
  const { message } = App.useApp();
  const { settings, loadSettings, saveSettings, addAIConfig, removeAIConfig, updateAIConfig, setActiveAIConfig, testAIConfig, testingConfigId, exportLogs, exportingLogs } = useSettingsStore();

  const [testResults, setTestResults] = useState<Record<string, { ok: boolean; msg: string }>>({});
  const [appVersion, setAppVersion] = useState('');
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);

  useEffect(() => {
    loadSettings();
    // 获取当前版本号
    if (isTauri) {
      getVersion().then(v => setAppVersion(v)).catch(() => setAppVersion('unknown'));
    }
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

          <div className="flex items-center justify-between">
            <div>
              <span className="text-sm text-txt-primary">Agent 最大工具轮次</span>
              <span className="text-xs text-txt-muted ml-2">
                {settings.max_pick_tool_rounds ?? 10} 轮
              </span>
            </div>
            <div className="w-48">
              <Slider
                min={3}
                max={20}
                step={1}
                value={settings.max_pick_tool_rounds ?? 10}
                onChange={v => saveSettings({ ...settings, max_pick_tool_rounds: v })}
              />
            </div>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <span className="text-sm text-txt-primary">Agent Token 预算</span>
              <span className="text-xs text-txt-muted ml-2">
                {((settings.max_pick_token_budget ?? 100000) / 1000).toFixed(0)}K
              </span>
            </div>
            <div className="w-48">
              <Slider
                min={20000}
                max={500000}
                step={10000}
                value={settings.max_pick_token_budget ?? 100000}
                onChange={v => saveSettings({ ...settings, max_pick_token_budget: v })}
              />
            </div>
          </div>
        </div>
      </section>

      {/* 日志导出 */}
      <section>
        <div className="flex items-center gap-2 mb-4">
          <FileDown size={18} className="text-functional-info" />
          <h2 className="text-base font-bold text-txt-primary">日志管理</h2>
        </div>

        <div className="p-4 rounded-lg border border-[#30363D] bg-bg-card flex items-center justify-between">
          <div>
            <span className="text-sm text-txt-primary">导出日志文件</span>
            <p className="text-xs text-txt-muted mt-1">将应用日志打包为 ZIP 文件，便于发送给开发者诊断问题</p>
          </div>
          <button
            onClick={async () => {
              try {
                const result = await exportLogs();
                message.success(result);
              } catch (e: unknown) {
                const errMsg = e instanceof Error ? e.message : String(e);
                if (!errMsg.includes('取消')) {
                  message.error(`导出失败: ${errMsg}`);
                }
              }
            }}
            disabled={exportingLogs}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-functional-info/20 text-functional-info hover:bg-functional-info/30 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed text-sm"
          >
            {exportingLogs ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <FileDown size={14} />
            )}
            {exportingLogs ? '导出中...' : '导出日志'}
          </button>
        </div>
      </section>

      {/* 关于 / 版本更新 */}
      <section>
        <div className="flex items-center gap-2 mb-4">
          <Info size={18} className="text-purple-400" />
          <h2 className="text-base font-bold text-txt-primary">关于</h2>
        </div>

        <div className="p-4 rounded-lg border border-[#30363D] bg-bg-card space-y-3">
          <div className="flex items-center justify-between">
            <div>
              <span className="text-sm text-txt-primary">当前版本</span>
              <span className="text-sm text-txt-muted ml-2 font-mono">v{appVersion || '...'}</span>
            </div>
            <button
              onClick={async () => {
                setCheckingUpdate(true);
                try {
                  const result = await invoke<UpdateInfo | null>('check_update');
                  if (result) {
                    setUpdateInfo(result);
                  } else {
                    message.success('已是最新版本');
                  }
                } catch (e: unknown) {
                  const errMsg = e instanceof Error ? e.message : String(e);
                  message.error(`检查更新失败: ${errMsg}`);
                } finally {
                  setCheckingUpdate(false);
                }
              }}
              disabled={checkingUpdate}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-purple-500/20 text-purple-300 hover:bg-purple-500/30 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed text-sm"
            >
              {checkingUpdate ? (
                <Loader2 size={14} className="animate-spin" />
              ) : (
                <RefreshCw size={14} />
              )}
              {checkingUpdate ? '检查中...' : '检查更新'}
            </button>
          </div>
        </div>
      </section>

      {/* 更新弹窗 */}
      {updateInfo && (
        <UpdateModal info={updateInfo} onClose={() => setUpdateInfo(null)} />
      )}
    </div>
  );
}
