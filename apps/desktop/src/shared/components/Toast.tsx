import { createContext, useCallback, useContext, useState } from "react";
import type { ReactNode } from "react";
import { Icon, type IconName } from "./Icon";

/**
 * Toast V2 — feedback global e uniforme de ações (substitui as mensagens inline
 * por-tela da auditoria). Monta uma vez via <ToastProvider> no App.
 */
type ToastVariant = "success" | "danger" | "info";
type ToastItem = { id: number; variant: ToastVariant; message: string };

const ICON: Record<ToastVariant, IconName> = {
  success: "check",
  danger: "alert",
  info: "info",
};

type PushFn = (variant: ToastVariant, message: string) => void;
const ToastCtx = createContext<PushFn | null>(null);

let seq = 0;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const push = useCallback<PushFn>((variant, message) => {
    const id = ++seq;
    setToasts((prev) => [...prev, { id, variant, message }]);
    window.setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 4200);
  }, []);

  return (
    <ToastCtx.Provider value={push}>
      {children}
      <div className="v2-toast-wrap" aria-live="polite">
        {toasts.map((t) => (
          <div key={t.id} className={`v2-toast glass v2-toast-${t.variant}`}>
            <Icon name={ICON[t.variant]} size={16} />
            <span>{t.message}</span>
          </div>
        ))}
      </div>
    </ToastCtx.Provider>
  );
}

/** Retorna a função push(variant, message). No-op seguro fora do provider. */
export function useToast(): PushFn {
  const ctx = useContext(ToastCtx);
  return ctx ?? (() => undefined);
}
