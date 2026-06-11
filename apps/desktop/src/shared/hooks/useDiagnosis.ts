import { useCallback, useEffect, useState } from "react";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";

// Espelha tk_contracts::{Diagnosis, Finding, TkSpeedScore}.
export type Severity = "Info" | "Low" | "Medium" | "High" | "Critical";
export type Classification = "Critico" | "Regular" | "Bom" | "Excelente" | "Elite";

export type Finding = {
  kind: string;
  severity: Severity;
  title: string;
  impact: string;
  solution: string;
};

export type TkSpeedScore = {
  total: number;
  classification: Classification;
  breakdown: {
    cpu: number; gpu: number; ram: number; storage: number; windows: number;
    network: number; temperature: number; games: number; stability: number;
  };
  score_version: string;
};

export type Diagnosis = {
  run_id: number;
  findings: Finding[];
  score: TkSpeedScore;
};

export type DiagnosisState = {
  diagnosis: Diagnosis | null;
  loading: boolean;
  error: string | null;
  /** dispara uma nova análise (botão "Analisar Agora") */
  analyze: () => void;
};

/**
 * Executa a análise real (comando `analyze_full`) no backend. Roda uma vez na
 * montagem (após ~1.5s para a janela de telemetria encher) e sob demanda.
 */
export function useDiagnosis(): DiagnosisState {
  const [diagnosis, setDiagnosis] = useState<Diagnosis | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const analyze = useCallback(async () => {
    if (!isTauri()) return;
    setLoading(true);
    setError(null);
    try {
      const d = await invokeCmd<Diagnosis>("analyze_full");
      setDiagnosis(d);
    } catch (e) {
      setError(typeof e === "string" ? e : JSON.stringify(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!isTauri()) return;
    const t = setTimeout(analyze, 1500);
    return () => clearTimeout(t);
  }, [analyze]);

  return { diagnosis, loading, error, analyze };
}
