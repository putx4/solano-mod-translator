import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import GlassCard from "../components/ui/GlassCard";
import Badge from "../components/ui/Badge";
import { Package, Languages, Cpu, Zap, AlertCircle, CheckCircle2, FolderOpen, Loader2 } from "lucide-react";
import { useNavigate } from "react-router";
import { useModStore } from "../stores/modStore";
import { useTranslationStore } from "../stores/translationStore";
import ProgressBar from "../components/ui/ProgressBar";

interface Stats {
  total_translated: number;
  total_ai_requests: number;
  cache_hit_percent: number;
  mods_processed: number;
  errors: number;
}

export default function Dashboard() {
  const [stats, setStats] = useState<Stats | null>(null);
  const navigate = useNavigate();
  const { mods, hasScanned } = useModStore();
  const { activeJobs, completedJobs } = useTranslationStore();

  useEffect(() => {
    invoke<Stats>("get_stats")
      .then(setStats)
      .catch((e) => console.error("Error loading stats:", e));
  }, [completedJobs.length]);

  const cards = [
    { icon: Package, label: "Mods cargados", value: mods.length, color: "from-blue-500 to-cyan-500" },
    { icon: Languages, label: "Strings traducidos", value: stats?.total_translated ?? 0, color: "from-accent to-purple-500" },
    { icon: Zap, label: "Requests a IA", value: stats?.total_ai_requests ?? 0, color: "from-yellow-500 to-orange-500" },
    { icon: Cpu, label: "Cache hit", value: `${(stats?.cache_hit_percent ?? 0).toFixed(1)}%`, color: "from-green-500 to-emerald-500" },
  ];

  return (
    <div className="space-y-6">
      <motion.div initial={{ opacity: 0, y: -10 }} animate={{ opacity: 1, y: 0 }}>
        <h1 className="text-3xl font-bold">Dashboard</h1>
        <p className="text-white/50 mt-1">Estado general del traductor</p>
      </motion.div>

      <div className="grid grid-cols-4 gap-4">
        {cards.map((card, i) => (
          <GlassCard key={card.label} delay={i * 0.05}>
            <div className="flex items-start justify-between">
              <div>
                <p className="text-sm text-white/50">{card.label}</p>
                <p className="text-3xl font-bold mt-2">{card.value.toLocaleString()}</p>
              </div>
              <div className={`w-10 h-10 rounded-lg bg-gradient-to-br ${card.color} flex items-center justify-center shadow-lg`}>
                <card.icon size={20} className="text-white" />
              </div>
            </div>
          </GlassCard>
        ))}
      </div>

      <div className="grid grid-cols-3 gap-4">
        {/* Traducciones activas */}
        <GlassCard className="col-span-2" delay={0.2}>
          <h3 className="font-semibold mb-4 flex items-center gap-2">
            {Object.keys(activeJobs).length > 0 ? (
              <Loader2 size={18} className="animate-spin text-accent" />
            ) : (
              <CheckCircle2 size={18} className="text-green-400" />
            )}
            Traducciones {Object.keys(activeJobs).length > 0 ? "en curso" : "recientes"}
          </h3>

          {Object.keys(activeJobs).length > 0 ? (
            <div className="space-y-3">
              {Object.values(activeJobs).slice(0, 5).map((job) => (
                <div key={job.job_id} className="flex items-center gap-3">
                  <span className="text-sm flex-1 truncate">{job.mod_name}</span>
                  <div className="flex-1">
                    <ProgressBar value={job.percent} showLabel={false} />
                  </div>
                  <span className="text-xs text-white/50 w-12 text-right">{job.percent.toFixed(0)}%</span>
                </div>
              ))}
            </div>
          ) : completedJobs.length > 0 ? (
            <div className="space-y-2">
              {completedJobs.slice(-5).reverse().map((job) => (
                <div key={job.job_id} className="flex items-center justify-between py-1">
                  <span className="text-sm">{job.mod_name}</span>
                  <Badge variant="success">Completado</Badge>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-8">
              <p className="text-white/40 text-sm">
                {hasScanned
                  ? "No hay traducciones en curso. Ve a la pestaña Traducción para empezar."
                  : "Escanea una carpeta de mods para comenzar."}
              </p>
              {!hasScanned && (
                <button onClick={() => navigate("/mods")} className="btn btn-ghost mt-3 text-sm">
                  <FolderOpen size={14} /> Ir a Mods
                </button>
              )}
            </div>
          )}
        </GlassCard>

        {/* Estado del sistema */}
        <GlassCard delay={0.3}>
          <h3 className="font-semibold mb-4 flex items-center gap-2">
            <AlertCircle size={18} className="text-yellow-400" />
            Sistema
          </h3>
          <div className="space-y-3 text-sm">
            <div className="flex justify-between">
              <span className="text-white/60">Mods en memoria</span>
              <span className="font-mono">{mods.length}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-white/60">Traducciones activas</span>
              <span className="font-mono">{Object.keys(activeJobs).length}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-white/60">Completadas</span>
              <span className="font-mono text-green-400">{completedJobs.length}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-white/60">Errores</span>
              <span className="font-mono text-red-400">{stats?.errors ?? 0}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-white/60">Base de datos</span>
              <span className="flex items-center gap-1 text-green-400">
                <span className="status-dot success" /> Conectada
              </span>
            </div>
          </div>
        </GlassCard>
      </div>
    </div>
  );
}