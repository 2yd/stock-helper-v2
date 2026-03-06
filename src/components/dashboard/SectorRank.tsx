import { SectorInfo, GlobalIndex } from '../../types';
import { Flame, Globe } from 'lucide-react';

interface Props {
  sectorTop: SectorInfo[];
  sectorBottom: SectorInfo[];
  globalIndexes: GlobalIndex[];
}

const regionLabel: Record<string, string> = {
  america: '美洲',
  asia: '亚太',
  europe: '欧洲',
  common: '重点',
};

export default function SectorRank({ sectorTop, sectorBottom, globalIndexes }: Props) {
  const groupedGlobal: Record<string, GlobalIndex[]> = {};
  globalIndexes.forEach(idx => {
    const key = idx.region || 'other';
    if (!groupedGlobal[key]) groupedGlobal[key] = [];
    groupedGlobal[key].push(idx);
  });

  return (
    <div className="grid grid-cols-2 gap-3">
      {/* 板块排行 */}
      <div className="rounded-xl bg-bg-card border border-[#30363D] p-3 hover:brightness-110 transition-all duration-200">
        <div className="flex items-center gap-1.5 mb-2.5">
          <Flame size={13} className="text-functional-up" />
          <span className="text-xs font-semibold text-txt-primary">板块热点</span>
        </div>

        {/* 领涨 */}
        <div className="space-y-1 mb-2.5">
          {sectorTop.map((s, i) => (
            <div key={`top-${i}`} className="flex items-center justify-between px-1.5 py-1 rounded hover:bg-bg-elevated transition-colors cursor-pointer">
              <div className="flex items-center gap-2 min-w-0">
                <span className="text-xs text-txt-primary truncate">{s.name}</span>
                <span className="text-[10px] text-txt-muted truncate">{s.lead_stock}</span>
              </div>
              <span className="text-xs font-din font-semibold text-functional-up flex-shrink-0">
                +{s.change_pct.toFixed(2)}%
              </span>
            </div>
          ))}
        </div>

        <div className="border-t border-[#30363D] my-2" />

        {/* 领跌 */}
        <div className="space-y-1">
          {sectorBottom.map((s, i) => (
            <div key={`bot-${i}`} className="flex items-center justify-between px-1.5 py-1 rounded hover:bg-bg-elevated transition-colors cursor-pointer">
              <div className="flex items-center gap-2 min-w-0">
                <span className="text-xs text-txt-primary truncate">{s.name}</span>
                <span className="text-[10px] text-txt-muted truncate">{s.lead_stock}</span>
              </div>
              <span className="text-xs font-din font-semibold text-functional-down flex-shrink-0">
                {s.change_pct.toFixed(2)}%
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* 全球指数 */}
      <div className="rounded-xl bg-bg-card border border-[#30363D] p-3 hover:brightness-110 transition-all duration-200">
        <div className="flex items-center gap-1.5 mb-2.5">
          <Globe size={13} className="text-functional-info" />
          <span className="text-xs font-semibold text-txt-primary">全球指数</span>
        </div>

        <div className="space-y-2">
          {Object.entries(groupedGlobal).map(([region, items]) => (
            <div key={region}>
              <div className="text-[10px] text-txt-muted mb-1 uppercase tracking-wider">
                {regionLabel[region] || region}
              </div>
              <div className="space-y-0.5">
                {items.map((idx, i) => {
                  const pctStr = idx.change_pct.replace('%', '');
                  const pctVal = parseFloat(pctStr);
                  const isUp = pctVal >= 0;
                  return (
                    <div key={i} className="flex items-center justify-between px-1 py-0.5">
                      <span className="text-xs text-txt-secondary truncate flex-1">{idx.name}</span>
                      <span className="text-xs font-din text-txt-primary mx-2">{idx.price}</span>
                      <span className={`text-xs font-din font-semibold ${isUp ? 'text-functional-up' : 'text-functional-down'}`}>
                        {isUp ? '+' : ''}{idx.change_pct}
                      </span>
                    </div>
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
