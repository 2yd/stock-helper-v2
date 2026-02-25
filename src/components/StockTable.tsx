import { useMemo, useCallback } from 'react';
import { Table, Tooltip } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { Trash2 } from 'lucide-react';
import { StrategyResultRow, StockLabel, AIInstruction } from '../types';
import ScoreBadge from './ScoreBadge';
import TagLabel from './TagLabel';
import InstructionTag from './InstructionTag';

/* ------------------------------------------------------------------ */
/*  Unified row type — both watchlist and strategy feed into this      */
/* ------------------------------------------------------------------ */

/** 统一行数据：单位约定
 *  - total_market_cap / float_market_cap: 亿元
 *  - main_net_inflow: 万元
 *  - amount: 万元
 *  - turnover_rate / change_pct / pct_*: %
 */
export interface UnifiedStockRow {
  code: string;
  name: string;
  price: number;
  change_pct: number;
  pe_ttm: number;
  pb: number;
  roe: number;
  revenue_yoy: number;
  total_market_cap: number;   // 亿
  turnover_rate: number;      // %
  volume_ratio: number;
  main_net_inflow: number;    // 万
  amount: number;             // 万
  pct_5d: number;
  pct_20d: number;
  // 以下字段策略模式专用，盯盘模式可为 undefined
  score?: number;
  score_detail?: {
    value_score: number;
    quality_score: number;
    momentum_score: number;
    capital_score: number;
    risk_score: number;
    sentiment_score: number;
  };
  sentiment_score?: number;
  news_heat?: number;
  matched_themes?: string[];
  labels?: StockLabel[];
  instruction?: AIInstruction | null;
  // 盯盘模式专用
  hasQuote?: boolean;
}

/* ------------------------------------------------------------------ */
/*  Props                                                              */
/* ------------------------------------------------------------------ */

interface BaseProps {
  data: UnifiedStockRow[];
  loading: boolean;
  scrollY?: string;
}

interface StrategyModeProps extends BaseProps {
  mode: 'strategy';
  onStockClick: (row: UnifiedStockRow) => void;
  onRemove?: never;
  onRowClick?: never;
}

interface WatchlistModeProps extends BaseProps {
  mode: 'watchlist';
  onRowClick: (code: string) => void;
  onRemove: (code: string) => void;
  onStockClick?: never;
}

type StockTableProps = StrategyModeProps | WatchlistModeProps;

/* ------------------------------------------------------------------ */
/*  Component                                                          */
/* ------------------------------------------------------------------ */

export default function StockTable(props: StockTableProps) {
  const { data, loading, mode, scrollY } = props;

  /* — shared cell renderers — */

  const pctColor = useCallback(
    (v: number) => (v > 0 ? 'text-functional-up' : v < 0 ? 'text-functional-down' : 'text-txt-secondary'),
    [],
  );

  const pctCell = useCallback(
    (val: number, row?: UnifiedStockRow) => {
      if (mode === 'watchlist' && row && !row.hasQuote) return <span className="text-txt-muted">-</span>;
      const color = pctColor(val);
      return (
        <span className={`${color} font-mono font-semibold tabular-nums`}>
          {val > 0 ? '+' : ''}{val.toFixed(2)}%
        </span>
      );
    },
    [mode, pctColor],
  );

  /** 如果是盯盘模式且无行情，返回 '-' */
  const guard = useCallback(
    (row: UnifiedStockRow) => mode === 'watchlist' && !row.hasQuote,
    [mode],
  );

  /* — columns — */

  const columns: ColumnsType<UnifiedStockRow> = useMemo(() => {
    const cols: ColumnsType<UnifiedStockRow> = [];

    // 代码
    cols.push({
      title: '代码',
      dataIndex: 'code',
      key: 'code',
      width: 75,
      fixed: 'left',
      render: (code: string) => (
        <span className="font-mono text-txt-secondary text-xs">{code.replace(/^(sh|sz|bj)/, '')}</span>
      ),
    });

    // 名称
    cols.push({
      title: '名称',
      dataIndex: 'name',
      key: 'name',
      width: 80,
      fixed: 'left',
      render: (name: string, record) => (
        <button
          className="text-txt-primary hover:text-primary-gold transition-colors font-medium cursor-pointer bg-transparent border-none text-left"
          onClick={(e) => {
            e.stopPropagation();
            if (mode === 'strategy' && props.onStockClick) {
              props.onStockClick(record);
            } else if (mode === 'watchlist' && props.onRowClick) {
              props.onRowClick(record.code);
            }
          }}
        >
          {name}
        </button>
      ),
    });

    // 综合分 (策略模式)
    if (mode === 'strategy') {
      cols.push({
        title: '综合分',
        dataIndex: 'score',
        key: 'score',
        width: 70,
        sorter: (a, b) => (a.score ?? 0) - (b.score ?? 0),
        defaultSortOrder: 'descend',
        render: (score: number, record) => (
          <Tooltip
            title={
              <div className="text-xs space-y-1">
                <div>价值: {((record.score_detail?.value_score ?? 0) * 100).toFixed(0)}</div>
                <div>质量: {((record.score_detail?.quality_score ?? 0) * 100).toFixed(0)}</div>
                <div>动量: {((record.score_detail?.momentum_score ?? 0) * 100).toFixed(0)}</div>
                <div>资金: {((record.score_detail?.capital_score ?? 0) * 100).toFixed(0)}</div>
                <div>风险: {((record.score_detail?.risk_score ?? 0) * 100).toFixed(0)}</div>
                <div>消息: {((record.score_detail?.sentiment_score ?? 0) * 100).toFixed(0)}</div>
                {record.matched_themes && record.matched_themes.length > 0 && (
                  <div className="pt-1 border-t border-white/20">
                    主题: {record.matched_themes.join('、')}
                  </div>
                )}
              </div>
            }
          >
            <span><ScoreBadge score={score ?? 0} /></span>
          </Tooltip>
        ),
      });
    }

    // 最新价
    cols.push({
      title: '最新价',
      dataIndex: 'price',
      key: 'price',
      width: 70,
      sorter: (a, b) => a.price - b.price,
      render: (v: number, r) => (
        <span className={`font-mono tabular-nums ${mode === 'watchlist' ? pctColor(r.change_pct) : 'text-txt-primary'}`}>
          {guard(r) ? '-' : v.toFixed(2)}
        </span>
      ),
    });

    // 涨跌%
    cols.push({
      title: '涨跌%',
      dataIndex: 'change_pct',
      key: 'change_pct',
      width: 72,
      sorter: (a, b) => a.change_pct - b.change_pct,
      defaultSortOrder: mode === 'watchlist' ? 'descend' : undefined,
      render: (v: number, r) => pctCell(v, r),
    });

    // PE
    cols.push({
      title: 'PE(TTM)',
      dataIndex: 'pe_ttm',
      key: 'pe_ttm',
      width: 72,
      sorter: (a, b) => a.pe_ttm - b.pe_ttm,
      render: (v: number, r) => (
        <span
          className={`font-mono tabular-nums ${
            guard(r) ? 'text-txt-muted' : v > 0 && v < 20 ? 'text-functional-down' : v > 60 ? 'text-primary-orange' : 'text-txt-secondary'
          }`}
        >
          {guard(r) ? '-' : v > 0 ? v.toFixed(1) : '-'}
        </span>
      ),
    });

    // PB
    cols.push({
      title: 'PB',
      dataIndex: 'pb',
      key: 'pb',
      width: 55,
      sorter: (a, b) => a.pb - b.pb,
      render: (v: number, r) => (
        <span
          className={`font-mono tabular-nums ${
            guard(r) ? 'text-txt-muted' : v > 0 && v < 2 ? 'text-functional-down' : 'text-txt-secondary'
          }`}
        >
          {guard(r) ? '-' : v > 0 ? v.toFixed(2) : '-'}
        </span>
      ),
    });

    // ROE
    cols.push({
      title: 'ROE%',
      dataIndex: 'roe',
      key: 'roe',
      width: 65,
      sorter: (a, b) => a.roe - b.roe,
      render: (v: number, r) => (
        <span
          className={`font-mono tabular-nums ${
            guard(r) ? 'text-txt-muted' : v >= 20 ? 'text-primary-gold' : v >= 10 ? 'text-functional-up' : 'text-txt-secondary'
          }`}
        >
          {guard(r) ? '-' : v > 0 ? v.toFixed(1) : '-'}
        </span>
      ),
    });

    // 营收增% (策略模式)
    if (mode === 'strategy') {
      cols.push({
        title: '营收增%',
        dataIndex: 'revenue_yoy',
        key: 'revenue_yoy',
        width: 72,
        sorter: (a, b) => a.revenue_yoy - b.revenue_yoy,
        render: (v: number, r) => pctCell(v, r),
      });
    }

    // 市值
    cols.push({
      title: '市值(亿)',
      dataIndex: 'total_market_cap',
      key: 'total_market_cap',
      width: 80,
      sorter: (a, b) => a.total_market_cap - b.total_market_cap,
      render: (v: number, r) => {
        if (guard(r)) return <span className="text-txt-muted">-</span>;
        return (
          <span className={`font-mono tabular-nums ${v >= 1000 ? 'text-functional-info' : 'text-txt-secondary'}`}>
            {v >= 1 ? v.toFixed(0) : v.toFixed(2)}
          </span>
        );
      },
    });

    // 换手%
    cols.push({
      title: '换手%',
      dataIndex: 'turnover_rate',
      key: 'turnover_rate',
      width: 62,
      sorter: (a, b) => a.turnover_rate - b.turnover_rate,
      render: (v: number, r) => (
        <span className="font-mono tabular-nums text-txt-secondary">
          {guard(r) ? '-' : v.toFixed(2)}
        </span>
      ),
    });

    // 量比
    cols.push({
      title: '量比',
      dataIndex: 'volume_ratio',
      key: 'volume_ratio',
      width: 55,
      sorter: (a, b) => a.volume_ratio - b.volume_ratio,
      render: (v: number, r) => (
        <span className={`font-mono tabular-nums ${v >= 2 ? 'text-primary-gold' : 'text-txt-secondary'}`}>
          {guard(r) ? '-' : v.toFixed(2)}
        </span>
      ),
    });

    // 主力净流入 (万)
    cols.push({
      title: '主力净流入',
      dataIndex: 'main_net_inflow',
      key: 'main_net_inflow',
      width: 90,
      sorter: (a, b) => a.main_net_inflow - b.main_net_inflow,
      render: (v: number, r) => {
        if (guard(r)) return <span className="text-txt-muted">-</span>;
        const color = v > 0 ? 'text-functional-up' : v < 0 ? 'text-functional-down' : 'text-txt-secondary';
        const abs = Math.abs(v);
        const display = abs >= 10000 ? `${(v / 10000).toFixed(2)}亿` : `${v.toFixed(0)}万`;
        return <span className={`font-mono tabular-nums font-semibold ${color}`}>{display}</span>;
      },
    });

    // 5日%
    cols.push({
      title: '5日%',
      dataIndex: 'pct_5d',
      key: 'pct_5d',
      width: 65,
      sorter: (a, b) => a.pct_5d - b.pct_5d,
      render: (v: number, r) => pctCell(v, r),
    });

    // 20日%
    cols.push({
      title: '20日%',
      dataIndex: 'pct_20d',
      key: 'pct_20d',
      width: 65,
      sorter: (a, b) => a.pct_20d - b.pct_20d,
      render: (v: number, r) => pctCell(v, r),
    });

    // 成交额 (盯盘模式)
    if (mode === 'watchlist') {
      cols.push({
        title: '成交额',
        dataIndex: 'amount',
        key: 'amount',
        width: 80,
        sorter: (a, b) => a.amount - b.amount,
        render: (v: number, r) => {
          if (guard(r) || v <= 0) return <span className="text-txt-muted">-</span>;
          if (v >= 10000) return <span className="text-primary-gold font-semibold">{(v / 10000).toFixed(2)}亿</span>;
          return <span className="text-txt-secondary">{v.toFixed(0)}万</span>;
        },
      });
    }

    // 标签 (两种模式都显示)
    cols.push({
      title: '标签',
      key: 'labels',
      width: 160,
      render: (_: unknown, record: UnifiedStockRow) => {
        const labels = record.labels || [];
        if (labels.length === 0) return <span className="text-txt-muted text-xs">-</span>;
        return (
          <div className="flex flex-wrap gap-1">
            {labels.slice(0, 3).map((label, i) => (
              <TagLabel key={i} label={label} />
            ))}
          </div>
        );
      },
    });

    // AI (策略模式)
    if (mode === 'strategy') {
      cols.push({
        title: 'AI',
        key: 'instruction',
        width: 100,
        render: (_: unknown, record: UnifiedStockRow) =>
          record.instruction
            ? <InstructionTag instruction={record.instruction} />
            : <span className="text-txt-muted text-xs">-</span>,
      });
    }

    // 删除按钮 (盯盘模式)
    if (mode === 'watchlist') {
      cols.push({
        title: '',
        key: 'action',
        width: 40,
        render: (_: unknown, record: UnifiedStockRow) => (
          <button
            onClick={(e) => {
              e.stopPropagation();
              if (props.onRemove) props.onRemove(record.code);
            }}
            className="p-1 rounded hover:bg-red-500/20 transition-all cursor-pointer opacity-30 hover:opacity-100"
          >
            <Trash2 size={12} className="text-functional-up" />
          </button>
        ),
      });
    }

    return cols;
  }, [mode, pctCell, pctColor, guard, props]);

  /* — row click (watchlist) — */
  const onRow = useMemo(() => {
    if (mode !== 'watchlist') return undefined;
    return (record: UnifiedStockRow) => ({
      onClick: () => {
        if (props.onRowClick) props.onRowClick(record.code);
      },
      style: { cursor: 'pointer' as const },
    });
  }, [mode, props]);

  const scrollX = mode === 'strategy' ? 1500 : 1400;
  const yVal = scrollY || (mode === 'strategy' ? 'calc(100vh - 170px)' : 'calc(100vh - 130px)');

  return (
    <Table
      columns={columns}
      dataSource={data}
      rowKey="code"
      loading={loading}
      pagination={false}
      size="small"
      scroll={{ x: scrollX, y: yVal }}
      className="stock-table"
      onRow={onRow}
    />
  );
}

/* ------------------------------------------------------------------ */
/*  Helper: convert StrategyResultRow → UnifiedStockRow                */
/* ------------------------------------------------------------------ */

export function strategyRowToUnified(row: StrategyResultRow): UnifiedStockRow {
  return {
    ...row,
    hasQuote: true,
  };
}

/* ------------------------------------------------------------------ */
/*  Helper: convert WatchlistQuote → UnifiedStockRow (单位换算)         */
/* ------------------------------------------------------------------ */

export interface WatchlistQuoteRow {
  code: string;
  name: string;
  price: number;
  change_pct: number;
  pe_ttm: number;
  pb: number;
  roe: number;
  revenue_yoy: number;
  /** 元 → 亿 */
  total_market_cap: number;
  turnover_rate: number;
  volume_ratio: number;
  /** 元 → 万 */
  main_net_inflow: number;
  /** 元 → 万 */
  amount: number;
  pct_5d: number;
  pct_20d: number;
  hasQuote: boolean;
}

export function watchlistQuoteToUnified(row: WatchlistQuoteRow): UnifiedStockRow {
  return {
    code: row.code,
    name: row.name,
    price: row.price,
    change_pct: row.change_pct,
    pe_ttm: row.pe_ttm,
    pb: row.pb,
    roe: row.roe,
    revenue_yoy: row.revenue_yoy,
    total_market_cap: row.total_market_cap / 1e8,  // 元 → 亿
    turnover_rate: row.turnover_rate,
    volume_ratio: row.volume_ratio,
    main_net_inflow: row.main_net_inflow / 1e4,    // 元 → 万
    amount: row.amount / 1e4,                       // 元 → 万
    pct_5d: row.pct_5d,
    pct_20d: row.pct_20d,
    hasQuote: row.hasQuote,
  };
}
