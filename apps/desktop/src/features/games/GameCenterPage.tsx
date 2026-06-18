import { useCallback, useEffect, useRef, useState } from "react";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";
import { AxBadge, AxButton, AxEmptyState, AxModal, useAxToast } from "@/shared/apex";
import "./games.css";

// ── Tipos ─────────────────────────────────────────────────────────────────────

interface InstalledGame {
  id: string;
  name: string;
  exe: string;
  launcher: string;
  install_path: string;
}

interface RunningGame {
  pid: number;
  name: string;
  exe: string;
}

interface FpsMetric { metric: string; value: number; unit: string; }
interface FpsSession { ts: number; metrics: FpsMetric[]; }

function fpsVal(s: FpsSession, key: string): number {
  return s.metrics.find((m) => m.metric === key)?.value ?? 0;
}

function useLatestFps(): FpsSession | null {
  const [fps, setFps] = useState<FpsSession | null>(null);
  useEffect(() => {
    if (!isTauri()) return;
    let alive = true;
    async function poll() {
      try {
        const s = await invokeCmd<FpsSession | null>("perf_latest_fps_session");
        if (alive && s) setFps(s);
      } catch { /* silencioso */ }
    }
    poll();
    const id = setInterval(poll, 20_000);
    return () => { alive = false; clearInterval(id); };
  }, []);
  return fps;
}

interface GameAssignment {
  db_id: number;
  exe_match: string;
  game_name: string;
  profile_id: string;
}

interface GameRun {
  id: number;
  ts: number;
  exe_match: string;
  game_name: string;
  profile_id: string;
  source?: string;
}

interface BoostResult {
  pid: number;
  success: boolean;
  message: string;
}

interface GpuInfo { name: string; vendor: string; }

// ── Constantes ────────────────────────────────────────────────────────────────

const PROFILES = [
  { id: "",            label: "Sem perfil" },
  { id: "competitive", label: "Competitivo" },
  { id: "balanced",    label: "Balanceado" },
  { id: "streaming",   label: "Streaming" },
  { id: "power_saver", label: "Economia" },
] as const;

// Metadados visuais dos perfis (descrição + cor do badge + ícone).
const PROFILE_META: Record<string, { label: string; desc: string; icon: string; variant: "ion" | "signal" | "warn" | "ok" }> = {
  competitive: { label: "Competitivo", icon: "🎯", variant: "ion",
    desc: "Máxima resposta e FPS. Prioriza o jogo e reduz interferências em segundo plano." },
  balanced:    { label: "Balanceado",  icon: "⚖️", variant: "signal",
    desc: "Equilíbrio entre desempenho e estabilidade para o dia a dia." },
  streaming:   { label: "Streaming",   icon: "📡", variant: "warn",
    desc: "Reserva recursos para captura e transmissão sem travar o jogo." },
  power_saver: { label: "Economia",    icon: "🔋", variant: "ok",
    desc: "Reduz consumo e calor — ideal para notebooks na bateria." },
};

const LAUNCHER_LABEL: Record<string, string> = {
  steam:     "Steam",
  epic:      "Epic Games",
  riot:      "Riot Games",
  battlenet: "Battle.net",
  ea:        "EA App",
  ubisoft:   "Ubisoft Connect",
  gog:       "GOG Galaxy",
  rockstar:  "Rockstar",
};

const LAUNCHER_COLOR: Record<string, "ok" | "signal" | "neutral" | "warn"> = {
  steam:     "ok",
  epic:      "neutral",
  riot:      "warn",
  battlenet: "signal",
  ea:        "neutral",
  ubisoft:   "neutral",
  gog:       "neutral",
  rockstar:  "neutral",
};

function profileLabel(id: string): string {
  return PROFILES.find((p) => p.id === id)?.label ?? id;
}

function relTime(ts: number): string {
  const s = Math.max(0, Math.floor((Date.now() - ts) / 1000));
  if (s < 60) return "agora";
  const m = Math.floor(s / 60);
  if (m < 60) return `há ${m} min`;
  const h = Math.floor(m / 60);
  if (h < 24) return `há ${h}h`;
  return `há ${Math.floor(h / 24)} dias`;
}

// ── Hook principal ────────────────────────────────────────────────────────────

function useGameCenter() {
  const toast = useAxToast();
  const [installed, setInstalled]     = useState<InstalledGame[] | null>(null);
  const [running, setRunning]         = useState<RunningGame[]>([]);
  const [assignments, setAssignments] = useState<GameAssignment[]>([]);
  const [loadingInstall, setLoadingInstall] = useState(false);
  const [loadingRunning, setLoadingRunning] = useState(false);
  const [savingExe, setSavingExe]     = useState<string | null>(null);
  const [applyingExe, setApplyingExe] = useState<string | null>(null);
  const [boostingPid, setBoostingPid] = useState<number | null>(null);
  const autoApplied = useRef(new Set<number>());

  const available = isTauri();

  const loadAssignments = useCallback(async () => {
    if (!available) return;
    const data = await invokeCmd<GameAssignment[]>("game_assignments_list", {}).catch(() => []);
    setAssignments(data ?? []);
  }, [available]);

  const scanInstalled = useCallback(async () => {
    if (!available) return;
    setLoadingInstall(true);
    try {
      const games = await invokeCmd<InstalledGame[]>("detect_installed_games", {});
      setInstalled(games ?? []);
    } catch {
      setInstalled([]);
    } finally {
      setLoadingInstall(false);
    }
  }, [available]);

  const pollRunning = useCallback(async () => {
    if (!available) return;
    setLoadingRunning(true);
    try {
      const games = await invokeCmd<RunningGame[]>("detect_games", {});
      setRunning(games ?? []);
    } catch {
      setRunning([]);
    } finally {
      setLoadingRunning(false);
    }
  }, [available]);

  // Auto-apply: quando jogo detectado tem atribuição e ainda não foi aplicado nesta sessão
  useEffect(() => {
    if (!available || running.length === 0 || assignments.length === 0) return;
    for (const g of running) {
      if (autoApplied.current.has(g.pid)) continue;
      const exeLower = g.exe.replace(/\\/g, "/").split("/").pop()?.replace(/\.exe$/i, "").toLowerCase() ?? "";
      const assignment = assignments.find((a) =>
        exeLower.includes(a.exe_match.toLowerCase()) || a.exe_match.toLowerCase().includes(exeLower)
      );
      if (!assignment || !assignment.profile_id) continue;
      autoApplied.current.add(g.pid);
      toast("signal", `${g.name} detectado — aplicando perfil ${profileLabel(assignment.profile_id)}…`);
      invokeCmd("advisor_apply_profile", { profileId: assignment.profile_id })
        .then(() => {
          toast("ok", `Perfil ${profileLabel(assignment.profile_id)} aplicado para ${g.name}.`);
          invokeCmd("game_run_record", {
            exeMatch: assignment.exe_match,
            gameName: assignment.game_name,
            profileId: assignment.profile_id,
            source: "auto",
          }).catch(() => {});
        })
        .catch(() => {
          toast("danger", `Falha ao aplicar perfil para ${g.name}.`);
        });
    }
  }, [running, assignments, available, toast]);

  useEffect(() => {
    if (!available) return;
    loadAssignments();
    scanInstalled();
    pollRunning();
    const interval = setInterval(pollRunning, 10_000);
    return () => clearInterval(interval);
  }, [available, loadAssignments, scanInstalled, pollRunning]);

  async function saveAssignment(exe: string, gameName: string, profileId: string) {
    if (!available) return;
    setSavingExe(exe);
    try {
      if (profileId === "") {
        await invokeCmd("game_assignment_delete", { exeMatch: exe });
      } else {
        await invokeCmd("game_assignment_save", { exeMatch: exe, gameName, profileId });
      }
      await loadAssignments();
      toast("ok", profileId ? `${gameName} → perfil ${profileLabel(profileId)} salvo.` : `Atribuição de ${gameName} removida.`);
    } catch {
      toast("danger", "Falha ao salvar atribuição.");
    } finally {
      setSavingExe(null);
    }
  }

  async function applyProfile(exe: string, gameName: string, profileId: string) {
    if (!available || !profileId) return;
    setApplyingExe(exe);
    try {
      await invokeCmd("advisor_apply_profile", { profileId });
      await invokeCmd("game_run_record", { exeMatch: exe, gameName, profileId, source: "manual" });
      toast("ok", `Perfil ${profileLabel(profileId)} aplicado para ${gameName}.`);
    } catch {
      toast("danger", `Falha ao aplicar perfil.`);
    } finally {
      setApplyingExe(null);
    }
  }

  async function boostGame(pid: number, _name: string) {
    if (!available) return;
    setBoostingPid(pid);
    try {
      const r = await invokeCmd<BoostResult>("boost_game", { pid });
      if (r.success) toast("ok", r.message);
      else toast("danger", r.message);
    } finally {
      setBoostingPid(null);
    }
  }

  async function restoreGame(pid: number) {
    if (!available) return;
    setBoostingPid(pid);
    try {
      const r = await invokeCmd<BoostResult>("restore_game_priority", { pid });
      if (r.success) toast("ok", r.message);
      else toast("danger", r.message);
    } finally {
      setBoostingPid(null);
    }
  }

  function getAssignment(exe: string): GameAssignment | undefined {
    const lower = exe.toLowerCase();
    return assignments.find((a) =>
      lower.includes(a.exe_match.toLowerCase()) || a.exe_match.toLowerCase().includes(lower)
    );
  }

  return {
    available, installed, running, assignments,
    loadingInstall, loadingRunning, savingExe, applyingExe, boostingPid,
    scanInstalled, pollRunning,
    saveAssignment, applyProfile, boostGame, restoreGame, getAssignment,
  };
}

// ── Componente de perfil (selector inline) ────────────────────────────────────

function ProfileSelector({
  current, saving, onChange,
}: {
  exe?: string;
  gameName?: string;
  current: string;
  saving: boolean;
  onChange: (profileId: string) => void;
}) {
  return (
    <select
      className="gc-profile-select"
      value={current}
      disabled={saving}
      onChange={(e) => onChange(e.target.value)}
    >
      {PROFILES.map((p) => (
        <option key={p.id} value={p.id}>{p.label}</option>
      ))}
    </select>
  );
}

// ── Seção: Jogos Instalados ───────────────────────────────────────────────────

function InstalledSection({
  games, assignments, savingExe, applyingExe, running,
  onSave, onApply, onHistory,
}: {
  games: InstalledGame[];
  assignments: GameAssignment[];
  savingExe: string | null;
  applyingExe: string | null;
  running: RunningGame[];
  onSave: (exe: string, name: string, profileId: string) => void;
  onApply: (exe: string, name: string, profileId: string) => void;
  onHistory: (exe: string, name: string) => void;
}) {
  function getAssignment(exe: string): string {
    const lower = exe.toLowerCase();
    return assignments.find(
      (a) => lower.includes(a.exe_match.toLowerCase()) || a.exe_match.toLowerCase().includes(lower)
    )?.profile_id ?? "";
  }

  function isRunning(exe: string): RunningGame | undefined {
    const lower = exe.toLowerCase();
    return running.find((r) => {
      const rExe = r.exe.replace(/\\/g, "/").split("/").pop()?.replace(/\.exe$/i, "").toLowerCase() ?? "";
      return rExe.includes(lower) || lower.includes(rExe);
    });
  }

  if (games.length === 0) return null;

  return (
    <section className="gc-section">
      <div className="gc-section-hd">
        <span>Jogos Instalados</span>
        <span className="gc-section-count">{games.length}</span>
      </div>
      <div className="gc-game-list">
        {games.map((g) => {
          const profile = getAssignment(g.exe);
          const runEntry = isRunning(g.exe);
          const saving = savingExe === g.exe;
          const applying = applyingExe === g.exe;

          return (
            <div key={g.id} className={`gc-game-row${runEntry ? " gc-game-row-running" : ""}`}>
              <div className="gc-game-icon">🎮</div>

              <div className="gc-game-info">
                <strong className="gc-game-name">{g.name}</strong>
                <div className="gc-game-meta">
                  <AxBadge variant={LAUNCHER_COLOR[g.launcher] ?? "neutral"}>
                    {LAUNCHER_LABEL[g.launcher] ?? g.launcher}
                  </AxBadge>
                  {runEntry && (
                    <AxBadge variant="ok" dot>Em execução</AxBadge>
                  )}
                </div>
              </div>

              <div className="gc-game-profile-col">
                <ProfileSelector
                  exe={g.exe}
                  gameName={g.name}
                  current={profile}
                  saving={saving}
                  onChange={(pid) => onSave(g.exe, g.name, pid)}
                />
                {saving && <span className="gc-saving-label">Salvando…</span>}
              </div>

              <div className="gc-game-actions">
                {profile && runEntry && (
                  <AxButton
                    size="sm"
                    variant="primary"
                    onClick={() => onApply(g.exe, g.name, profile)}
                    disabled={applying}
                  >
                    {applying ? "Aplicando…" : "Aplicar"}
                  </AxButton>
                )}
                <AxButton size="sm" variant="ghost" onClick={() => onHistory(g.exe, g.name)}>
                  Histórico
                </AxButton>
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}

// ── Seção: Em Execução ───────────────────────────────────────────────────────

function RunningSection({
  running, assignments, boostingPid,
  onBoost, onRestore, onHistory,
}: {
  running: RunningGame[];
  assignments: GameAssignment[];
  boostingPid: number | null;
  onBoost: (pid: number, name: string) => void;
  onRestore: (pid: number) => void;
  onHistory: (exe: string, name: string) => void;
}) {
  const [boosted, setBoosted] = useState<Set<number>>(new Set());
  const latestFps = useLatestFps();
  const FPS_RECENT_MS = 10 * 60 * 1000;
  const recentFps = latestFps && (Date.now() - latestFps.ts) < FPS_RECENT_MS ? latestFps : null;

  function getAssignment(exe: string): GameAssignment | undefined {
    const lower = exe.replace(/\\/g, "/").split("/").pop()?.replace(/\.exe$/i, "").toLowerCase() ?? "";
    return assignments.find((a) =>
      lower.includes(a.exe_match.toLowerCase()) || a.exe_match.toLowerCase().includes(lower)
    );
  }

  async function handleBoost(pid: number, name: string) {
    onBoost(pid, name);
    setBoosted((prev) => new Set([...prev, pid]));
  }

  async function handleRestore(pid: number) {
    onRestore(pid);
    setBoosted((prev) => { const s = new Set(prev); s.delete(pid); return s; });
  }

  if (running.length === 0) return (
    <section className="gc-section">
      <div className="gc-section-hd">
        <span>Em Execução Agora</span>
        <AxBadge variant="neutral">0</AxBadge>
      </div>
      <div className="gc-empty-running">
        <span className="gc-empty-running-icon">🎮</span>
        <strong>Nenhum jogo em execução</strong>
        <p>Abra um jogo e ele aparecerá aqui com controles de otimização instantânea.</p>
      </div>
    </section>
  );

  return (
    <section className="gc-section">
      <div className="gc-section-hd">
        <span>Em Execução Agora</span>
        <AxBadge variant="ok" dot>{running.length}</AxBadge>
      </div>
      <div className="gc-hero-list">
        {running.map((g) => {
          const assignment = getAssignment(g.exe);
          const profileId = assignment?.profile_id;
          const meta = profileId ? PROFILE_META[profileId] : undefined;
          const isBoosted = boosted.has(g.pid);
          const busy = boostingPid === g.pid;
          const exeMatch = assignment?.exe_match ?? (g.exe.replace(/\\/g, "/").split("/").pop()?.replace(/\.exe$/i, "") ?? g.exe);

          return (
            <div key={g.pid} className={`gc-hero${isBoosted ? " gc-hero-boosted" : ""}`}>
              <div className="gc-hero-top">
                <div className="gc-hero-icon">🎮</div>
                <div className="gc-hero-title">
                  <strong>{g.name}</strong>
                  <AxBadge variant="ok" dot>Em execução</AxBadge>
                </div>
                <div className="gc-hero-priority">
                  {isBoosted
                    ? <AxBadge variant="ok">CPU elevada</AxBadge>
                    : <AxBadge variant="neutral">Prioridade normal</AxBadge>}
                </div>
              </div>

              <div className="gc-hero-profile">
                <span className="gc-hero-profile-lbl">Perfil ativo</span>
                {meta ? (
                  <span className="gc-hero-profile-val">
                    <span>{meta.icon}</span> {meta.label}
                  </span>
                ) : (
                  <span className="gc-hero-profile-none">Nenhum — atribua um perfil abaixo</span>
                )}
              </div>

              {recentFps ? (
                <div className="gc-hero-fps">
                  <span className="gc-fps-lbl">FPS</span>
                  <div className="gc-fps-vals">
                    <div className="gc-fps-stat">
                      <strong>{Math.round(fpsVal(recentFps, "fps_avg"))}</strong>
                      <small>Médio</small>
                    </div>
                    <div className="gc-fps-sep" />
                    <div className="gc-fps-stat">
                      <strong>{Math.round(fpsVal(recentFps, "fps_1pct_low"))}</strong>
                      <small>1% Low</small>
                    </div>
                    <div className="gc-fps-sep" />
                    <div className="gc-fps-stat">
                      <strong>{fpsVal(recentFps, "frametime_avg").toFixed(1)}</strong>
                      <small>ms frame</small>
                    </div>
                  </div>
                </div>
              ) : running.length > 0 ? (
                <div className="gc-hero-fps gc-fps-pending">
                  <span className="gc-fps-lbl">FPS</span>
                  <span className="gc-fps-pending-txt">Medindo… (alguns segundos)</span>
                </div>
              ) : null}

              <div className="gc-hero-actions">
                {!isBoosted ? (
                  <AxButton size="sm" variant="primary" onClick={() => handleBoost(g.pid, g.name)} disabled={busy}>
                    {busy ? "…" : "⚡ Boost CPU"}
                  </AxButton>
                ) : (
                  <AxButton size="sm" variant="ghost" onClick={() => handleRestore(g.pid)} disabled={busy}>
                    {busy ? "…" : "Restaurar"}
                  </AxButton>
                )}
                <AxButton size="sm" variant="ghost" onClick={() => onHistory(exeMatch, g.name)}>
                  Histórico
                </AxButton>
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}

// ── Seção: Histórico de um jogo ───────────────────────────────────────────────

function HistoryModal({
  exe, gameName, open, onClose,
}: {
  exe: string; gameName: string; open: boolean; onClose: () => void;
}) {
  const [runs, setRuns] = useState<GameRun[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!open || !isTauri()) return;
    setLoading(true);
    invokeCmd<GameRun[]>("game_runs_list", { exeMatch: exe })
      .then((data) => setRuns(data ?? []))
      .catch(() => setRuns([]))
      .finally(() => setLoading(false));
  }, [open, exe]);

  return (
    <AxModal
      open={open}
      title={`Histórico — ${gameName}`}
      onClose={onClose}
      footer={<button className="ax-btn ax-btn-ghost" onClick={onClose}>Fechar</button>}
    >
      {loading && <p style={{ color: "var(--ink-mid)", fontSize: 13 }}>Carregando…</p>}
      {!loading && runs.length === 0 && (
        <p style={{ color: "var(--ink-mid)", fontSize: 13 }}>
          Nenhuma ativação de perfil registrada para este jogo ainda.
        </p>
      )}
      {!loading && runs.length > 0 && (
        <ul className="gc-history-list">
          {runs.map((r) => (
            <li key={r.id} className="gc-history-item">
              <strong className="gc-history-profile">{profileLabel(r.profile_id)}</strong>
              <span className="gc-history-meta">
                {r.source === "auto" ? "Auto-aplicado" : "Manual"} · {relTime(r.ts)}
              </span>
            </li>
          ))}
        </ul>
      )}
    </AxModal>
  );
}

// ── Página principal ──────────────────────────────────────────────────────────

export function GameCenterPage() {
  const gc = useGameCenter();
  const [gpus, setGpus]           = useState<GpuInfo[]>([]);
  const [historyGame, setHistory] = useState<{ exe: string; name: string } | null>(null);
  const openHistory = (exe: string, name: string) => setHistory({ exe, name });

  useEffect(() => {
    if (!isTauri()) return;
    invokeCmd<GpuInfo[]>("gpu_detect", {}).then(setGpus).catch(() => {});
  }, []);

  const totalInstalled = gc.installed?.length ?? 0;
  const totalAssigned  = gc.assignments.length;

  return (
    <div className="gc-page">
      {/* Cabeçalho */}
      <header className="gc-header">
        <div>
          <h1>Game Center</h1>
          <p>
            {gc.available
              ? `${totalInstalled} jogo${totalInstalled !== 1 ? "s" : ""} detectado${totalInstalled !== 1 ? "s" : ""} · ${totalAssigned} com perfil atribuído`
              : "Abra o aplicativo TkSpeed para detectar jogos."}
          </p>
        </div>
        <div className="gc-header-actions">
          <AxButton size="sm" variant="ghost" icon="refresh" onClick={gc.scanInstalled} disabled={gc.loadingInstall}>
            {gc.loadingInstall ? "Detectando…" : "Detectar Jogos"}
          </AxButton>
          <AxButton size="sm" variant="ghost" icon="refresh" onClick={gc.pollRunning} disabled={gc.loadingRunning}>
            {gc.loadingRunning ? "…" : "Atualizar"}
          </AxButton>
        </div>
      </header>

      {/* Barra GPU */}
      {gpus.length > 0 && (
        <div className="gc-gpu-bar">
          <span className="gc-gpu-label">GPU</span>
          {gpus.map((g) => (
            <span key={g.name} className="gc-gpu-item">
              <AxBadge variant={g.vendor === "NVIDIA" ? "ok" : g.vendor === "AMD" ? "warn" : "neutral"}>
                {g.vendor}
              </AxBadge>
              <span className="gc-gpu-name">{g.name}</span>
            </span>
          ))}
        </div>
      )}

      {/* Perfis: cards descritivos */}
      <section className="gc-section">
        <div className="gc-section-hd">
          <span>Perfis de Otimização</span>
          <span className="gc-section-note">aplicados automaticamente quando o jogo abre</span>
        </div>
        <div className="gc-profile-cards">
          {PROFILES.filter((p) => p.id !== "").map((p) => {
            const meta = PROFILE_META[p.id];
            return (
              <div key={p.id} className={`gc-profile-card gc-profile-${p.id}`}>
                <div className="gc-profile-card-hd">
                  <span className="gc-profile-card-icon">{meta.icon}</span>
                  <strong className="gc-profile-card-label">{meta.label}</strong>
                </div>
                <p className="gc-profile-card-desc">{meta.desc}</p>
              </div>
            );
          })}
        </div>
      </section>

      {/* Em execução agora */}
      <RunningSection
        running={gc.running}
        assignments={gc.assignments}
        boostingPid={gc.boostingPid}
        onBoost={gc.boostGame}
        onRestore={gc.restoreGame}
        onHistory={openHistory}
      />

      {/* Jogos instalados */}
      {gc.installed === null && gc.loadingInstall && (
        <div className="gc-loading">Detectando jogos instalados…</div>
      )}

      {gc.installed !== null && gc.installed.length === 0 && (
        <AxEmptyState
          icon="game"
          title="Nenhum jogo instalado detectado"
          description="O TkSpeed detecta jogos do Steam, Epic Games, Riot Games, Battle.net, EA App, Ubisoft Connect, GOG Galaxy e Rockstar."
          action={
            <AxButton icon="refresh" onClick={gc.scanInstalled} disabled={gc.loadingInstall}>
              Tentar novamente
            </AxButton>
          }
        />
      )}

      {gc.installed && gc.installed.length > 0 && (
        <InstalledSection
          games={gc.installed}
          assignments={gc.assignments}
          savingExe={gc.savingExe}
          applyingExe={gc.applyingExe}
          running={gc.running}
          onSave={gc.saveAssignment}
          onApply={gc.applyProfile}
          onHistory={openHistory}
        />
      )}

      {/* Modal de histórico */}
      {historyGame && (
        <HistoryModal
          exe={historyGame.exe}
          gameName={historyGame.name}
          open={!!historyGame}
          onClose={() => setHistory(null)}
        />
      )}
    </div>
  );
}

// Manter export compatível com App.tsx
export { GameCenterPage as GameProfilesPage };
