import { createContext, useCallback, useContext, useState } from "react";
import type { ReactNode } from "react";
import { AxIcon, type AxIconName } from "./AxIcon";
import { BrandSymbol } from "./branding/BrandSymbol";

type AxToastVariant = "ok" | "danger" | "signal";
type AxToastItem = { id: number; variant: AxToastVariant; message: string };

const ICON: Record<AxToastVariant, AxIconName> = {
  ok: "check",
  danger: "alert",
  signal: "bolt",
};

type PushFn = (variant: AxToastVariant, message: string) => void;
const Ctx = createContext<PushFn | null>(null);

let seq = 0;

export function AxToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<AxToastItem[]>([]);

  const push = useCallback<PushFn>((variant, message) => {
    const id = ++seq;
    setToasts((prev) => [...prev, { id, variant, message }]);
    window.setTimeout(() => setToasts((prev) => prev.filter((t) => t.id !== id)), 4200);
  }, []);

  return (
    <Ctx.Provider value={push}>
      {children}
      <div className="ax-toast-wrap" aria-live="polite">
        {toasts.map((t) => (
          <div key={t.id} className={`ax-toast ax-anim ax-toast-${t.variant}`}>
            <AxIcon className="ax-toast-i" name={ICON[t.variant]} size={16} />
            <span>{t.message}</span>
            <BrandSymbol size={13} className="ax-brand-mark" alt="" />
          </div>
        ))}
      </div>
    </Ctx.Provider>
  );
}

export function useAxToast(): PushFn {
  return useContext(Ctx) ?? (() => undefined);
}
