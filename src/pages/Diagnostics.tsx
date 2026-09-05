import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import GlassCard from "../components/ui/GlassCard";
import Badge from "../components/ui/Badge";
import { Stethoscope, Loader2, Wrench, CheckCircle2, AlertTriangle } from "lucide-react";
import { useModStore } from "../stores/modStore";

interface DiagnosticReport {
  mod_id: string;
  json_valid: boolean;
  total_keys: number;
  placeholders_ok: boolean;
  translated_percent: number;
  suspicious: string[];
  duplicates: string[];
  corrupted_files: string[];
  repairable: boolean;
}

export default function Diagnostics() {
  const { mods } = useModStore();
  const [selectedMod, setSelectedMod] = useState<string>("");
  const [report, setReport] = useState<DiagnosticReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [repairing, setRepairing] = useState(false);
  const [repairMsg, setRepairMsg] = useState<string | null>(null);

  const runDiagnostic = async (jarPath: string) => {
    setLoading(true);
    setReport(null);
    setRepairMsg(null);
    try {
      const res = await invoke<DiagnosticReport>("diagnose_mod", { jarPath });
      setReport(res);
    } catch (e) {
      console.error("Diagnostic error:", e);
      setReport(null);
    } finally {
      setLoading(false);
    }
  };

  const handleSelect = (path: string) => {
    setSelectedMod(path);
    runDiagnostic(path);
  };

  const handleRepair = async () => {
    if (!selectedMod) return;
    setRepairing(true);
    setRepairMsg(null);
    try {
      await invoke("repair_mod", { jarPath: selectedMod });
      setRepairMsg("Mod reparado correctamente.");
      // Re-run diagnostic after repair
      await runDiagnostic(selectedMod);
    } catch (e) {
      console.error("Repair error:", e);
      setRepairMsg(`Error al reparar: ${e}`);
    } finally {
      setRepairing(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Diagnóstico</h1>
          <p className="text-white/50 mt-1">Analiza y repara los mods escaneados</p>
        </div>
      </div>

      <GlassCard>
        <div className="flex items-center gap-2 mb-4">
          <Stethoscope size={18} className="text-accent" />
          <h3 className="font-semibold">Selecciona un mod</h3>
        </div>
        {mods.length === 0 ? (
          <p className="text-white/50 text-sm">
            No hay mods escaneados. Ve a la pestaña Mods y escanea una carpeta primero.
          </p>
        ) : (
          <div className="grid grid-cols-2 gap-2 max-h-64 overflow-y-auto">
            {mods.map((m) => (
              <button
                key={m.path}
                onClick={() => handleSelect(m.path)}
                className={`text-left px-3 py-2 rounded-lg border text-sm transition-colors ${
                  selectedMod === m.path
                    ? "border-accent bg-accent/10"
                    : "border-glass-border bg-white/5 hover:bg-white/10"
                }`}
              >
                <span className="font-medium truncate block">{m.name}</span>
                <span className="text-xs text-white/40">{m.version}</span>
              </button>
            ))}
          </div>
        )}
      </GlassCard>

      {loading && (
        <GlassCard className="flex items-center justify-center py-10">
          <Loader2 size={24} className="animate-spin text-accent" />
        </GlassCard>
      )}

      {report && !loading && (
        <GlassCard>
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-semibold flex items-center gap-2">
              <CheckCircle2 size={18} className="text-green-400" />
              Informe de {report.mod_id || "mod"}
            </h3>
            {report.repairable && (
              <button
                onClick={handleRepair}
                disabled={repairing}
                className="btn btn-primary text-sm disabled:opacity-50"
              >
                <Wrench size={14} />
                {repairing ? "Reparando..." : "Reparar"}
              </button>
            )}
          </div>

          {repairMsg && (
            <p className="text-sm text-green-400 mb-3">{repairMsg}</p>
          )}

          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
            <div className="rounded-lg bg-white/5 p-3">
              <div className="text-xs text-white/40 uppercase">Archivos JSON</div>
              <div className={`font-semibold ${report.json_valid ? "text-green-400" : "text-red-400"}`}>
                {report.json_valid ? "Válidos" : "Con errores"}
              </div>
            </div>
            <div className="rounded-lg bg-white/5 p-3">
              <div className="text-xs text-white/40 uppercase">Claves totales</div>
              <div className="font-semibold">{report.total_keys}</div>
            </div>
            <div className="rounded-lg bg-white/5 p-3">
              <div className="text-xs text-white/40 uppercase">Traducido</div>
              <div className="font-semibold text-blue-400">{report.translated_percent.toFixed(1)}%</div>
            </div>
            <div className="rounded-lg bg-white/5 p-3">
              <div className="text-xs text-white/40 uppercase">Placeholders</div>
              <div className={`font-semibold ${report.placeholders_ok ? "text-green-400" : "text-red-400"}`}>
                {report.placeholders_ok ? "OK" : "Con problemas"}
              </div>
            </div>
          </div>

          {report.suspicious.length > 0 && (
            <div className="mb-3">
              <div className="flex items-center gap-2 text-sm font-medium text-yellow-400 mb-2">
                <AlertTriangle size={14} /> Sospechosas ({report.suspicious.length})
              </div>
              <div className="flex flex-wrap gap-1">
                {report.suspicious.slice(0, 20).map((k) => (
                  <Badge key={k} variant="warning">{k}</Badge>
                ))}
                {report.suspicious.length > 20 && (
                  <Badge variant="neutral">+{report.suspicious.length - 20} más</Badge>
                )}
              </div>
            </div>
          )}

          {report.duplicates.length > 0 && (
            <div className="mb-3">
              <div className="flex items-center gap-2 text-sm font-medium text-red-400 mb-2">
                <AlertTriangle size={14} /> Duplicados ({report.duplicates.length})
              </div>
              <div className="flex flex-wrap gap-1">
                {report.duplicates.slice(0, 20).map((k) => (
                  <Badge key={k} variant="error">{k}</Badge>
                ))}
              </div>
            </div>
          )}

          {report.corrupted_files.length > 0 && (
            <div>
              <div className="flex items-center gap-2 text-sm font-medium text-red-400 mb-2">
                <AlertTriangle size={14} /> Archivos corruptos ({report.corrupted_files.length})
              </div>
              <div className="space-y-1">
                {report.corrupted_files.map((f) => (
                  <div key={f} className="text-xs font-mono text-white/50">{f}</div>
                ))}
              </div>
            </div>
          )}

          {!report.suspicious.length && !report.duplicates.length && !report.corrupted_files.length && (
            <p className="text-sm text-green-400">Sin problemas detectados.</p>
          )}
        </GlassCard>
      )}
    </div>
  );
}
