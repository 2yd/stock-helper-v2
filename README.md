# Stock Helper V2

A股量化选股助手 —— AI 赋能竞价选股系统

基于 Tauri 2 构建的桌面应用，集成大语言模型 AI Agent，通过 Tool Calling 自主调用股票分析工具进行智能选股、多因子量化评分和技术分析。

## 功能概览

### 智能选股
- 自然语言搜索选股（如"低估值高股息的银行股"）
- 热门策略推荐（龙头战法、底部反转、趋势突破等）
- AI 智能筛选，自动匹配最佳策略

### AI 自主选股
- AI Agent 自主调用工具链进行多轮分析选股
- 流式对话交互，实时展示推理过程和工具调用
- 相似股发现：从同板块中挖掘低位补涨机会
- 追踪管理与亏损原因分析

### 盯盘 / 自选股
- 自选股管理与实时行情监控
- K 线图可视化（KLineCharts）
- 技术指标分析（MA / MACD / KDJ / RSI / BOLL）
- AI 个股诊断：一键分析持仓股的技术面和基本面

### 资讯中心
- 8 大数据源聚合：财联社电报、东方财富、新浪滚动、新浪 7×24、华尔街见闻、个股新闻、公告、研报

### 设置
- AI 模型配置（支持自定义 base_url / api_key / model，兼容 OpenAI 格式）
- 策略参数配置
- 数据源设置

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2 |
| 前端 | React 18 + TypeScript + Vite 5 |
| UI | Ant Design 6（暗色主题）+ TailwindCSS 3 |
| 状态管理 | Zustand 5 |
| 图表 | KLineCharts 9 + Lightweight Charts 4 + Recharts 3 |
| 后端 | Rust 2021 Edition |
| 数据库 | SQLite（rusqlite，bundled） |
| HTTP | reqwest 0.12（JSON / gzip / stream） |
| 异步 | Tokio（full features） |

## 架构

```
┌─────────────────────────────────────────────────┐
│               Tauri 桌面应用                      │
│                                                   │
│  ┌──────────── 前端 (Web) ─────────────┐         │
│  │  React + TypeScript + Vite           │         │
│  │  Ant Design + TailwindCSS            │         │
│  │  Zustand + KLineCharts               │         │
│  │                                      │         │
│  │  智能选股 │ AI选股 │ 盯盘 │ 资讯     │         │
│  └──────────────┬───────────────────────┘         │
│                 │ tauri invoke                     │
│  ┌──────────────▼───────────────────────┐         │
│  │  Rust 后端                            │         │
│  │  Commands → Services → Models         │         │
│  │       │          │                    │         │
│  │   ┌───▼───┐  ┌───▼────┐  ┌────────┐  │         │
│  │   │SQLite │  │HTTP/API│  │AI Agent│  │         │
│  │   │持久化  │  │行情/新闻│  │Tool Call│  │         │
│  │   └───────┘  └────────┘  └────────┘  │         │
│  └──────────────────────────────────────┘         │
└─────────────────────────────────────────────────┘
```

## 核心模块

### 后端服务（`src-tauri/src/services/`）

| 模块 | 说明 |
|------|------|
| `ai_service` | AI 对话、流式响应、Tool Calling、多轮推理选股 |
| `stock_tools` | AI Agent 工具函数集（供 Function Calling 调用） |
| `scoring` | 六维多因子评分（价值/质量/动量/资金/风险/情绪） |
| `technical_indicators` | 技术指标计算（MA/MACD/KDJ/RSI/BOLL） |
| `market_scanner` | 全市场扫描筛选 |
| `stock_data` | 行情数据获取（新浪/腾讯/东财） |
| `news_service` | 8 源新闻聚合 |
| `smart_stock` | 智能选股逻辑 |
| `market_pool` | 资金池分析（涨停池/连板池/高位池） |

### 前端页面（`src/pages/`）

| 页面 | 说明 |
|------|------|
| `SmartStock` | 智能选股（默认首页） |
| `AIPick` | AI 自主选股 |
| `Watchlist` | 盯盘 / 自选股 |
| `NewsCenter` | 资讯中心 |
| `Settings` | 设置 |

## 开发

### 前置要求

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) >= 1.77
- [Tauri 2 Prerequisites](https://v2.tauri.app/start/prerequisites/)

### 安装依赖

```bash
npm install
```

### 开发运行

```bash
npm run tauri dev
```

### 构建

```bash
npm run tauri build
```

## 项目结构

```
stock-helper-v2/
├── src/                    # 前端源码
│   ├── pages/              # 页面组件
│   ├── components/         # 通用组件
│   ├── stores/             # Zustand 状态管理
│   ├── hooks/              # 自定义 Hooks
│   ├── types/              # TypeScript 类型定义
│   └── styles/             # 全局样式
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── commands/       # Tauri 命令（前后端桥接）
│       ├── services/       # 业务逻辑
│       ├── models/         # 数据模型
│       ├── db/             # SQLite 数据库
│       └── utils/          # 工具函数
├── index.html
├── package.json
├── vite.config.ts
└── tailwind.config.js
```
