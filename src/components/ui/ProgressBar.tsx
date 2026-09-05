import { motion } from "framer-motion";
import clsx from "clsx";

interface Props {
  value: number; // 0-100
  className?: string;
  showLabel?: boolean;
  variant?: "default" | "success" | "warning" | "error";
}

export default function ProgressBar({ value, className, showLabel = true, variant = "default" }: Props) {
  const colors = {
    default: "from-accent to-purple-500",
    success: "from-green-500 to-emerald-400",
    warning: "from-yellow-500 to-orange-400",
    error: "from-red-500 to-pink-500",
  };

  return (
    <div className={clsx("flex items-center gap-3", className)}>
      <div className="flex-1 h-2 bg-white/5 rounded-full overflow-hidden border border-glass-border">
        <motion.div
          initial={{ width: 0 }}
          animate={{ width: `${Math.min(100, Math.max(0, value))}%` }}
          transition={{ duration: 0.5, ease: [0.16, 1, 0.3, 1] }}
          className={clsx("h-full bg-gradient-to-r rounded-full relative", colors[variant])}
        >
          <div className="absolute inset-0 bg-white/20 animate-pulse" />
        </motion.div>
      </div>
      {showLabel && (
        <span className="text-sm font-mono text-white/70 w-12 text-right">
          {value.toFixed(1)}%
        </span>
      )}
    </div>
  );
}