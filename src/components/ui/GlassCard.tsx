import { motion } from "framer-motion";
import { ReactNode } from "react";
import clsx from "clsx";

interface Props {
  children: ReactNode;
  className?: string;
  hover?: boolean;
  delay?: number;
}

export default function GlassCard({ children, className, hover = true, delay = 0 }: Props) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay, duration: 0.4, ease: [0.16, 1, 0.3, 1] }}
      className={clsx(
        "glass rounded-xl p-5",
        hover && "glass-hover cursor-default",
        className
      )}
    >
      {children}
    </motion.div>
  );
}