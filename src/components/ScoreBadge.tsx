interface ScoreBadgeProps {
  score: number;
}

export default function ScoreBadge({ score }: ScoreBadgeProps) {
  const getScoreStyle = () => {
    if (score >= 90) {
      return 'bg-gradient-to-r from-yellow-600 to-yellow-400 text-black font-bold shadow-lg shadow-yellow-500/30';
    }
    if (score >= 80) {
      return 'bg-gradient-to-r from-red-700 to-red-500 text-white font-bold shadow-lg shadow-red-500/20';
    }
    if (score >= 60) {
      return 'bg-gradient-to-r from-orange-600 to-orange-400 text-white font-semibold';
    }
    if (score >= 40) {
      return 'bg-[#2D333B] text-txt-secondary';
    }
    return 'bg-[#1C2333] text-txt-muted';
  };

  return (
    <span
      className={`inline-flex items-center justify-center min-w-[48px] px-2 py-0.5 rounded-md text-sm tabular-nums ${getScoreStyle()}`}
    >
      {score}分
    </span>
  );
}
