import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import GlassCard from "../components/ui/GlassCard";
import Badge from "../components/ui/Badge";
import ProgressBar from "../components/ui/ProgressBar";
import { Languages, FolderOpen, Play, Loader2, CheckCircle2 } from "lucide-react";
import { useNavigate } from "react-router";
import { useModStore } from "../stores/modStore";
import { useTranslationStore } from "../stores/translationStore";

const LANGS = [
  { code: "en_us", label: "English (US)" },
  { code: "es_es", label: "Español (ES)" },
  { code: "es_ar", label: "Español (Argentina)" },
  { code: "es_mx", label: "Español (México)" },
  { code: "pt_br", label: "Português (Brasil)" },
  { code: "fr_fr", label: "Français" },
  { code: "de_de", label: "Deutsch" },
  { code: "it_it", label: "Italiano" },
  { code: "ko_kr", label: "한국어" },
  { code: "ja_jp", label: "日本語" },
  { code: "zh_cn", label: "简体中文" },
  { code: "zh_tw", label: "繁體中文" },
  { code: "ru_ru", label: "Русский" },
];

export default function Translation() {
  const { mods, hasScanned, scannedFolder } = useModStore();
  const { activeJobs, completedJobs, startTranslation, setupListeners, cleanup } = useTranslationStore();
  const [sourceLocale, setSourceLocale] = useState("en_us");
  const [targetLocale, setTargetLocale] = useState("es_es");
  const [selectedPaths, setSelectedPaths] = useState<Set<string>>(new Set());
  const navigate = useNavigate();

  useEffect(() => {
    setupListeners();
    return () => cleanup();
  }, [setupListeners, cleanup]);

  if (!hasScanned || mods.length === 0) {
    return (
      <div className="space-y-6">
        <h1 className="text-3xl font-bold">Traducción</h1>
        <GlassCard className="text-center py-16">
          <Languages size={48} className="mx-auto text-white/20 mb-4" />
          <p className="text-white/60 mb-2">No hay mods cargados para traducir.</p>
          <p className="text-white/40 text-sm mb-6">Primero escanea una carpeta de mods.</p>
          <button onClick={() => navigate("/mods")} className="btn btn-primary mx-auto">
            <FolderOpen size={14} />
            Ir a Mods
          </button>
        </GlassCard>
      </div>
    );
  }

  const translatableMods = mods.filter((m) =>
    m.lang_files.some((lf) => lf.locale === sourceLocale)
  );

  const getJobForMod = (modId: string) => {
    return Object.values(activeJobs).find((j) => j.mod_id === modId) ||
           completedJobs.find((j) => j.mod_id === modId);
  };

  const handleTranslate = (mod: any) => {
    const outputFolder = scannedFolder
      ? scannedFolder.replace(/\\/g, "/").replace(/\/mods\/?$/, "") + "/translated_mods"
      : "translated_mods";
    
    startTranslation(mod.path, outputFolder, mod.id, mod.name, sourceLocale, targetLocale);
  };

  // Estado de un mod (en curso / completado)
  const modState = (modId: string) => {
    const job = getJobForMod(modId);
    if (!job) return { isTranslating: false, isCompleted: false };
    return {
      isTranslating: job.status === "translating" || job.status === "starting",
      isCompleted: job.status === "completed",
    };
  };

  const isModSelectable = (mod: any) => {
    const { isTranslating, isCompleted } = modState(mod.id);
    return !isTranslating && !isCompleted;
  };

  const toggleSelect = (path: string) => {
    setSelectedPaths((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  };

  const allSelectable = translatableMods.filter(isModSelectable);
  const allSelected = allSelectable.length > 0 && allSelectable.every((m) => selectedPaths.has(m.path));

  const handleSelectAll = () => {
    if (allSelected) {
      setSelectedPaths(new Set());
    } else {
      setSelectedPaths(new Set(allSelectable.map((m) => m.path)));
    }
  };

  const handleTranslateSelected = () => {
    const toTranslate = translatableMods.filter(
      (m) => selectedPaths.has(m.path) && isModSelectable(m)
    );
    toTranslate.forEach((mod) => handleTranslate(mod));
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Traducción</h1>
          <p className="text-white/50 mt-1">
            {translatableMods.length} mods con archivos en {LANGS.find(l => l.code === sourceLocale)?.label} disponibles
          </p>
        </div>
        <div className="flex items-center gap-3">
          <label className="flex items-center gap-2 text-sm text-white/60">
            De:
            <select
              value={sourceLocale}
              onChange={(e) => setSourceLocale(e.target.value)}
              className="bg-white/5 border border-glass-border rounded-md px-2 py-1 text-white text-sm"
            >
              {LANGS.map((l) => (
                <option key={l.code} value={l.code} className="bg-neutral-800">{l.label}</option>
              ))}
            </select>
          </label>
          <label className="flex items-center gap-2 text-sm text-white/60">
            A:
            <select
              value={targetLocale}
              onChange={(e) => setTargetLocale(e.target.value)}
              className="bg-white/5 border border-glass-border rounded-md px-2 py-1 text-white text-sm"
            >
              {LANGS.filter((l) => l.code !== sourceLocale).map((l) => (
                <option key={l.code} value={l.code} className="bg-neutral-800">{l.label}</option>
              ))}
            </select>
          </label>
          <div className="flex items-center gap-2 ml-2 border-l border-glass-border pl-3">
            <button
              onClick={handleSelectAll}
              disabled={allSelectable.length === 0}
              className="btn text-sm disabled:opacity-40 disabled:cursor-not-allowed"
            >
              {allSelected ? "Deseleccionar todos" : "Seleccionar todos"}
            </button>
            <button
              onClick={handleTranslateSelected}
              disabled={selectedPaths.size === 0}
              className="btn btn-primary text-sm disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1"
            >
              <Play size={12} />
              Traducir seleccionados ({selectedPaths.size})
            </button>
          </div>
        </div>
      </div>

      {/* Trabajos activos */}
      {Object.keys(activeJobs).length > 0 && (
        <GlassCard className="border-accent/30">
          <h3 className="font-semibold mb-4 flex items-center gap-2">
            <Loader2 size={18} className="animate-spin text-accent" />
            Traducciones en curso ({Object.keys(activeJobs).length})
          </h3>
          <div className="space-y-4">
            {Object.values(activeJobs).map((job) => (
              <div key={job.job_id} className="border-b border-glass-border last:border-0 pb-4 last:pb-0">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-3">
                    <h4 className="font-semibold">{job.mod_name}</h4>
                    <Badge variant="warning">
                      {job.status === "translating" ? "Traduciendo..." : job.status}
                    </Badge>
                  </div>
                  <div className="text-sm text-white/60 text-right">
                    <div>{job.translated.toLocaleString()} / {job.total.toLocaleString()}</div>
                    {job.cached > 0 && (
                      <div className="text-xs text-green-400">⚡ {job.cached} desde caché</div>
                    )}
                  </div>
                </div>
                <ProgressBar value={job.percent} />
                {job.current_key && (
                  <p className="text-xs text-white/40 mt-2 font-mono truncate">
                    Actual: {job.current_key}
                  </p>
                )}
              </div>
            ))}
          </div>
        </GlassCard>
      )}

      {/* Trabajos completados */}
      {completedJobs.length > 0 && (
        <GlassCard className="border-green-500/30">
          <h3 className="font-semibold mb-4 flex items-center gap-2">
            <CheckCircle2 size={18} className="text-green-400" />
            Completados ({completedJobs.length})
          </h3>
          <div className="space-y-2">
            {completedJobs.map((job) => (
              <div key={job.job_id} className="flex items-center justify-between py-2">
                <span className="font-medium">{job.mod_name}</span>
                <div className="flex items-center gap-3 text-sm">
                  <span className="text-white/60">{job.translated.toLocaleString()} strings</span>
                  <Badge variant="success">
                    <CheckCircle2 size={10} /> 100%
                  </Badge>
                </div>
              </div>
            ))}
          </div>
        </GlassCard>
      )}

      {/* Mods disponibles */}
      <GlassCard>
        <h3 className="font-semibold mb-4">Mods disponibles para traducir</h3>
        <div className="divide-y divide-glass-border">
          {translatableMods.map((mod, i) => {
            const { isTranslating, isCompleted } = modState(mod.id);
            const selectable = isModSelectable(mod);
            const srcFile = mod.lang_files.find((lf) => lf.locale === sourceLocale);
            const hasTarget = mod.lang_files.some((lf) => lf.locale === targetLocale);

            return (
              <motion.div
                key={mod.path}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: Math.min(i * 0.02, 0.3) }}
                className={`py-3 flex items-center gap-4 -mx-2 px-2 rounded transition-colors ${
                  selectedPaths.has(mod.path) ? "bg-accent/10" : "hover:bg-white/5"
                }`}
              >
                <input
                  type="checkbox"
                  checked={selectedPaths.has(mod.path)}
                  onChange={() => toggleSelect(mod.path)}
                  disabled={!selectable}
                  className="w-4 h-4 accent-current cursor-pointer disabled:cursor-not-allowed disabled:opacity-30"
                />
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    <h4 className="font-medium truncate max-w-[250px]">{mod.name}</h4>
                    <Badge variant="neutral">{mod.loader}</Badge>
                    <span className="text-xs text-white/40">v{mod.version}</span>
                  </div>
                  <p className="text-xs text-white/50 mt-0.5">
                    {srcFile ? `${srcFile.keys_count || "?"} strings en ${sourceLocale}` : `Sin archivo ${sourceLocale}`}
                  </p>
                </div>

                {hasTarget && (
                  <Badge variant="info">Ya tiene {targetLocale}</Badge>
                )}

                {isTranslating && (
                  <Badge variant="warning">
                    <Loader2 size={10} className="animate-spin" /> En curso
                  </Badge>
                )}

                {isCompleted && (
                  <Badge variant="success">
                    <CheckCircle2 size={10} /> Completado
                  </Badge>
                )}

                <button
                  onClick={() => handleTranslate(mod)}
                  disabled={isTranslating || isCompleted}
                  className="btn btn-primary text-sm disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isTranslating ? (
                    <>
                      <Loader2 size={12} className="animate-spin" />
                      Traduciendo...
                    </>
                  ) : isCompleted ? (
                    <>
                      <CheckCircle2 size={12} />
                      Listo
                    </>
                  ) : (
                    <>
                      <Play size={12} />
                      Traducir
                    </>
                  )}
                </button>
              </motion.div>
            );
          })}
        </div>
      </GlassCard>

      {translatableMods.length === 0 && (
        <GlassCard className="text-center py-12">
          <p className="text-white/60">Ningún mod tiene archivos en inglés para traducir.</p>
        </GlassCard>
      )}
    </div>
  );
}