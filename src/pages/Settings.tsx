import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import GlassCard from "../components/ui/GlassCard";
import Badge from "../components/ui/Badge";
import { Save, TestTube, Key, Zap, CheckCircle2, XCircle, Loader2 } from "lucide-react";

interface ProviderConfig {
  name: string;
  api_key?: string;
  model: string;
  base_url?: string;
  enabled: boolean;
  priority: number;
  temperature: number;
  max_tokens: number;
  batch_size: number;
  timeout_secs: number;
}

interface AppConfig {
  providers: ProviderConfig[];
  fallback_order: string[];
  source_locale: string;
  target_locale: string;
  workers: number;
  global_batch_size: number;
  max_retries: number;
  enable_backup: boolean;
  enable_validation: boolean;
  enable_cache: boolean;
  reject_suspicious: boolean;
}

type TestState = Record<string, "testing" | "success" | "error" | undefined>;

export default function Settings() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [testing, setTesting] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<TestState>({});
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState<string | null>(null);

  useEffect(() => {
    invoke<AppConfig>("get_config").then(setConfig).catch(console.error);
  }, []);

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    setSaved(null);
    try {
      await invoke("save_config", { cfg: config });
      setSaved("Configuración guardada correctamente.");
    } catch (e) {
      console.error("Error saving config:", e);
      setSaved(`Error al guardar: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const handleTest = async (provider: ProviderConfig) => {
    setTesting(provider.name);
    setTestResults((r) => ({ ...r, [provider.name]: "testing" }));
    try {
      const ok = await invoke<boolean>("test_provider", { name: provider.name, cfg: provider });
      setTestResults((r) => ({ ...r, [provider.name]: ok ? "success" : "error" }));
    } catch (e) {
      console.error("Test error:", e);
      setTestResults((r) => ({ ...r, [provider.name]: "error" }));
    }
    setTesting(null);
  };

  if (!config) return <div className="text-white/50">Loading...</div>;

  const updateProvider = (name: string, patch: Partial<ProviderConfig>) => {
    setConfig({
      ...config,
      providers: config.providers.map((p) =>
        p.name === name ? { ...p, ...patch } : p
      ),
    });
  };

  return (
    <div className="space-y-6 max-w-4xl">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Ajustes</h1>
          <p className="text-white/50 mt-1">Configura los proveedores de IA y las opciones de traducción</p>
        </div>
        <button onClick={handleSave} disabled={saving} className="btn btn-primary disabled:opacity-50">
          {saving ? <Loader2 size={14} className="animate-spin" /> : <Save size={14} />}
          {saving ? "Guardando..." : "Guardar"}
        </button>
      </div>

      {saved && (
        <div
          className={`flex items-center gap-2 text-sm px-3 py-2 rounded-lg ${
            saved.startsWith("Error")
              ? "bg-red-500/10 text-red-400 border border-red-500/20"
              : "bg-green-500/10 text-green-400 border border-green-500/20"
          }`}
        >
          {saved.startsWith("Error") ? <XCircle size={14} /> : <CheckCircle2 size={14} />}
          {saved}
        </div>
      )}

      <GlassCard>
        <h3 className="font-semibold mb-4 flex items-center gap-2">
          <Key size={18} /> Proveedores de IA
          <span className="text-xs font-normal text-white/40 ml-auto">
            Las claves se guardan cifradas por el sistema
          </span>
        </h3>
        <div className="space-y-3">
          {config.providers.map((p) => (
            <div key={p.name} className="p-4 rounded-lg bg-white/5 border border-glass-border">
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-3">
                  <h4 className="font-medium capitalize">{p.name}</h4>
                  <Badge variant={p.enabled ? "success" : "neutral"}>
                    {p.enabled ? "Activo" : "Inactivo"}
                  </Badge>
                  {testResults[p.name] && (
                    <Badge variant={testResults[p.name] === "success" ? "success" : "error"}>
                      {testResults[p.name] === "testing" ? (
                        <Loader2 size={10} className="animate-spin" />
                      ) : testResults[p.name] === "success" ? (
                        <CheckCircle2 size={10} />
                      ) : (
                        <XCircle size={10} />
                      )}
                      {testResults[p.name] === "success" ? "Conexión OK" : testResults[p.name] === "error" ? "Conexión fallida" : "Probando..."}
                    </Badge>
                  )}
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={() => handleTest(p)}
                    disabled={testing === p.name}
                    className="btn btn-ghost text-xs"
                  >
                    <TestTube size={12} />
                    {testing === p.name ? "Probando..." : "Probar"}
                  </button>
                  <button
                    onClick={() => updateProvider(p.name, { enabled: !p.enabled })}
                    className="btn btn-ghost text-xs"
                  >
                    {p.enabled ? "Desactivar" : "Activar"}
                  </button>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <label className="text-white/50 text-xs">API Key</label>
                  <input
                    type="password"
                    value={p.api_key || ""}
                    onChange={(e) => updateProvider(p.name, { api_key: e.target.value })}
                    placeholder="sk-..."
                    className="w-full mt-1 px-3 py-1.5 rounded bg-white/5 border border-glass-border outline-none text-sm"
                  />
                </div>
                <div>
                  <label className="text-white/50 text-xs">Modelo</label>
                  <input
                    type="text"
                    value={p.model}
                    onChange={(e) => updateProvider(p.name, { model: e.target.value })}
                    className="w-full mt-1 px-3 py-1.5 rounded bg-white/5 border border-glass-border outline-none text-sm"
                  />
                </div>
                {p.base_url !== undefined && (
                  <div className="col-span-2">
                    <label className="text-white/50 text-xs">Base URL (opcional)</label>
                    <input
                      type="text"
                      value={p.base_url || ""}
                      onChange={(e) => updateProvider(p.name, { base_url: e.target.value })}
                      placeholder="https://api.example.com/v1"
                      className="w-full mt-1 px-3 py-1.5 rounded bg-white/5 border border-glass-border outline-none text-sm"
                    />
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      </GlassCard>

      <GlassCard>
        <h3 className="font-semibold mb-4 flex items-center gap-2">
          <Zap size={18} /> Opciones de traducción
        </h3>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="text-white/50 text-xs">Idioma origen</label>
            <input
              type="text"
              value={config.source_locale}
              onChange={(e) => setConfig({ ...config, source_locale: e.target.value })}
              className="w-full mt-1 px-3 py-1.5 rounded bg-white/5 border border-glass-border outline-none text-sm"
            />
          </div>
          <div>
            <label className="text-white/50 text-xs">Idioma destino</label>
            <input
              type="text"
              value={config.target_locale}
              onChange={(e) => setConfig({ ...config, target_locale: e.target.value })}
              className="w-full mt-1 px-3 py-1.5 rounded bg-white/5 border border-glass-border outline-none text-sm"
            />
          </div>
          <div>
            <label className="text-white/50 text-xs">Workers</label>
            <input
              type="number"
              value={config.workers}
              onChange={(e) => setConfig({ ...config, workers: parseInt(e.target.value) || 0 })}
              className="w-full mt-1 px-3 py-1.5 rounded bg-white/5 border border-glass-border outline-none text-sm"
            />
          </div>
          <div>
            <label className="text-white/50 text-xs">Batch Size</label>
            <input
              type="number"
              value={config.global_batch_size}
              onChange={(e) => setConfig({ ...config, global_batch_size: parseInt(e.target.value) || 0 })}
              className="w-full mt-1 px-3 py-1.5 rounded bg-white/5 border border-glass-border outline-none text-sm"
            />
          </div>
          <div>
            <label className="text-white/50 text-xs">Reintentos máximos</label>
            <input
              type="number"
              value={config.max_retries}
              onChange={(e) => setConfig({ ...config, max_retries: parseInt(e.target.value) || 0 })}
              className="w-full mt-1 px-3 py-1.5 rounded bg-white/5 border border-glass-border outline-none text-sm"
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3 mt-4">
          {[
            { key: "enable_backup", label: "Copia de seguridad automática" },
            { key: "enable_validation", label: "Validar traducciones" },
            { key: "enable_cache", label: "Usar caché de traducción" },
            { key: "reject_suspicious", label: "Rechazar sospechosas" },
          ].map((opt) => (
            <label key={opt.key} className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={Boolean((config as any)[opt.key])}
                onChange={(e) => setConfig({ ...config, [opt.key]: e.target.checked } as any)}
                className="accent-accent"
              />
              <span className="text-sm">{opt.label}</span>
            </label>
          ))}
        </div>
      </GlassCard>
    </div>
  );
}
