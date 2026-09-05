import { motion } from "framer-motion";
import { useLocation } from "react-router";

const titles: Record<string, string> = {
  "/dashboard": "Dashboard",
  "/mods": "Mods",
  "/translation": "Traducción",
  "/editor": "Editor",
  "/stats": "Estadísticas",
  "/history": "Historial",
  "/diagnostics": "Diagnóstico",
  "/settings": "Ajustes",
};

export default function TopBar() {
  const location = useLocation();
  const title = titles[location.pathname] ?? "Solano Mod Translator";

  return (
    <motion.header
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      className="h-16 glass border-b border-glass-border flex items-center justify-between px-6 relative z-20"
    >
      <div className="flex items-center gap-2 text-sm">
        <span className="text-white/40">Solano</span>
        <span className="text-white/20">/</span>
        <span className="font-medium">{title}</span>
      </div>

      <div className="flex items-center gap-3">
        <div className="w-8 h-8 rounded-full bg-gradient-to-br from-accent to-pink-500" />
      </div>
    </motion.header>
  );
}
