import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import GlassCard from "../components/ui/GlassCard";
import Badge from "../components/ui/Badge";
import { Search, Edit3, CheckCircle2, Save, Loader2 } from "lucide-react";
import { useModStore } from "../stores/modStore";

interface Translation {
  id: number | null;
  mod_id: string;
  key: string;
  source_text: string;
  target_text: string;
  source_locale: string;
  target_locale: string;
  provider: string;
  confidence: number;
  is_manual_edit: boolean;
  category: string | null;
  created_at: string;
  updated_at: string;
}

export default function Editor() {
  const { mods } = useModStore();
  const [selectedMod, setSelectedMod] = useState<string>("");
  const [translations, setTranslations] = useState<Translation[]>([]);
  const [drafts, setDrafts] = useState<Record<number, string>>({});
  const [loading, setLoading] = useState(false);
  const [search, setSearch] = useState("");
  const [savingId, setSavingId] = useState<number | null>(null);
  const [savedMsg, setSavedMsg] = useState<string | null>(null);

  const load = async (modId: string) => {
    setLoading(true);
    setTranslations([]);
    setDrafts({});
    try {
      const data = await invoke<Translation[]>("get_translations_for_mod", { modId });
      setTranslations(data);
    } catch (e) {
      console.error("Error loading translations:", e);
    } finally {
      setLoading(false);
    }
  };

  const handleSelect = (modId: string) => {
    setSelectedMod(modId);
    load(modId);
  };

  const save = async (id: number) => {
    setSavingId(id);
    setSavedMsg(null);
    const current = translations.find((t) => t.id === id);
    if (!current) return;
    const target_text = drafts[id] ?? current.target_text;
    try {
      await invoke("update_translation", {
        entry: {
          ...current,
          target_text,
          is_manual_edit: true,
          updated_at: new Date().toISOString(),
        },
      });
      setTranslations((prev) =>
        prev.map((t) => (t.id === id ? { ...t, target_text, is_manual_edit: true } : t))
      );
      setSavedMsg("Cambio guardado.");
    } catch (e) {
      console.error("Error saving:", e);
      setSavedMsg(`Error al guardar: ${e}`);
    } finally {
      setSavingId(null);
    }
  };

  const filtered = translations.filter((t) =>
    t.key.toLowerCase().includes(search.toLowerCase()) ||
    t.source_text.toLowerCase().includes(search.toLowerCase()) ||
    t.target_text.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Editor</h1>
          <p className="text-white/50 mt-1">Revisa y edita traducciones guardadas</p>
        </div>
      </div>

      <GlassCard>
        <h3 className="font-semibold mb-3 flex items-center gap-2">
          <Edit3 size={18} className="text-accent" />
          Selecciona un mod
        </h3>
        {mods.length === 0 ? (
          <p className="text-white/50 text-sm">
            No hay mods escaneados. Escanea una carpeta en la pestaña Mods.
          </p>
        ) : (
          <div className="grid grid-cols-2 md:grid-cols-3 gap-2 max-h-48 overflow-y-auto">
            {mods.map((m) => (
              <button
                key={m.path}
                onClick={() => handleSelect(m.id)}
                className={`text-left px-3 py-2 rounded-lg border text-sm transition-colors ${
                  selectedMod === m.id
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

      {savedMsg && <p className="text-sm text-green-400">{savedMsg}</p>}

      {loading && (
        <GlassCard className="flex items-center justify-center py-12">
          <Loader2 size={24} className="animate-spin text-accent" />
        </GlassCard>
      )}

      {selectedMod && !loading && translations.length === 0 && (
        <GlassCard className="text-center py-12">
          <p className="text-white/50">Este mod aún no tiene traducciones guardadas.</p>
        </GlassCard>
      )}

      {translations.length > 0 && (
        <GlassCard>
          <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-white/5 border border-glass-border mb-4">
            <Search size={16} className="text-white/40" />
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Buscar por clave, texto origen o traducción..."
              className="bg-transparent outline-none text-sm flex-1"
            />
          </div>

          <div className="grid grid-cols-[1fr_1fr_1fr_auto] gap-4 text-xs text-white/40 uppercase tracking-wider pb-2 border-b border-glass-border">
            <div>Key</div>
            <div>Origen</div>
            <div>Traducción</div>
            <div></div>
          </div>

          <div className="divide-y divide-glass-border">
            {filtered.map((t) => {
              const id = t.id!;
              const isDirty = drafts[id] !== undefined && drafts[id] !== t.target_text;
              const saving = savingId === id;
              return (
                <div key={id} className="grid grid-cols-[1fr_1fr_1fr_auto] gap-4 py-3 items-center hover:bg-white/5 -mx-2 px-2 rounded transition-colors">
                  <div className="font-mono text-xs text-white/70 truncate" title={t.key}>{t.key}</div>
                  <div className="text-sm">{t.source_text}</div>
                  <div className="flex items-center gap-2">
                    <input
                      type="text"
                      value={drafts[id] ?? t.target_text}
                      onChange={(e) => setDrafts((d) => ({ ...d, [id]: e.target.value }))}
                      className="bg-white/5 border border-glass-border rounded px-2 py-1 text-sm w-full"
                    />
                    {isDirty && (
                      <button
                        onClick={() => save(id)}
                        disabled={saving}
                        className="btn btn-primary text-xs shrink-0"
                        title="Guardar"
                      >
                        {saving ? <Loader2 size={12} className="animate-spin" /> : <Save size={12} />}
                      </button>
                    )}
                  </div>
                  <div className="flex items-center justify-end gap-1">
                    {t.is_manual_edit ? (
                      <Badge variant="success"><Edit3 size={10} /> Manual</Badge>
                    ) : (
                      <Badge variant="info"><CheckCircle2 size={10} /> {(t.confidence * 100).toFixed(0)}%</Badge>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </GlassCard>
      )}
    </div>
  );
}
