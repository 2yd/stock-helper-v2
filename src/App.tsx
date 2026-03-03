import { useState, useEffect } from 'react';
import { Modal } from 'antd';
import { open } from '@tauri-apps/plugin-shell';
import AIPick from './pages/AIPick';
import Settings from './pages/Settings';
import SmartStock from './pages/SmartStock';
import Watchlist from './pages/Watchlist';
import NewsCenter from './pages/NewsCenter';
import { safeInvoke as invoke } from './hooks/useTauri';
import logger from './utils/logger';
import { Settings as SettingsIcon, TrendingUp, ChevronLeft, Search, Brain, Eye, Newspaper } from 'lucide-react';

interface UpdateInfo {
  version: string;
  current_version: string;
  body: string;
  published_at: string;
  html_url: string;
}

type Page = 'board' | 'settings' | 'smart' | 'watchlist' | 'news';

export default function App() {
  const [currentPage, setCurrentPage] = useState<Page>('smart');

  // 启动时静默检查更新
  useEffect(() => {
    const timer = setTimeout(async () => {
      try {
        const result = await invoke<UpdateInfo | null>('check_update');
        if (result) {
          const publishDate = result.published_at
            ? new Date(result.published_at).toLocaleDateString('zh-CN')
            : '';

          Modal.confirm({
            title: `发现新版本 v${result.version}`,
            width: 520,
            icon: null,
            content: (
              <div className="mt-2 space-y-3">
                <div className="flex items-center gap-4 text-sm text-txt-muted">
                  <span>当前版本: v{result.current_version}</span>
                  {publishDate && <span>发布日期: {publishDate}</span>}
                </div>
                {result.body && (
                  <div className="max-h-60 overflow-y-auto p-3 rounded-lg bg-bg-elevated border border-[#30363D]">
                    <p className="text-xs text-txt-muted mb-1 font-medium">更新内容:</p>
                    <pre className="text-sm text-txt-secondary whitespace-pre-wrap font-sans leading-relaxed">{result.body}</pre>
                  </div>
                )}
                <p className="text-xs text-txt-muted">下载后直接安装即可，数据不会丢失。</p>
              </div>
            ),
            okText: '前往下载',
            cancelText: '稍后再说',
            onOk: () => {
              if (result.html_url) {
                open(result.html_url).catch((e: unknown) =>
                  logger.error(`Failed to open release URL: ${e}`)
                );
              }
            },
          });
        }
      } catch (e) {
        // 静默失败，不打扰用户
        logger.info(`Auto update check failed (non-critical): ${e}`);
      }
    }, 3000); // 延迟 3 秒检查，避免启动时阻塞

    return () => clearTimeout(timer);
  }, []);

  return (
    <div className="w-full h-full flex flex-col bg-bg-base">
      {/* Top Navigation Bar */}
      <header className="h-12 flex items-center px-4 border-b border-[#30363D] bg-bg-card flex-shrink-0">
        <div className="flex items-center gap-2">
          {currentPage === 'settings' && (
            <button
              onClick={() => setCurrentPage('smart')}
              className="p-1 rounded hover:bg-bg-elevated transition-colors cursor-pointer"
            >
              <ChevronLeft size={18} className="text-txt-secondary" />
            </button>
          )}
          <TrendingUp size={20} className="text-primary-red" />
          <span className="text-base font-bold text-txt-primary tracking-wide">
            Stock Helper
          </span>
          <span className="text-xs text-txt-muted ml-2">A股量化选股助手</span>
        </div>

        {/* Navigation Tabs */}
        {currentPage !== 'settings' && (
          <div className="flex items-center gap-1 ml-6">
            <button
              onClick={() => setCurrentPage('smart')}
              className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md transition-all cursor-pointer ${
                currentPage === 'smart'
                  ? 'bg-purple-600/20 text-purple-300 border border-purple-500/30'
                  : 'text-txt-secondary hover:text-txt-primary hover:bg-bg-elevated'
              }`}
            >
              <Search size={13} />
              智能选股
            </button>
            <button
              onClick={() => setCurrentPage('board')}
              className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md transition-all cursor-pointer ${
                currentPage === 'board'
                  ? 'bg-cyan-600/20 text-cyan-300 border border-cyan-500/30'
                  : 'text-txt-secondary hover:text-txt-primary hover:bg-bg-elevated'
              }`}
            >
              <Brain size={13} />
              AI选股
            </button>
            <button
              onClick={() => setCurrentPage('watchlist')}
              className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md transition-all cursor-pointer ${
                currentPage === 'watchlist'
                  ? 'bg-green-600/20 text-green-300 border border-green-500/30'
                  : 'text-txt-secondary hover:text-txt-primary hover:bg-bg-elevated'
              }`}
            >
              <Eye size={13} />
              盯盘
            </button>
            <button
              onClick={() => setCurrentPage('news')}
              className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md transition-all cursor-pointer ${
                currentPage === 'news'
                  ? 'bg-orange-600/20 text-orange-300 border border-orange-500/30'
                  : 'text-txt-secondary hover:text-txt-primary hover:bg-bg-elevated'
              }`}
            >
              <Newspaper size={13} />
              资讯
            </button>
          </div>
        )}

        <div className="flex-1" />
        {currentPage !== 'settings' && (
          <button
            onClick={() => setCurrentPage('settings')}
            className="p-2 rounded-lg hover:bg-bg-elevated transition-colors cursor-pointer"
          >
            <SettingsIcon size={18} className="text-txt-secondary hover:text-txt-primary transition-colors" />
          </button>
        )}
      </header>

      {/* Page Content */}
      <main className="flex-1 min-h-0 overflow-hidden">
        {currentPage === 'smart' ? (
          <SmartStock />
        ) : currentPage === 'board' ? (
          <AIPick />
        ) : currentPage === 'watchlist' ? (
          <Watchlist />
        ) : currentPage === 'news' ? (
          <NewsCenter />
        ) : (
          <Settings />
        )}
      </main>
    </div>
  );
}
