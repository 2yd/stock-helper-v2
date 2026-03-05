import { useMemo, useCallback } from 'react';
import { Table, Tooltip } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { Trash2 } from 'lucide-react';
import { StockLabel } from '../types';
import TagLabel from './TagLabel';

/* ------------------------------------------------------------------ */
/*  Unified row type — watchlist data                                  */
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
  labels?: StockLabel[];
  hasQuote?: boolean;
}

/* ------------------------------------------------------------------ */
/*  Props                                                              */
/* ------------------------------------------------------------------ */

interface StockTableProps {
  data: UnifiedStockRow[];
  loading: boolean;
  scrollY?: string;
  onRowClick: (code: string) => void;
  onRemove: (code: string) => void;
}

/* ------------------------------------------------------------------ */
/*  Component                                                          */
/* ------------------------------------------------------------------ */

export default function StockTable(props: StockTableProps) {
  const { data, loading, scrollY } = props;

  /* — shared cell renderers — */

  const pctColor = useCallback(
    (v: number) => (v > 0 ? 'text-functional-up' : v < 0 ? 'text-functional-down' : 'text-txt-secondary'),
    [],
  );

  const pctCell = useCallback(
    (val: number, row?: UnifiedStockRow) => {
      if (row && !row.hasQuote) return <span className="text-txt-muted">-</span>;
      const color = pctColor(val);
      return (
        <span className={`${color} font-mono font-semibold tabular-nums`}>
          {val > 0 ? '+' : ''}{val.toFixed(2)}%
        </span>
      );
    },
    [pctColor],
  );

  /** 如果无行情，返回 '-' */
  const guard = useCallback(
    (row: UnifiedStockRow) => !row.hasQuote,
    [],
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
            if (props.onRowClick) {
              props.onRowClick(record.code);
            }
          }}
        >
          {name}
        </button>
      ),
    });

    // 最新价
    cols.push({
      title: '最新价',
      dataIndex: 'price',
      key: 'price',
      width: 70,
      sorter: (a, b) => a.price - b.price,
      render: (v: number, r) => (
        <span className={`font-mono tabular-nums ${pctColor(r.change_pct)}`}>
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
      defaultSortOrder: 'descend',
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

    // 成交额
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

    // 标签
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

    // 删除按钮
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

    return cols;
  }, [pctCell, pctColor, guard, props]);

  /* — row click — */
  const onRow = useMemo(() => {
    return (record: UnifiedStockRow) => ({
      onClick: () => {
        if (props.onRowClick) props.onRowClick(record.code);
      },
      style: { cursor: 'pointer' as const },
    });
  }, [props]);

  const yVal = scrollY || 'calc(100vh - 130px)';

  return (
    <Table
      columns={columns}
      dataSource={data}
      rowKey="code"
      loading={loading}
      pagination={false}
      size="small"
      scroll={{ x: 1400, y: yVal }}
      className="stock-table"
      onRow={onRow}
    />
  );
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
