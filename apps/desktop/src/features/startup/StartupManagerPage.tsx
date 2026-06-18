import { useEffect, useState } from "react";
import { useOptimize, type StartupItem } from "@/shared/hooks/useOptimize";
import { AxBadge, AxEmptyState, AxButton, AxModal, useAxToast } from "@/shared/apex";
import { invokeCmd } from "@/shared/lib/tauri";
import "./startup.css";

type Capability = { id: string; status: string; detail: string };

// ── UX-006: normalização de nomes técnicos → nomes amigáveis ─────────────────
const NAME_MAP: { match: RegExp; name: string }[] = [
  { match: /microsoftedge/i,            name: "Microsoft Edge" },
  { match: /rtkaud|rtkngui|rtkcpl|realtek/i, name: "Realtek Audio" },
  { match: /lghub|logitech ?g ?hub/i,   name: "Logitech G HUB" },
  { match: /icue|corsair/i,             name: "Corsair iCUE" },
  { match: /armoury|armourycrate/i,     name: "ASUS Armoury Crate" },
  { match: /razer|synapse/i,            name: "Razer Synapse" },
  { match: /geforce|nvidia/i,           name: "NVIDIA GeForce" },
  { match: /onedrive/i,                 name: "Microsoft OneDrive" },
  { match: /\bteams\b/i,                name: "Microsoft Teams" },
  { match: /discord/i,                  name: "Discord" },
  { match: /spotify/i,                  name: "Spotify" },
  { match: /steam/i,                    name: "Steam" },
  { match: /epicgames/i,                name: "Epic Games" },
  { match: /eadesktop|ea ?app|origin/i, name: "EA App" },
  { match: /battle\.?net/i,             name: "Battle.net" },
  { match: /riot/i,                     name: "Riot Client" },
  { match: /ubisoft/i,                  name: "Ubisoft Connect" },
  { match: /gog ?galaxy/i,              name: "GOG Galaxy" },
  { match: /rockstar/i,                 name: "Rockstar Launcher" },
  { match: /overwolf/i,                 name: "Overwolf" },
  { match: /curseforge/i,               name: "CurseForge" },
  { match: /dropbox/i,                  name: "Dropbox" },
  { match: /adobe/i,                    name: "Adobe Creative Cloud" },
  { match: /msi ?center/i,              name: "MSI Center" },
];

function displayName(item: StartupItem): string {
  const hay = `${item.name} ${item.command}`;
  for (const n of NAME_MAP) if (n.match.test(hay)) return n.name;
  // Fallback: remove sufixos tipo "_C46CFC0629..." e separadores técnicos.
  const cleaned = item.name.replace(/_[0-9a-fA-F]{6,}.*$/, "").replace(/[_]+/g, " ").trim();
  return cleaned.length >= 3 ? cleaned : item.name;
}

// ── Inferência de impacto ────────────────────────────────────────────────────

type Impact = "alto" | "medio" | "baixo";

const HEAVY_APPS = [
  "discord", "spotify", "steam", "origin", "epicgameslauncher", "epicgames",
  "teams", "skype", "zoom", "slack", "onedrive", "dropbox", "googledrive",
  "googledrivefs", "ea app", "eadesktop", "battlenet", "battle.net",
  "ubisoft", "ubisoftconnect", "riot", "riotclient", "rockstar", "rockstarlauncher",
  "xbox", "gamepass", "geforceexperience", "nvidia", "amd",
  "overwolf", "curseforge", "adobe", "adobeupdater", "adobegenuineclient",
  "gog", "gogalaxy", "gog galaxy",
];
const MEDIUM_APPS = [
  "dropbox", "onedrive", "googledrive", "ccleaner", "malwarebytes",
  "avast", "avg", "kaspersky", "bitdefender", "mcafee", "norton",
  "razer", "corsair", "logitech", "asus", "msi",
];

// Descrições curtas para apps conhecidos, exibidas no modal e na linha de item.
const APP_DESCRIPTIONS: Record<string, string> = {
  discord:            "Chat de voz e texto para jogos. Consome ~150 MB de RAM no boot.",
  spotify:            "Player de música. Carrega o Chromium integrado durante o boot.",
  steam:              "Plataforma de jogos da Valve. Atualiza jogos automaticamente no boot.",
  epicgames:          "Launcher da Epic Games. Verifica atualizações e carrega no boot.",
  epicgameslauncher:  "Launcher da Epic Games. Verifica atualizações e carrega no boot.",
  origin:             "Launcher da EA (legado). Substituído pelo EA App.",
  eadesktop:          "EA App (substituto do Origin). Carrega serviços de DRM no boot.",
  "ea app":           "EA App (substituto do Origin). Carrega serviços de DRM no boot.",
  battlenet:          "Launcher da Blizzard (Battle.net). Carrega agente de atualização no boot.",
  "battle.net":       "Launcher da Blizzard (Battle.net). Carrega agente de atualização no boot.",
  ubisoft:            "Ubisoft Connect. Carrega serviço de DRM e overlay no boot.",
  ubisoftconnect:     "Ubisoft Connect. Carrega serviço de DRM e overlay no boot.",
  riot:               "Riot Games Client. Necessário para Valorant/LoL — pode ser reiniciado ao abrir o jogo.",
  riotclient:         "Riot Games Client. Necessário para Valorant/LoL — pode ser reiniciado ao abrir o jogo.",
  rockstar:           "Rockstar Games Launcher. Necessário para GTA V/RDR2 no PC.",
  rockstarlauncher:   "Rockstar Games Launcher. Necessário para GTA V/RDR2 no PC.",
  gog:                "GOG Galaxy. Plataforma de jogos DRM-free. Carrega serviços de sync no boot.",
  gogalaxy:           "GOG Galaxy. Plataforma de jogos DRM-free. Carrega serviços de sync no boot.",
  "gog galaxy":       "GOG Galaxy. Plataforma de jogos DRM-free. Carrega serviços de sync no boot.",
  overwolf:           "Overlay de jogos e app store. Injeta hooks em jogos — alto impacto em CPU/RAM.",
  curseforge:         "Gerenciador de mods de jogos. Carrega serviço de sync de mods no boot.",
  adobe:              "Serviços Adobe (Creative Cloud, Updater). Carrega múltiplos processos no boot.",
  adobeupdater:       "Adobe Updater. Verifica atualizações de produtos Adobe no boot.",
  adobegenuineclient: "Adobe Genuine Client. Serviço de verificação de licença Adobe.",
  teams:              "Microsoft Teams. Carrega cliente Electron completo no boot.",
  onedrive:           "Microsoft OneDrive. Sync de arquivos em nuvem — alto uso de disco no boot.",
  geforceexperience:  "NVIDIA GeForce Experience. Carrega overlay e serviços de driver no boot.",
};

function getAppDescription(item: StartupItem): string | null {
  const lower = item.name.toLowerCase() + " " + item.command.toLowerCase();
  for (const [key, desc] of Object.entries(APP_DESCRIPTIONS)) {
    if (lower.includes(key)) return desc;
  }
  return null;
}

// UX-007: descrição sempre presente — fallback humano por nível de impacto.
function describe(item: StartupItem): string {
  const d = getAppDescription(item);
  if (d) return d;
  const imp = inferImpact(item);
  if (imp === "alto")  return "Inicia com o Windows e consome recursos significativos durante o boot.";
  if (imp === "medio") return "App de segundo plano que inicia junto com o Windows.";
  return "Componente leve que inicia com o Windows.";
}

function inferImpact(item: StartupItem): Impact {
  const lower = item.name.toLowerCase() + " " + item.command.toLowerCase();
  if (HEAVY_APPS.some((k) => lower.includes(k))) return "alto";
  if (MEDIUM_APPS.some((k) => lower.includes(k))) return "medio";
  return "baixo";
}

// Memória RAM aproximada liberada ao desabilitar o app no boot (MB).
const APP_MEMORY_MB: Record<string, number> = {
  discord:            150,
  spotify:            200,
  steam:              200,
  epicgames:          150,
  epicgameslauncher:  150,
  origin:             120,
  eadesktop:          130,
  "ea app":           130,
  battlenet:          100,
  "battle.net":       100,
  ubisoft:            100,
  ubisoftconnect:     100,
  riot:               100,
  riotclient:         100,
  rockstar:           80,
  gog:                80,
  gogalaxy:           80,
  overwolf:           180,
  curseforge:         120,
  adobe:              200,
  adobeupdater:       60,
  teams:              350,
  onedrive:           80,
  geforceexperience:  120,
  dropbox:            80,
  slack:              300,
  skype:              150,
  zoom:               200,
};

function estimatedMemoryMb(item: StartupItem): number | null {
  const lower = item.name.toLowerCase() + " " + item.command.toLowerCase();
  for (const [key, mb] of Object.entries(APP_MEMORY_MB)) {
    if (lower.includes(key)) return mb;
  }
  return null;
}

function estimatedSaving(impact: Impact, location: string): string {
  if (location === "HKLM") return "—";
  if (impact === "alto")  return "2–4 s";
  if (impact === "medio") return "1–2 s";
  return "0.5–1 s";
}

const IMPACT_LABEL: Record<Impact, string> = { alto: "Alto", medio: "Médio", baixo: "Baixo" };
const IMPACT_VARIANT = { alto: "risk", medio: "warn", baixo: "ok" } as const;

// ── Componente de item ───────────────────────────────────────────────────────

function StartupRow({
  item,
  onRequestDisable,
  disabled: alreadyDisabled,
  isElevated,
}: {
  item: StartupItem;
  onRequestDisable: (item: StartupItem) => void;
  disabled: boolean;
  isElevated: boolean;
}) {
  const impact     = inferImpact(item);
  const saving     = estimatedSaving(impact, item.location);
  const canDisable = item.location === "HKCU";
  const desc       = describe(item);
  const memMb      = estimatedMemoryMb(item);
  const name       = displayName(item);

  return (
    <div className={`su2-row ${alreadyDisabled ? "su2-row-dim" : ""}`}>
      <div className="su2-info">
        <strong className="su2-name">{name}</strong>
        <p className="su2-desc">{desc}</p>
        <div className="su2-meta">
          <AxBadge variant={IMPACT_VARIANT[impact]}>{IMPACT_LABEL[impact]} impacto</AxBadge>
          <span className="su2-loc">{item.location === "HKCU" ? "Usuário" : "Sistema"}</span>
        </div>
      </div>

      <div className="su2-saving">
        <span className="su2-saving-label">Impacto estimado</span>
        {memMb !== null && <strong className="su2-saving-val">≈ {memMb} MB RAM</strong>}
        {saving !== "—" && <span className="su2-saving-mem">+{saving} no boot</span>}
        {memMb === null && saving === "—" && <strong className="su2-saving-val">leve</strong>}
      </div>

      <div className="su2-action">
        {canDisable ? (
          <AxButton
            size="sm"
            variant="ghost"
            onClick={() => !alreadyDisabled && onRequestDisable(item)}
            disabled={alreadyDisabled}
          >
            {alreadyDisabled ? "Desabilitado" : "Desabilitar"}
          </AxButton>
        ) : (
          // UX-005: com o processo elevado, NÃO mostrar "Requer admin".
          <span className="su2-admin">{isElevated ? "Item do sistema" : "Requer administrador"}</span>
        )}
      </div>
    </div>
  );
}

// ── Página ───────────────────────────────────────────────────────────────────

export function StartupManagerPage() {
  const { available, startupAnalysis, disableStartup } = useOptimize();
  const toast = useAxToast();

  const [items, setItems]                 = useState<StartupItem[] | null>(null);
  const [loading, setLoading]             = useState(false);
  const [pendingItem, setPendingItem]     = useState<StartupItem | null>(null);
  const [confirming, setConfirming]       = useState(false);
  const [disabled, setDisabled]           = useState<Set<string>>(new Set());
  const [isElevated, setIsElevated]       = useState(false);

  async function load() {
    if (!available) return;
    setLoading(true);
    const result = await startupAnalysis();
    setLoading(false);
    if (result) setItems(result);
  }

  useEffect(() => { load(); }, [available]); // eslint-disable-line react-hooks/exhaustive-deps

  // UX-005: detecta elevação real para não mostrar "Requer admin" indevidamente.
  useEffect(() => {
    if (!available) return;
    invokeCmd<Capability[]>("system_capabilities")
      .then((caps) => setIsElevated(caps.find((c) => c.id === "admin_privileges")?.status === "ready"))
      .catch(() => {});
  }, [available]);

  async function onConfirmDisable() {
    if (!pendingItem) return;
    setConfirming(true);
    const snapId = await disableStartup(pendingItem.name);
    setConfirming(false);
    if (snapId !== null) {
      setDisabled((prev) => new Set([...prev, pendingItem.name]));
      toast("ok", `"${pendingItem.name}" desabilitado · snapshot #${snapId} criado (reversível em Restauração).`);
    } else {
      toast("danger", `Não foi possível desabilitar "${pendingItem.name}".`);
    }
    setPendingItem(null);
  }

  // Agrupar por impacto
  const high   = (items ?? []).filter((i) => inferImpact(i) === "alto");
  const medium = (items ?? []).filter((i) => inferImpact(i) === "medio");
  const low    = (items ?? []).filter((i) => inferImpact(i) === "baixo");

  const totalSavingHigh = high.filter((i) => i.location === "HKCU").length * 3;

  return (
    <div className="startup2">
      <header className="su2-head">
        <div>
          <h1>Gerenciador de Inicialização</h1>
          <p>Apps que iniciam com o Windows.{items !== null && totalSavingHigh > 0 ? ` Desabilitar os de alto impacto pode economizar até ${totalSavingHigh} segundos no boot.` : " Desabilitar apps desnecessários acelera o boot do Windows."}</p>
        </div>
        <AxButton size="sm" icon="refresh" variant="ghost" onClick={load} disabled={loading || !available}>
          {loading ? "Carregando…" : "Atualizar"}
        </AxButton>
      </header>

      {!available && (
        <div className="su2-banner">
          Abra o aplicativo TkSpeed para controlar os apps de inicialização.
        </div>
      )}

      {available && items === null && loading && (
        <div className="su2-loading">Lendo apps de inicialização…</div>
      )}

      {items !== null && items.length === 0 && (
        <AxEmptyState
          icon="startup"
          title="Nenhum app de inicialização encontrado"
          description="Nenhum app de inicialização automática encontrado neste sistema."
        />
      )}

      {items !== null && items.length > 0 && (
        <div className="su2-content">
          {/* Sumário */}
          <div className="su2-stats">
            <div className="su2-stat">
              <strong>{items.length}</strong>
              <span>apps no boot</span>
            </div>
            <div className="su2-stat su2-stat-risk">
              <strong>{high.length}</strong>
              <span>alto impacto</span>
            </div>
            <div className="su2-stat su2-stat-ok">
              <strong>{high.filter((i) => i.location === "HKCU").length}</strong>
              <span>podem ser desabilitados</span>
            </div>
            <div className="su2-stat su2-stat-time">
              <strong>{totalSavingHigh > 0 ? `~${totalSavingHigh}s` : "—"}</strong>
              <span>economia potencial</span>
            </div>
          </div>

          {/* Alto impacto */}
          {high.length > 0 && (
            <section className="su2-section">
              <div className="su2-section-hd">
                <span>Alto impacto</span>
                <AxBadge variant="risk">{high.length}</AxBadge>
              </div>
              <div className="su2-list">
                {high.map((item) => (
                  <StartupRow
                    key={item.name}
                    item={item}
                    onRequestDisable={setPendingItem}
                    disabled={disabled.has(item.name)}
                    isElevated={isElevated}
                  />
                ))}
              </div>
            </section>
          )}

          {/* Médio impacto */}
          {medium.length > 0 && (
            <section className="su2-section">
              <div className="su2-section-hd">
                <span>Médio impacto</span>
                <AxBadge variant="warn">{medium.length}</AxBadge>
              </div>
              <div className="su2-list">
                {medium.map((item) => (
                  <StartupRow
                    key={item.name}
                    item={item}
                    onRequestDisable={setPendingItem}
                    disabled={disabled.has(item.name)}
                    isElevated={isElevated}
                  />
                ))}
              </div>
            </section>
          )}

          {/* Baixo impacto */}
          {low.length > 0 && (
            <section className="su2-section">
              <div className="su2-section-hd">
                <span>Baixo impacto</span>
                <AxBadge variant="ok">{low.length}</AxBadge>
              </div>
              <div className="su2-list">
                {low.map((item) => (
                  <StartupRow
                    key={item.name}
                    item={item}
                    onRequestDisable={setPendingItem}
                    disabled={disabled.has(item.name)}
                    isElevated={isElevated}
                  />
                ))}
              </div>
            </section>
          )}
        </div>
      )}

      {/* Modal de confirmação */}
      <AxModal
        open={!!pendingItem}
        title="Desabilitar app de inicialização?"
        onClose={() => confirming ? undefined : setPendingItem(null)}
        footer={
          <>
            <button className="ax-btn ax-btn-ghost" onClick={() => setPendingItem(null)} disabled={confirming}>
              Cancelar
            </button>
            <button className="ax-btn ax-btn-primary" onClick={onConfirmDisable} disabled={confirming}>
              {confirming ? "Desabilitando…" : "Desabilitar"}
            </button>
          </>
        }
      >
        {pendingItem && (
          <>
            <p><strong>{displayName(pendingItem)}</strong> deixará de iniciar com o Windows.</p>
            <p style={{ marginTop: 8, color: "var(--ink-mid)", fontSize: 13 }}>
              {describe(pendingItem)}
            </p>
            <p style={{ marginTop: 10, color: "var(--ink-mid)", fontSize: 13 }}>
              Um snapshot de restauração será criado automaticamente. Você pode reverter a qualquer momento pela Central de Restauração.
            </p>
          </>
        )}
      </AxModal>
    </div>
  );
}
