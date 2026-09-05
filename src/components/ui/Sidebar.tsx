import { NavLink } from "react-router";
import { motion } from "framer-motion";
import {
  LayoutDashboard, Package, Languages, Edit3,
  BarChart3, History as HistoryIcon, Stethoscope, Settings,
} from "lucide-react";
import clsx from "clsx";

const items = [
  { to: "/dashboard", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/mods", icon: Package, label: "Mods" },
  { to: "/translation", icon: Languages, label: "Translation" },
  { to: "/editor", icon: Edit3, label: "Editor" },
  { to: "/stats", icon: BarChart3, label: "Stats" },
  { to: "/history", icon: HistoryIcon, label: "History" },
  { to: "/diagnostics", icon: Stethoscope, label: "Diagnostics" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export default function Sidebar() {
  return (
    <aside className="w-64 h-full glass border-r border-glass-border flex flex-col relative z-20">
      <div className="p-6 border-b border-glass-border">
        <motion.div
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          className="flex items-center gap-3"
        >
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-accent to-purple-700 flex items-center justify-center shadow-glow">
            <span className="text-xl">🧩</span>
          </div>
          <div>
            <h1 className="font-bold text-lg">Solano</h1>
            <p className="text-xs text-white/50">Mod Translator</p>
          </div>
        </motion.div>
      </div>

      <nav className="flex-1 p-3 space-y-1">
        {items.map((item, i) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              clsx(
                "flex items-center gap-3 px-4 py-2.5 rounded-lg text-sm font-medium transition-all",
                isActive
                  ? "bg-accent/20 text-white border border-accent/30"
                  : "text-white/60 hover:text-white hover:bg-white/5"
              )
            }
          >
            <motion.div
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: i * 0.05 }}
              className="flex items-center gap-3 w-full"
            >
              <item.icon size={18} />
              <span>{item.label}</span>
            </motion.div>
          </NavLink>
        ))}
      </nav>

      <div className="p-4 border-t border-glass-border">
        <div className="text-xs text-white/40 text-center">v1.0.0</div>
      </div>
    </aside>
  );
}