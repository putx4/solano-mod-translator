import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { motion } from "framer-motion";
import GlassCard from "../components/ui/GlassCard";
import Badge from "../components/ui/Badge";
import { FolderOpen, Package, Search, Filter, Loader2 } from "lucide-react";
import { useModStore } from "../stores/modStore";

export default function Mods() {
  const { mods, scanning, scanMessage, scanFolder, hasScanned } = useModStore();
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState<"all" | "forge" | "fabric" | "neoforge" | "quilt">("all");

  const handleSelectFolder = async () => {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (selected) {
        await scanFolder(selected as string);
      }
    } catch (error) {
      console.error("Error:", error);
    }
  };

  const filtered = mods.filter((m) => {
    const matchSearch = m.name.toLowerCase().includes(search.toLowerCase()) ||
                       m.id.toLowerCase().includes(search.toLowerCase());
    const matchFilter = filter === "all" || m.loader.toLowerCase() === filter;
    return matchSearch && matchFilter;
  });

  const loaderColors: Record<string, "info" | "success" | "warning" | "error" | "neutral"> = {
    forge: "error",
    neoforge: "warning",
    fabric: "info",
    quilt: "success",
    unknown: "neutral",
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Mods</h1>
          <p className="text-white/50 mt-1">
            {mods.length > 0 ? `${mods.length} mods detectados` : "Ningún mod cargado"}
          </p>
        </div>
        <button
          onClick={handleSelectFolder}
          className="btn btn-primary"
          disabled={scanning}
        >
          {scanning ? <Loader2 size={16} className="animate-spin" /> : <FolderOpen size={16} />}
          {scanning ? "Escaneando..." : "Seleccionar carpeta de mods"}
        </button>
      </div>

      {scanning && (
        <GlassCard className="border-accent/30">
          <div className="flex items-center gap-3">
            <Loader2 size={20} className="animate-spin text-accent" />
            <div>
              <p className="text-sm font-medium">{scanMessage}</p>
              <p className="text-xs text-white/50 mt-1">Detectando loaders y archivos de idioma...</p>
            </div>
          </div>
        </GlassCard>
      )}

      {hasScanned && mods.length > 0 && (
        <>
          <GlassCard className="flex gap-3 flex-wrap">
            <div className="flex items-center gap-2 flex-1 min-w-[200px] px-3 py-2 rounded-lg bg-white/5 border border-glass-border">
              <Search size={16} className="text-white/40" />
              <input
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="Buscar mods..."
                className="bg-transparent outline-none text-sm flex-1"
              />
            </div>
            <div className="flex gap-2 flex-wrap">
              {(["all", "forge", "neoforge", "fabric", "quilt"] as const).map((f) => (
                <button
                  key={f}
                  onClick={() => setFilter(f)}
                  className={`btn ${filter === f ? "btn-primary" : "btn-ghost"}`}
                >
                  {f === "all" && <Filter size={14} />}
                  {f.charAt(0).toUpperCase() + f.slice(1)}
                </button>
              ))}
            </div>
          </GlassCard>

          <div className="grid grid-cols-1 gap-3">
            {filtered.map((mod, i) => (
              <motion.div
                key={mod.path}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: Math.min(i * 0.02, 0.5) }}
              >
                <GlassCard>
                  {/* Línea 1: icono + nombre + badges */}
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-accent to-purple-700 flex items-center justify-center flex-shrink-0">
                      <Package size={18} className="text-white" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 flex-wrap">
                        <h3 className="font-semibold truncate max-w-[300px]">{mod.name}</h3>
                        <Badge variant={loaderColors[mod.loader.toLowerCase()] || "neutral"}>
                          {mod.loader}
                        </Badge>
                        <span className="text-xs text-white/40">v{mod.version}</span>
                      </div>
                      <p className="text-sm text-white/50 mt-0.5 truncate">
                        {mod.author} · MC {mod.mc_version || "?"} · {(mod.size_bytes / 1024).toFixed(0)} KB
                      </p>
                    </div>
                  </div>

                  {/* Línea 2: archivos de idioma (separada, sin superposición) */}
                  {mod.lang_files.length > 0 && (
                    <div className="mt-3 pt-3 border-t border-glass-border">
                      <div className="flex gap-2 flex-wrap items-center">
                        <span className="text-xs text-white/40 mr-1">Idiomas:</span>
                        {mod.lang_files.map((lf) => (
                          <Badge key={lf.path} variant="info" className="font-mono text-xs">
                            {lf.locale}
                            {lf.keys_count > 0 && ` · ${lf.keys_count} keys`}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  )}

                  {mod.lang_files.length === 0 && (
                    <div className="mt-3 pt-3 border-t border-glass-border">
                      <span className="text-xs text-white/30">⚠️ Sin archivos de idioma</span>
                    </div>
                  )}
                </GlassCard>
              </motion.div>
            ))}
          </div>
        </>
      )}

      {hasScanned && mods.length === 0 && !scanning && (
        <GlassCard className="text-center py-12">
          <Package size={48} className="mx-auto text-white/20 mb-4" />
          <p className="text-white/60">No se encontraron mods en esa carpeta.</p>
        </GlassCard>
      )}

      {!hasScanned && !scanning && (
        <GlassCard className="text-center py-12">
          <Package size={48} className="mx-auto text-white/20 mb-4" />
          <p className="text-white/60">Selecciona tu carpeta de mods para empezar.</p>
        </GlassCard>
      )}
    </div>
  );
}