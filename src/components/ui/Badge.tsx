import clsx from "clsx";
import { ReactNode } from "react";

interface Props {
  children: ReactNode;
  variant?: "success" | "warning" | "error" | "info" | "neutral";
  className?: string;
}

export default function Badge({ children, variant = "neutral", className }: Props) {
  const colors = {
    success: "bg-green-500/10 text-green-400 border-green-500/20",
    warning: "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
    error: "bg-red-500/10 text-red-400 border-red-500/20",
    info: "bg-blue-500/10 text-blue-400 border-blue-500/20",
    neutral: "bg-white/5 text-white/60 border-white/10",
  };

  return (
    <span className={clsx(
      "inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-xs font-medium border",
      colors[variant],
      className
    )}>
      {children}
    </span>
  );
}