import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import GlassCard from "../components/ui/GlassCard";
import Badge from "../components/ui/Badge";
import { History as HistoryIcon, Loader2, RotateCcw } from "lucide-react";

interface HistoryEntry {
  id: number;
  mod_id: string;
  total_strings: number;
  translated: number;
  cached: number;
  failed: number;
  provider: string;
  started_at: string;
  finished_at: string | null;
}

export default function History() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);

  const load = async () => {
    setLoading(true);
    try {
      const data = await invoke<HistoryEntry[]>("get_history");
      setEntries(data);
    } catch (e) {
      console.error("Error loading history:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, []);

  const formatDate = (iso: string) => {
    try {
      return new Date(iso).toLocaleString();
    } catch {
      return iso;
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Historial</h1>
          <p className="text-white/50 mt-1">Registro de todas las traducciones realizadas</p>
        </div>
        <button onClick={load} className="btn btn-ghost text-sm" title="Actualizar">
          <RotateCcw size={14} />
          Actualizar
        </button>
      </div>

      {loading ? (
        <GlassCard className="flex items-center justify-center py-16">
          <Loader2 size={24} className="animate-spin text-accent" />
        </GlassCard>
      ) : entries.length === 0 ? (
        <GlassCard className="text-center py-16">
          <HistoryIcon size={48} className="mx-auto text-white/20 mb-4" />
          <p className="text-white/60">No hay traducciones registradas todavía.</p>
          <p className="text-white/40 text-sm mt-2">Cuando traduzcas un mod, aparecerá aquí su historial.</p>
        </GlassCard>
      ) : (
        <GlassCard>
          <div className="grid grid-cols-[1fr_auto_auto_auto_auto_auto_auto] gap-4 text-xs text-white/40 uppercase tracking-wider pb-2 border-b border-glass-border">
            <div>Mod</div>
            <div>Total</div>
            <div>Traducidas</div>
            <div>Caché</div>
            <div>Fallidas</div>
            <div>Proveedor</div>
            <div>Fecha</div>
          </div>
          <div className="divide-y divide-glass-border">
            {entries.map((e) => (
              <div
                key={e.id}
                className="grid grid-cols-[1fr_auto_auto_auto_auto_auto_auto] gap-4 py-3 items-center text-sm"
              >
                <span className="font-medium truncate" title={e.mod_id}>{e.mod_id}</span>
                <span className="font-mono text-white/60">{e.total_strings}</span>
                <span className="font-mono text-green-400">{e.translated}</span>
                <span className="font-mono text-blue-400">{e.cached}</span>
                <span className={`font-mono ${e.failed > 0 ? "text-red-400" : "text-white/40"}`}>{e.failed}</span>
                <Badge variant="neutral">{e.provider}</Badge>
                <span className="text-xs text-white/40">{formatDate(e.started_at)}</span>
              </div>
            ))}
          </div>
        </GlassCard>
      )}
    </div>
  );
}
