import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import GlassCard from "../components/ui/GlassCard";
import { Loader2, Languages, Cpu, Zap, AlertCircle, Package } from "lucide-react";

interface Stats {
  total_translated: number;
  total_ai_requests: number;
  cache_hit_percent: number;
  money_saved: number;
  total_time_secs: number;
  errors: number;
  mods_processed: number;
}

export default function Stats() {
  const [stats, setStats] = useState<Stats | null>(null);

  useEffect(() => {
    invoke<Stats>("get_stats")
      .then(setStats)
      .catch((e) => console.error("Error loading stats:", e));
  }, []);

  const cards = [
    { icon: Languages, label: "Strings traducidos", value: stats?.total_translated ?? 0, color: "from-accent to-purple-500" },
    { icon: Zap, label: "Peticiones a IA", value: stats?.total_ai_requests ?? 0, color: "from-yellow-500 to-orange-500" },
    { icon: Cpu, label: "Cache hit", value: stats ? `${stats.cache_hit_percent.toFixed(1)}%` : "0%", color: "from-green-500 to-emerald-500" },
    { icon: Package, label: "Mods procesados", value: stats?.mods_processed ?? 0, color: "from-blue-500 to-cyan-500" },
  ];

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold">Estadísticas</h1>
      <p className="text-white/50 mt-1 -mt-4">Métricas del traductor</p>

      {!stats ? (
        <GlassCard className="flex items-center justify-center py-16">
          <Loader2 size={24} className="animate-spin text-accent" />
        </GlassCard>
      ) : (
        <>
          <div className="grid grid-cols-4 gap-4">
            {cards.map((card, i) => (
              <GlassCard key={card.label} delay={i * 0.05}>
                <div className="flex items-start justify-between">
                  <div>
                    <p className="text-sm text-white/50">{card.label}</p>
                    <p className="text-3xl font-bold mt-2">{typeof card.value === "number" ? card.value.toLocaleString() : card.value}</p>
                  </div>
                  <div className={`w-10 h-10 rounded-lg bg-gradient-to-br ${card.color} flex items-center justify-center shadow-lg`}>
                    <card.icon size={20} className="text-white" />
                  </div>
                </div>
              </GlassCard>
            ))}
          </div>

          <GlassCard>
            <h3 className="font-semibold mb-4 flex items-center gap-2">
              <AlertCircle size={18} className="text-yellow-400" />
              Rendimiento
            </h3>
            <div className="space-y-4 text-sm">
              <div>
                <div className="flex justify-between mb-1">
                  <span className="text-white/60">Tasa de acierto de caché</span>
                  <span className="font-mono text-green-400">{stats.cache_hit_percent.toFixed(1)}%</span>
                </div>
                <div className="h-2 bg-white/5 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-gradient-to-r from-green-500 to-emerald-400"
                    style={{ width: `${Math.min(100, stats.cache_hit_percent)}%` }}
                  />
                </div>
              </div>
              <div className="flex justify-between">
                <span className="text-white/60">Errores registrados</span>
                <span className="font-mono text-red-400">{stats.errors}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-white/60">Tiempo total (seg)</span>
                <span className="font-mono">{stats.total_time_secs}</span>
              </div>
            </div>
          </GlassCard>
        </>
      )}
    </div>
  );
}
