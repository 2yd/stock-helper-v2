import { useState, useEffect } from 'react';
import { Modal, Input, message } from 'antd';
import { Plus, Copy, Trash2, LayoutGrid, Info, Shield, Target, Compass, TrendingUp } from 'lucide-react';
import { useAgentPromptStore, STRATEGY_TEMPLATES, StrategyTemplate } from '../stores/agentPromptStore';
import { AgentPrompt } from '../types';

const BUILTIN_DEFAULT_PROMPT_ID = 'builtin_default';

type RightView = 'default_summary' | 'edit' | 'template_select';

export default function AgentPromptModal() {
  const {
    prompts, modalVisible, editingPrompt,
    closeModal, setEditingPrompt, addPrompt, updatePrompt, deletePrompt, duplicatePrompt, createFromTemplate,
  } = useAgentPromptStore();

  const [editName, setEditName] = useState('');
  const [editDescription, setEditDescription] = useState('');
  const [editContent, setEditContent] = useState('');
  const [hasChanges, setHasChanges] = useState(false);
  const [rightView, setRightView] = useState<RightView>('default_summary');

  useEffect(() => {
    if (editingPrompt) {
      setEditName(editingPrompt.name);
      setEditDescription(editingPrompt.description || '');
      setEditContent(editingPrompt.strategy_prompt);
      setHasChanges(false);
      setRightView(editingPrompt.id === BUILTIN_DEFAULT_PROMPT_ID ? 'default_summary' : 'edit');
    }
  }, [editingPrompt]);

  const isBuiltin = editingPrompt?.id === BUILTIN_DEFAULT_PROMPT_ID;

  const handleSelectPrompt = (prompt: AgentPrompt) => {
    if (rightView === 'template_select') {
      setRightView(prompt.id === BUILTIN_DEFAULT_PROMPT_ID ? 'default_summary' : 'edit');
    }
    if (hasChanges && !isBuiltin && rightView === 'edit') {
      Modal.confirm({
        title: '未保存的更改',
        content: '当前编辑内容尚未保存，切换将丢失更改。是否继续？',
        okText: '继续',
        cancelText: '取消',
        onOk: () => {
          setEditingPrompt(prompt);
          setHasChanges(false);
        },
      });
      return;
    }
    setEditingPrompt(prompt);
  };

  const handleSave = async () => {
    if (!editingPrompt || isBuiltin) return;
    if (!editName.trim()) {
      message.error('策略名称不能为空');
      return;
    }
    if (!editContent.trim()) {
      message.error('策略内容不能为空');
      return;
    }
    await updatePrompt(editingPrompt.id, editName.trim(), editContent, editDescription.trim() || undefined);
    setHasChanges(false);
    message.success('保存成功');
  };

  const handleDelete = async () => {
    if (!editingPrompt || isBuiltin) return;
    Modal.confirm({
      title: '确认删除',
      content: `确定要删除策略「${editingPrompt.name}」吗？此操作不可恢复。`,
      okText: '删除',
      okButtonProps: { danger: true },
      cancelText: '取消',
      onOk: async () => {
        await deletePrompt(editingPrompt.id);
        message.success('已删除');
      },
    });
  };

  const handleAddNew = async () => {
    setRightView('template_select');
  };

  const handleDuplicate = async () => {
    if (!editingPrompt) return;
    await duplicatePrompt(editingPrompt);
    message.success('已复制');
  };

  const handleUseTemplate = async (template: StrategyTemplate) => {
    await createFromTemplate(template);
    setRightView('edit');
    message.success(`已创建「${template.name}」策略`);
  };

  const handleCreateBlank = async () => {
    await addPrompt('新策略', '', '');
    setRightView('edit');
  };

  return (
    <Modal
      open={modalVisible}
      onCancel={closeModal}
      footer={null}
      width={780}
      title={null}
      closable={false}
      centered
      className="agent-prompt-modal"
      styles={{
        body: { padding: 0, backgroundColor: '#161B22', borderRadius: 12, border: '1px solid #30363D', overflow: 'hidden' },
        mask: { backgroundColor: 'rgba(0,0,0,0.6)' },
      }}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-3.5 border-b border-[#30363D]">
        <h3 className="text-[15px] font-semibold text-[#E6EDF3]">选股策略</h3>
        <button onClick={closeModal} className="text-[#484F58] hover:text-[#E6EDF3] transition-colors cursor-pointer text-lg leading-none">✕</button>
      </div>

      {/* Body — two columns */}
      <div className="flex" style={{ height: 540 }}>
        {/* Left: prompt list */}
        <div className="w-[200px] border-r border-[#30363D] bg-[#0D1117] flex flex-col">
          <div className="flex-1 overflow-y-auto py-1">
            {prompts.map((p) => (
              <button
                key={p.id}
                onClick={() => handleSelectPrompt(p)}
                className={`w-full text-left px-3 py-2.5 text-[12px] flex items-center gap-2 transition-all cursor-pointer border-l-[3px] ${
                  editingPrompt?.id === p.id && rightView !== 'template_select'
                    ? 'border-l-[#22D3EE] bg-[#1C2333] text-[#E6EDF3]'
                    : 'border-l-transparent text-[#8B949E] hover:bg-[#161B22] hover:text-[#E6EDF3]'
                }`}
              >
                <span className="truncate flex-1">{p.name}</span>
                {p.is_builtin && (
                  <span className="shrink-0 text-[10px] px-1.5 py-0.5 rounded bg-[#1C2333] text-[#8B949E] border border-[#30363D]">
                    默认
                  </span>
                )}
              </button>
            ))}
          </div>

          {/* Bottom actions */}
          <div className="border-t border-[#30363D] px-3 py-2.5 space-y-1.5">
            <button
              onClick={handleAddNew}
              className={`flex items-center gap-1 text-[11px] transition-colors cursor-pointer w-full ${
                rightView === 'template_select'
                  ? 'text-[#22D3EE]'
                  : 'text-[#8B949E] hover:text-[#22D3EE]'
              }`}
            >
              <LayoutGrid size={12} />
              从模板创建
            </button>
            <div className="flex gap-3">
              <button
                onClick={handleCreateBlank}
                className="flex items-center gap-1 text-[11px] text-[#8B949E] hover:text-[#22D3EE] transition-colors cursor-pointer"
              >
                <Plus size={12} />
                空白新建
              </button>
              <button
                onClick={handleDuplicate}
                className="flex items-center gap-1 text-[11px] text-[#8B949E] hover:text-[#22D3EE] transition-colors cursor-pointer"
              >
                <Copy size={11} />
                复制当前
              </button>
            </div>
          </div>
        </div>

        {/* Right: content area */}
        <div className="flex-1 flex flex-col bg-[#161B22]">
          {/* View A: Default Strategy Summary */}
          {rightView === 'default_summary' && (
            <DefaultStrategySummary onDuplicate={handleDuplicate} />
          )}

          {/* View B: Strategy Edit */}
          {rightView === 'edit' && editingPrompt && !isBuiltin && (
            <>
              {/* Name + Description inputs */}
              <div className="px-4 pt-3 pb-1 space-y-2">
                <Input
                  value={editName}
                  onChange={(e) => { setEditName(e.target.value); setHasChanges(true); }}
                  placeholder="策略名称"
                  className="!bg-[#0D1117] !border-[#30363D] !text-[#E6EDF3] !text-[12px] hover:!border-[#484F58] focus:!border-[#22D3EE]"
                  styles={{ input: { backgroundColor: '#0D1117', color: '#E6EDF3' } }}
                />
                <Input
                  value={editDescription}
                  onChange={(e) => { setEditDescription(e.target.value); setHasChanges(true); }}
                  placeholder="一句话描述（可选，用于下拉列表中展示）"
                  className="!bg-[#0D1117] !border-[#30363D] !text-[#8B949E] !text-[11px] hover:!border-[#484F58] focus:!border-[#22D3EE]"
                  styles={{ input: { backgroundColor: '#0D1117', color: '#8B949E' } }}
                />
              </div>

              {/* Strategy textarea */}
              <div className="flex-1 px-4 pb-3 flex flex-col min-h-0">
                <div className="relative flex-1 flex flex-col">
                  <Input.TextArea
                    value={editContent}
                    onChange={(e) => { setEditContent(e.target.value); setHasChanges(true); }}
                    placeholder="在此编写你的选股策略..."
                    className="!flex-1 !resize-none !bg-[#0D1117] !border-[#30363D] !text-[#E6EDF3] !text-[12px] !leading-relaxed hover:!border-[#484F58] focus:!border-[#22D3EE]"
                    style={{ fontFamily: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace', minHeight: 0 }}
                    styles={{ textarea: { backgroundColor: '#0D1117', color: '#E6EDF3' } }}
                  />
                  <span className="absolute bottom-2 right-3 text-[10px] text-[#484F58]">
                    {editContent.length} 字符
                  </span>
                </div>
              </div>

              {/* Footer actions */}
              <div className="flex items-center justify-between px-4 py-3 border-t border-[#30363D]">
                <div>
                  <button
                    onClick={handleDelete}
                    className="flex items-center gap-1.5 text-[12px] text-[#F85149] hover:text-[#FF7B72] transition-colors cursor-pointer"
                  >
                    <Trash2 size={13} />
                    删除
                  </button>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={closeModal}
                    className="px-4 py-1.5 text-[12px] text-[#8B949E] hover:text-[#E6EDF3] rounded-md border border-[#30363D] hover:border-[#484F58] transition-all cursor-pointer"
                  >
                    取消
                  </button>
                  <button
                    onClick={handleSave}
                    disabled={!hasChanges}
                    className={`px-4 py-1.5 text-[12px] rounded-md font-medium transition-all cursor-pointer ${
                      hasChanges
                        ? 'bg-[#06B6D4] text-white hover:bg-[#22D3EE] active:bg-[#0891B2]'
                        : 'bg-[#06B6D4]/30 text-[#06B6D4]/50 cursor-not-allowed'
                    }`}
                  >
                    保存
                  </button>
                </div>
              </div>
            </>
          )}

          {/* View C: Template Selection */}
          {rightView === 'template_select' && (
            <TemplateSelectView onSelect={handleUseTemplate} />
          )}

          {/* Fallback */}
          {rightView !== 'default_summary' && rightView !== 'edit' && rightView !== 'template_select' && (
            <div className="flex-1 flex items-center justify-center text-[#484F58] text-[13px]">
              请选择或新建一个策略
            </div>
          )}
        </div>
      </div>
    </Modal>
  );
}

/** 视图 A：默认策略摘要卡片 */
function DefaultStrategySummary({ onDuplicate }: { onDuplicate: () => void }) {
  return (
    <div className="flex-1 flex flex-col p-5 overflow-y-auto">
      <div className="mb-4">
        <div className="flex items-center gap-2 mb-1">
          <Info size={14} className="text-[#06B6D4]" />
          <h4 className="text-[14px] font-semibold text-[#E6EDF3]">默认策略说明</h4>
        </div>
        <p className="text-[11px] text-[#8B949E] leading-relaxed">
          默认策略由系统内置，采用多维度均衡分析方法。以下是策略核心要点：
        </p>
      </div>

      <div className="space-y-3 flex-1">
        {/* Role */}
        <SummaryCard
          icon={<Compass size={13} className="text-[#06B6D4]" />}
          title="角色定位"
          content="20年经验的独立投研分析师（A股方向），具备自主决策能力"
        />

        {/* Goal */}
        <SummaryCard
          icon={<Target size={13} className="text-[#F59E0B]" />}
          title="核心目标"
          content="自主分析市场环境，推荐 3-8 只值得关注的 A 股股票"
        />

        {/* Strategy */}
        <SummaryCard
          icon={<TrendingUp size={13} className="text-[#10B981]" />}
          title="选股风格"
          content="趋势跟踪 · 事件驱动 · 价值洼地 · 技术突破 · 逆向投资 — 根据市场状态灵活组合"
        />

        {/* Risk */}
        <SummaryCard
          icon={<Shield size={13} className="text-[#EF4444]" />}
          title="风险控制"
          items={[
            '排除涨停/连板股票，不追高',
            '优先选择涨幅 -2%~5% 的个股',
            '系统性下跌时减少推荐、降低评级',
            '关注资金流向，警惕主力持续流出',
            '敏感时间节点（两会/财报季/长假前）适度保守',
          ]}
        />
      </div>

      {/* Bottom action */}
      <div className="mt-4 pt-3 border-t border-[#30363D]/60">
        <p className="text-[11px] text-[#8B949E] mb-2.5">
          默认策略适合大多数场景。如需定制，可基于此策略创建副本进行修改。
        </p>
        <button
          onClick={onDuplicate}
          className="flex items-center gap-1.5 px-4 py-2 rounded-md text-[12px] font-medium bg-[#06B6D4]/10 text-[#06B6D4] border border-[#06B6D4]/20 hover:bg-[#06B6D4]/20 transition-all cursor-pointer"
        >
          <Copy size={12} />
          基于此策略创建副本
        </button>
      </div>
    </div>
  );
}

/** 摘要卡片组件 */
function SummaryCard({ icon, title, content, items }: { icon: React.ReactNode; title: string; content?: string; items?: string[] }) {
  return (
    <div className="rounded-lg border border-[#30363D]/60 bg-[#0D1117]/50 px-3.5 py-2.5">
      <div className="flex items-center gap-2 mb-1.5">
        {icon}
        <span className="text-[12px] font-medium text-[#E6EDF3]">{title}</span>
      </div>
      {content && (
        <p className="text-[11px] text-[#8B949E] leading-relaxed ml-5">{content}</p>
      )}
      {items && (
        <ul className="text-[11px] text-[#8B949E] leading-relaxed ml-5 space-y-0.5">
          {items.map((item, i) => (
            <li key={i} className="flex items-start gap-1.5">
              <span className="text-[#484F58] mt-[3px] text-[8px]">●</span>
              <span>{item}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** 视图 C：模板选择网格 */
function TemplateSelectView({ onSelect }: { onSelect: (t: StrategyTemplate) => void }) {
  return (
    <div className="flex-1 flex flex-col p-4 overflow-y-auto">
      <div className="mb-3">
        <h4 className="text-[14px] font-semibold text-[#E6EDF3] mb-1">选择策略模板</h4>
        <p className="text-[11px] text-[#8B949E]">每个模板都是完整可用的策略，选择后可自由修改</p>
      </div>

      <div className="grid grid-cols-2 gap-2.5 flex-1 content-start">
        {STRATEGY_TEMPLATES.map((tpl) => (
          <button
            key={tpl.id}
            onClick={() => onSelect(tpl)}
            className="text-left rounded-lg border border-[#30363D]/60 bg-[#0D1117]/50 px-3.5 py-3 hover:border-[#06B6D4]/40 hover:bg-[#06B6D4]/5 transition-all cursor-pointer group"
          >
            <div className="flex items-center gap-2 mb-1.5">
              <span className="text-base">{tpl.icon}</span>
              <span className="text-[13px] font-medium text-[#E6EDF3] group-hover:text-[#22D3EE] transition-colors">{tpl.name}</span>
            </div>
            <p className="text-[11px] text-[#8B949E] leading-relaxed">{tpl.description}</p>
          </button>
        ))}
      </div>
    </div>
  );
}
