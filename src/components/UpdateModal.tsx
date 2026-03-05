import { useState } from 'react';
import { open } from '@tauri-apps/plugin-shell';
import { Sparkles, ArrowUpCircle, Calendar, Tag, ExternalLink, X } from 'lucide-react';
import logger from '../utils/logger';

export interface UpdateInfo {
  version: string;
  current_version: string;
  body: string;
  published_at: string;
  html_url: string;
}

interface UpdateModalProps {
  info: UpdateInfo;
  onClose: () => void;
}

export default function UpdateModal({ info, onClose }: UpdateModalProps) {
  const [hovering, setHovering] = useState(false);

  const publishDate = info.published_at
    ? new Date(info.published_at).toLocaleDateString('zh-CN', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
      })
    : '';

  const handleDownload = () => {
    if (info.html_url) {
      open(info.html_url).catch((e: unknown) =>
        logger.error(`Failed to open release URL: ${e}`)
      );
    }
    onClose();
  };

  // 简单解析 release notes 中的 markdown 列表
  const renderBody = (body: string) => {
    const lines = body.split('\n').filter(l => l.trim());
    return lines.map((line, i) => {
      const trimmed = line.trim();
      // heading
      if (trimmed.startsWith('### ')) {
        return (
          <p key={i} className="text-xs font-semibold text-purple-300 mt-3 mb-1 first:mt-0">
            {trimmed.replace(/^###\s*/, '')}
          </p>
        );
      }
      if (trimmed.startsWith('## ')) {
        return (
          <p key={i} className="text-sm font-semibold text-txt-primary mt-3 mb-1 first:mt-0">
            {trimmed.replace(/^##\s*/, '')}
          </p>
        );
      }
      // list item
      if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
        return (
          <div key={i} className="flex items-start gap-2 py-0.5">
            <span className="w-1.5 h-1.5 rounded-full bg-purple-400/60 mt-1.5 flex-shrink-0" />
            <span className="text-sm text-txt-secondary leading-relaxed">
              {trimmed.replace(/^[-*]\s*/, '')}
            </span>
          </div>
        );
      }
      return (
        <p key={i} className="text-sm text-txt-secondary leading-relaxed">
          {trimmed}
        </p>
      );
    });
  };

  return (
    <div className="fixed inset-0 z-[9999] flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="relative w-[460px] max-h-[80vh] flex flex-col rounded-2xl border border-[#30363D] bg-gradient-to-b from-[#1C2333] to-[#161B22] shadow-2xl shadow-purple-500/10 overflow-hidden animate-in">
        {/* Glow effect at top */}
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-48 h-24 bg-purple-500/15 blur-3xl pointer-events-none" />

        {/* Close button */}
        <button
          onClick={onClose}
          className="absolute top-4 right-4 p-1 rounded-lg text-txt-muted hover:text-txt-primary hover:bg-white/5 transition-colors cursor-pointer z-10"
        >
          <X size={16} />
        </button>

        {/* Header */}
        <div className="relative px-6 pt-6 pb-4">
          <div className="flex items-center gap-3 mb-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-purple-500 to-violet-600 flex items-center justify-center shadow-lg shadow-purple-500/20">
              <Sparkles size={20} className="text-white" />
            </div>
            <div>
              <h2 className="text-lg font-bold text-txt-primary">发现新版本</h2>
              <p className="text-xs text-txt-muted">有新的更新可用</p>
            </div>
          </div>

          {/* Version badge */}
          <div className="flex items-center gap-3 mt-4">
            <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[#0D1117] border border-[#30363D]">
              <Tag size={12} className="text-txt-muted" />
              <span className="text-xs text-txt-muted font-mono">v{info.current_version}</span>
            </div>
            <div className="flex items-center gap-1">
              <div className="w-2 h-px bg-[#30363D]" />
              <ArrowUpCircle size={16} className="text-purple-400" />
              <div className="w-2 h-px bg-[#30363D]" />
            </div>
            <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-purple-500/10 border border-purple-500/30">
              <Tag size={12} className="text-purple-400" />
              <span className="text-xs text-purple-300 font-mono font-semibold">v{info.version}</span>
            </div>
            {publishDate && (
              <div className="flex items-center gap-1.5 ml-auto text-xs text-txt-muted">
                <Calendar size={12} />
                {publishDate}
              </div>
            )}
          </div>
        </div>

        {/* Release notes */}
        {info.body && (
          <div className="px-6 pb-2 flex-1 min-h-0">
            <div className="max-h-52 overflow-y-auto p-4 rounded-xl bg-[#0D1117]/80 border border-[#30363D]/60">
              {renderBody(info.body)}
            </div>
          </div>
        )}

        {/* Footer */}
        <div className="px-6 py-5">
          <p className="text-[11px] text-txt-muted mb-4 text-center">
            下载后直接安装即可覆盖旧版本，数据不会丢失
          </p>
          <div className="flex items-center gap-3">
            <button
              onClick={onClose}
              className="flex-1 py-2.5 rounded-xl text-sm font-medium text-txt-secondary border border-[#30363D] hover:bg-white/5 hover:border-[#484F58] transition-all cursor-pointer"
            >
              稍后再说
            </button>
            <button
              onClick={handleDownload}
              onMouseEnter={() => setHovering(true)}
              onMouseLeave={() => setHovering(false)}
              className="flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl text-sm font-semibold text-white bg-gradient-to-r from-purple-600 to-violet-600 hover:from-purple-500 hover:to-violet-500 shadow-lg shadow-purple-500/25 hover:shadow-purple-500/40 transition-all cursor-pointer"
            >
              <ExternalLink size={14} className={hovering ? 'translate-x-0.5 -translate-y-0.5 transition-transform' : 'transition-transform'} />
              前往下载
            </button>
          </div>
        </div>
      </div>

      <style>{`
        @keyframes animate-in {
          from {
            opacity: 0;
            transform: scale(0.95) translateY(10px);
          }
          to {
            opacity: 1;
            transform: scale(1) translateY(0);
          }
        }
        .animate-in {
          animation: animate-in 0.25s ease-out;
        }
      `}</style>
    </div>
  );
}
