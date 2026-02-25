import { StockLabel } from '../types';
import { Crown, Ruler, AlertTriangle, Target, Bomb, Grab } from 'lucide-react';

interface TagLabelProps {
  label: StockLabel;
}

const iconMap: Record<string, React.ReactNode> = {
  crown: <Crown size={11} />,
  king: <Crown size={11} />,
  ruler: <Ruler size={11} />,
  warning: <AlertTriangle size={11} />,
  target: <Target size={11} />,
  nuke: <Bomb size={11} />,
  grab: <Grab size={11} />,
};

export default function TagLabel({ label }: TagLabelProps) {
  const icon = label.icon ? iconMap[label.icon] : null;

  return (
    <span
      className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] font-medium whitespace-nowrap border border-opacity-30"
      style={{
        color: label.color,
        borderColor: `${label.color}40`,
        backgroundColor: `${label.color}15`,
      }}
    >
      {icon}
      {label.text}
    </span>
  );
}
