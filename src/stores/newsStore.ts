import { create } from 'zustand';
import { safeInvoke as invoke } from '../hooks/useTauri';
import { NewsItem, AnnouncementItem, ReportItem } from '../types';

export type NewsTab = 'telegraph' | 'news' | 'announcement' | 'report';

interface NewsStore {
  activeTab: NewsTab;
  // 财联社快讯
  telegraphs: NewsItem[];
  telegraphLoading: boolean;
  // 财经要闻 (东方财富 + 新浪)
  newsList: NewsItem[];
  newsLoading: boolean;
  newsPage: number;
  // 公告
  announcements: AnnouncementItem[];
  announcementLoading: boolean;
  announcementPage: number;
  // 研报
  reports: ReportItem[];
  reportLoading: boolean;
  reportPage: number;
  // 个股新闻搜索
  stockKeyword: string;
  stockNews: NewsItem[];
  stockNewsLoading: boolean;
  // 自动刷新
  autoRefreshTimer: ReturnType<typeof setInterval> | null;

  setActiveTab: (tab: NewsTab) => void;
  setStockKeyword: (kw: string) => void;

  fetchTelegraphs: () => Promise<void>;
  fetchNews: (loadMore?: boolean) => Promise<void>;
  fetchAnnouncements: (stockCode?: string, loadMore?: boolean) => Promise<void>;
  fetchReports: (stockCode?: string, loadMore?: boolean) => Promise<void>;
  fetchStockNews: (keyword: string) => Promise<void>;

  startAutoRefresh: (intervalSecs: number) => void;
  stopAutoRefresh: () => void;
}

export const useNewsStore = create<NewsStore>((set, get) => ({
  activeTab: 'telegraph',
  telegraphs: [],
  telegraphLoading: false,
  newsList: [],
  newsLoading: false,
  newsPage: 1,
  announcements: [],
  announcementLoading: false,
  announcementPage: 1,
  reports: [],
  reportLoading: false,
  reportPage: 1,
  stockKeyword: '',
  stockNews: [],
  stockNewsLoading: false,
  autoRefreshTimer: null,

  setActiveTab: (tab) => set({ activeTab: tab }),
  setStockKeyword: (kw) => set({ stockKeyword: kw }),

  fetchTelegraphs: async () => {
    set({ telegraphLoading: true });
    try {
      const items = await invoke<NewsItem[]>('fetch_cls_telegraph', { count: 50 });
      set({ telegraphs: items, telegraphLoading: false });
    } catch (e) {
      console.error('获取财联社快讯失败:', e);
      set({ telegraphLoading: false });
    }
  },

  fetchNews: async (loadMore = false) => {
    const { newsPage, newsList } = get();
    const page = loadMore ? newsPage + 1 : 1;
    set({ newsLoading: true });
    try {
      const [emNews, sinaNews] = await Promise.all([
        invoke<NewsItem[]>('fetch_eastmoney_news', { page, pageSize: 20 }),
        invoke<NewsItem[]>('fetch_sina_news', { page, count: 20 }),
      ]);

      // 合并并按时间排序
      const combined = [...emNews, ...sinaNews].sort(
        (a, b) => b.publish_time.localeCompare(a.publish_time)
      );

      set({
        newsList: loadMore ? [...newsList, ...combined] : combined,
        newsPage: page,
        newsLoading: false,
      });
    } catch (e) {
      console.error('获取新闻失败:', e);
      set({ newsLoading: false });
    }
  },

  fetchAnnouncements: async (stockCode, loadMore = false) => {
    const { announcementPage, announcements } = get();
    const page = loadMore ? announcementPage + 1 : 1;
    set({ announcementLoading: true });
    try {
      const items = await invoke<AnnouncementItem[]>('fetch_announcements', {
        stockCode: stockCode || null,
        page,
        pageSize: 30,
      });
      set({
        announcements: loadMore ? [...announcements, ...items] : items,
        announcementPage: page,
        announcementLoading: false,
      });
    } catch (e) {
      console.error('获取公告失败:', e);
      set({ announcementLoading: false });
    }
  },

  fetchReports: async (stockCode, loadMore = false) => {
    const { reportPage, reports } = get();
    const page = loadMore ? reportPage + 1 : 1;
    set({ reportLoading: true });
    try {
      const items = await invoke<ReportItem[]>('fetch_reports', {
        stockCode: stockCode || null,
        page,
        pageSize: 30,
      });
      set({
        reports: loadMore ? [...reports, ...items] : items,
        reportPage: page,
        reportLoading: false,
      });
    } catch (e) {
      console.error('获取研报失败:', e);
      set({ reportLoading: false });
    }
  },

  fetchStockNews: async (keyword: string) => {
    if (!keyword.trim()) {
      set({ stockNews: [], stockKeyword: '' });
      return;
    }
    set({ stockNewsLoading: true, stockKeyword: keyword });
    try {
      const items = await invoke<NewsItem[]>('fetch_stock_news', {
        keyword,
        page: 1,
        pageSize: 20,
      });
      set({ stockNews: items, stockNewsLoading: false });
    } catch (e) {
      console.error('获取个股新闻失败:', e);
      set({ stockNewsLoading: false });
    }
  },

  startAutoRefresh: (intervalSecs: number) => {
    const { autoRefreshTimer } = get();
    if (autoRefreshTimer) clearInterval(autoRefreshTimer);

    const timer = setInterval(() => {
      const { activeTab } = get();
      if (activeTab === 'telegraph') {
        get().fetchTelegraphs();
      }
    }, intervalSecs * 1000);

    set({ autoRefreshTimer: timer });
  },

  stopAutoRefresh: () => {
    const { autoRefreshTimer } = get();
    if (autoRefreshTimer) {
      clearInterval(autoRefreshTimer);
      set({ autoRefreshTimer: null });
    }
  },
}));
