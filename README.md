# Solano Mod Translator

> **Español** | [**English**](#english)

---

## Español

Aplicación de escritorio para traducir automáticamente mods de Minecraft usando IA.

### Que es

Solano Mod Translator escanea mods `.jar` (Forge/Fabric/NeoForge), lee sus archivos
de idioma y traduce todas las cadenas de texto al idioma que quieras mediante
proveedores de IA. Genera un nuevo `.jar` con el idioma traducido añadido o reemplazado,
sin tocar el mod original.

### Tecnologias que usa

| Capa | Tecnologia |
|------|-----------|
| Frontend | React 18, TypeScript, Vite, Tailwind CSS, framer-motion, lucide-react, react-router, zustand |
| Backend | Rust + Tauri 2 (plugins: shell, dialog, fs, store, notification, process, sql) |
| Base de datos | SQLite (`rusqlite`) para cache, historial y glosario |
| Seguridad | `keyring` — las API keys se guardan en el Almacen de credenciales de Windows |
| Escritura de mods | `zip` (lectura/reescritura de archivos jar) |

### Como funciona

1. **Escaneo**: se selecciona una carpeta con mods y se inspeccionan los `.jar`.
2. **Lectura**: se abre el jar como un archivo zip y se detectan los archivos de idioma:
   - Formato actual: `assets/<modid>/lang/<locale>.json`
   - Formato antiguo: `lang/<locale>.lang`
   Se parsea cada archivo a un mapa `clave -> texto`.
3. **Traduccion por lotes**: las cadenas sin traducir se agrupan en lotes
   (_batch size_), se protegen las variables/placeholders (ej. `%s`, `{value}`, `<>`),
   se carga el glosario aplicable y se envian al proveedor de IA.
4. **Proveedores**: se recorre el orden de fallback configurado hasta que uno
   responde. Soporta **opencode** (local, via `opencode serve` en `127.0.0.1:4096`),
   **Gemini**, **OpenAI**, **Grok**, **Claude**, **DeepSeek** (API compatible con
   OpenAI) y **Ollama** (local). Los lotes fallidos se reintentan con backoff exponencial.
5. **Validacion**: cada cadena traducida se valida (porcentajes, placeholders,
   longitud) y se restaura el caso de las variables protegidas. Si la traduccion es
   sospechosa, puede rechazarse.
6. **Persistencia**: los pares traducidos se guardan en la cache SQLite para no
   re-pagar IA en futuros mods; tambien queda registrado el historial.
7. **Salida**: se escribe un nuevo `.jar` copiando el original y reemplazando o
   añadiendo el archivo de idioma traducido. Si esta activado, antes se crea un
   backup del mod original.

Las traducciones se pueden revisar y corregir a mano en el **Editor**, y el
**Diagnostico** puede analizar y reparar mods.

### Paginas de la app

- **Dashboard**: resumen general.
- **Mods**: escaneo de carpeta y detalles de cada mod.
- **Translator**: inicio de traducciones con barra de progreso en tiempo real.
- **Editor**: edicion manual del glosario de traducciones.
- **Stats** / **History**: estadisticas e historial de trabajos.
- **Diagnostics**: diagnostico y reparacion de mods.
- **Settings**: configuracion de idiomas, proveedores de IA, backoff, cache, etc.

### Estructura del proyecto

```
├── src/                  # Frontend React (paginas, componentes, stores)
├── src-tauri/
│   └── src/
│       ├── main.rs       # Arranque, plugins, comandos Tauri
│       ├── commands.rs   # Comandos expuestos al frontend
│       ├── translator.rs # Motor de traduccion por lotes
│       ├── providers/    # Adaptadores por proveedor de IA
│       ├── core/         # scanner, jar_reader, lang_parser, validator, backup...
│       ├── cache.rs      # SQLite (cache, glosario, historial)
│       ├── config.rs     # Configuracion + API keys (keyring)
│       └── glossary.rs   # Glosario aplicable durante la traduccion
```

### Requisitos y arranque en desarrollo

- Node.js + npm
- Rust (toolchain stable) y [Tauri prerequisites](https://tauri.app/start/prerequisites/)
- Opcional: `opencode` instalado para usar el proveedor local

```bash
npm install
npm run tauri dev
```

Para construir el instalador:

```bash
npm run tauri build
```

### Configuracion inicial

1. Ve a **Settings**.
2. Elige el proveedor de IA (o deja **opencode** local) y pega tu API key.
   - Gemini: https://aistudio.google.com/apikey
   - OpenAI: https://platform.openai.com/api-keys
3. Pulsa **Guardar** y **Probar conexion**.
4. Selecciona el idioma de origen/target y traduce.

Las API keys se guardan en el Almacen de credenciales de Windows, nunca en disco.

### Notas

- Los idiomas usan codigos estandar de Minecraft, por defecto `en_us` -> `es_es`.
- El proveedor local **opencode** se intenta lanzar en segundo plano al arrancar
  (`opencode serve` en `http://127.0.0.1:4096`) si no esta corriendo.
- Para el usuario final existe `LEEME.txt` con las instrucciones de instalacion
  del paquete `SolanoModTranslator_PaqueteInstalacion.zip`.

---

## English

Desktop application to automatically translate Minecraft mods using AI.

### What it is

Solano Mod Translator scans `.jar` mods (Forge/Fabric/NeoForge), reads their language
files and translates every text string to the language you want using AI providers.
It outputs a new `.jar` with the translated language added or replaced, without
touching the original mod.

### Technologies used

| Layer | Technology |
|-------|------------|
| Frontend | React 18, TypeScript, Vite, Tailwind CSS, framer-motion, lucide-react, react-router, zustand |
| Backend | Rust + Tauri 2 (plugins: shell, dialog, fs, store, notification, process, sql) |
| Database | SQLite (`rusqlite`) for cache, history and glossary |
| Security | `keyring` — API keys are stored in the Windows Credential Manager |
| Mod writing | `zip` (reading/rewriting jar files) |

### How it works

1. **Scanning**: you pick a folder with mods and the `.jar` files are inspected.
2. **Reading**: each jar is opened as a zip archive and language files are detected:
   - Modern format: `assets/<modid>/lang/<locale>.json`
   - Legacy format: `lang/<locale>.lang`
   Every file is parsed into a `key -> text` map.
3. **Batch translation**: untranslated strings are grouped into batches
   (_batch size_), variables/placeholders are protected (e.g. `%s`, `{value}`, `<>`),
   the applicable glossary is loaded and the batch is sent to the AI provider.
4. **Providers**: the configured fallback order is tried until one succeeds.
   Supports **opencode** (local, via `opencode serve` on `127.0.0.1:4096`),
   **Gemini**, **OpenAI**, **Grok**, **Claude**, **DeepSeek** (OpenAI-compatible
   API) and **Ollama** (local). Failed batches are retried with exponential backoff.
5. **Validation**: every translated string is validated (placeholders, lengths, ...)
   and protected variables are restored. Suspicious translations can be rejected.
6. **Persistence**: translated pairs are stored in the SQLite cache so you don't
   pay AI again for future mods; the job is also recorded in the history.
7. **Output**: a new `.jar` is written by copying the original and replacing or
   adding the translated language file. If enabled, a backup of the mod is created
   beforehand.

Translations can be reviewed and edited manually in the **Editor**, and
**Diagnostics** can analyze and repair mods.

### App pages

- **Dashboard**: general overview.
- **Mods**: folder scanning and per-mod details.
- **Translator**: start translations with a real-time progress bar.
- **Editor**: manual editing of the translation glossary.
- **Stats** / **History**: stats and job history.
- **Diagnostics**: mod diagnostics and repair.
- **Settings**: language, AI providers, backoff, cache, etc.

### Project structure

```
├── src/                  # React frontend (pages, components, stores)
├── src-tauri/
│   └── src/
│       ├── main.rs       # Startup, plugins, Tauri commands
│       ├── commands.rs   # Commands exposed to the frontend
│       ├── translator.rs # Batch translation engine
│       ├── providers/    # AI provider adapters
│       ├── core/         # scanner, jar_reader, lang_parser, validator, backup...
│       ├── cache.rs      # SQLite (cache, glossary, history)
│       ├── config.rs     # Config + API keys (keyring)
│       └── glossary.rs   # Glossary applied during translation
```

### Requirements and development setup

- Node.js + npm
- Rust (stable toolchain) and [Tauri prerequisites](https://tauri.app/start/prerequisites/)
- Optional: `opencode` installed to use the local provider

```bash
npm install
npm run tauri dev
```

To build the installer:

```bash
npm run tauri build
```

### Initial setup

1. Go to **Settings**.
2. Pick the AI provider (or keep local **opencode**) and paste your API key.
   - Gemini: https://aistudio.google.com/apikey
   - OpenAI: https://platform.openai.com/api-keys
3. Click **Save** and **Test connection**.
4. Select source/target language and translate.

API keys are stored in the Windows Credential Manager, never on disk.

### Notes

- Languages use standard Minecraft locale codes, `en_us` -> `es_es` by default.
- The local **opencode** provider is launched in the background at startup
  (`opencode serve` on `http://127.0.0.1:4096`) if it isn't already running.
- End users should check `LEEME.txt` with the install instructions of the
  `SolanoModTranslator_PaqueteInstalacion.zip` package.