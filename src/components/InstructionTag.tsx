import { AIInstruction } from '../types';
import { Flame, Eye, X } from 'lucide-react';

interface InstructionTagProps {
  instruction: AIInstruction;
}

export default function InstructionTag({ instruction }: InstructionTagProps) {
  const config = {
    buy: {
      bg: 'bg-red-900/40',
      border: 'border-red-500/50',
      text: 'text-red-400',
      icon: <Flame size={13} className="text-red-400" />,
      glow: 'shadow-red-500/20',
    },
    watch: {
      bg: 'bg-orange-900/30',
      border: 'border-orange-500/40',
      text: 'text-orange-400',
      icon: <Eye size={13} className="text-orange-400" />,
      glow: '',
    },
    eliminate: {
      bg: 'bg-gray-800/50',
      border: 'border-gray-600/40',
      text: 'text-gray-500',
      icon: <X size={13} className="text-gray-500" />,
      glow: '',
    },
  }[instruction.action];

  return (
    <div
      className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-xs font-semibold border ${config.bg} ${config.border} ${config.text} ${config.glow} cursor-default`}
      title={instruction.reason}
    >
      {config.icon}
      <span>{instruction.label}</span>
    </div>
  );
}
