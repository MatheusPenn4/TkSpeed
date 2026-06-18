import { useEffect, useState } from "react";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";
import { AxButton, AxBadge, AxEmptyState, useAxToast } from "@/shared/apex";
import "./games.css";

interface GpuDetectResult {
  name: string;
  vendor: string;
}

interface DetectedGame {
  pid: number;
  name: string;
  exe: string;
}

interface BoostResult {
  pid: number;
  success: boolean;
  message: string;
}

type GameState = "idle" | "boosted";

interface GameEntry {
  game: DetectedGame;
  state: GameState;
}

export function GameProfilesPage() {
  const push = useAxToast();
  const [entries, setEntries] = useState<GameEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [busyPid, setBusyPid] = useState<number | null>(null);
  const [gpus, setGpus] = useState<GpuDetectResult[]>([]);

  useEffect(() => {
    if (!isTauri()) return;
    invokeCmd<GpuDetectResult[]>("gpu_detect").then(setGpus).catch(() => {});
  }, []);

  async function refresh() {
    if (!isTauri()) { setLoading(false); return; }
    setLoading(true);
    try {
      const games = await invokeCmd<DetectedGame[]>("detect_games");
      setEntries((prev) => {
        const pidMap = new Map(prev.map((e) => [e.game.pid, e.state]));
        return games.map((g) => ({ game: g, state: pidMap.get(g.pid) ?? "idle" }));
      });
    } catch (e: unknown) {
      const msg = typeof e === "string" ? e : (e as { message?: string })?.message ?? "Erro ao detectar jogos.";
      push("danger", msg);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => { refresh(); }, []);

  async function boost(entry: GameEntry) {
    if (!isTauri()) return;
    setBusyPid(entry.game.pid);
    try {
      const r = await invokeCmd<BoostResult>("boost_game", { pid: entry.game.pid });
      if (r.success) {
        setEntries((prev) => prev.map((e) => e.game.pid === entry.game.pid ? { ...e, state: "boosted" } : e));
        push("ok", r.message);
      } else {
        push("danger", r.message);
      }
    } finally {
      setBusyPid(null);
    }
  }

  async function restore(entry: GameEntry) {
    if (!isTauri()) return;
    setBusyPid(entry.game.pid);
    try {
      const r = await invokeCmd<BoostResult>("restore_game_priority", { pid: entry.game.pid });
      if (r.success) {
        setEntries((prev) => prev.map((e) => e.game.pid === entry.game.pid ? { ...e, state: "idle" } : e));
        push("ok", r.message);
      } else {
        push("danger", r.message);
      }
    } finally {
      setBusyPid(null);
    }
  }

  const exeLabel = (exe: string) => {
    const parts = exe.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || exe;
  };

  return (
    <div className="gp-page">
      <header className="gp-header">
        <div className="gp-header-text">
          <h1 className="gp-title">Otimização para Jogos</h1>
          <p className="gp-subtitle">
            Detecta jogos em execução e eleva sua prioridade de CPU para Above Normal.
            Não usa Realtime — seguro e reversível.
          </p>
        </div>
        <AxButton variant="ghost" onClick={refresh} disabled={loading}>
          {loading ? "Detectando…" : "Atualizar"}
        </AxButton>
      </header>

      {gpus.length > 0 && (
        <div className="gp-gpu-bar">
          <span className="gp-gpu-label">GPU detectada</span>
          <div className="gp-gpu-list">
            {gpus.map((g) => (
              <div key={g.name} className="gp-gpu-item">
                <AxBadge variant={g.vendor === "NVIDIA" ? "ok" : g.vendor === "AMD" ? "warn" : "neutral"}>
                  {g.vendor}
                </AxBadge>
                <span className="gp-gpu-name">{g.name}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="gp-info-bar">
        <span className="gp-info-note">
          Prioridade Above Normal reserva mais ciclos de CPU para o jogo sem travar o sistema.
          Restaure ao fechar o jogo.
        </span>
      </div>

      {entries.length === 0 && !loading ? (
        <AxEmptyState
          icon="game"
          title="Nenhum jogo detectado"
          description="Abra um jogo suportado e clique em Atualizar para vê-lo aqui."
        />
      ) : (
        <div className="gp-list">
          {entries.map((entry) => {
            const busy = busyPid === entry.game.pid;
            return (
              <div key={entry.game.pid} className={`gp-row ${entry.state === "boosted" ? "gp-row-boosted" : ""}`}>
                <div className="gp-row-icon">🎮</div>
                <div className="gp-row-info">
                  <strong className="gp-row-name">{entry.game.name}</strong>
                  <span className="gp-row-meta">
                    {exeLabel(entry.game.exe)}
                  </span>
                </div>
                <div className="gp-row-badge">
                  {entry.state === "boosted" ? (
                    <AxBadge variant="ok">Above Normal</AxBadge>
                  ) : (
                    <AxBadge variant="neutral">Normal</AxBadge>
                  )}
                </div>
                <div className="gp-row-actions">
                  {entry.state === "idle" ? (
                    <AxButton variant="primary" size="sm" onClick={() => boost(entry)} disabled={busy}>
                      {busy ? "…" : "Boost"}
                    </AxButton>
                  ) : (
                    <AxButton variant="ghost" size="sm" onClick={() => restore(entry)} disabled={busy}>
                      {busy ? "…" : "Restaurar"}
                    </AxButton>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
