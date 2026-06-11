import { useEffect } from "react";
import type { ReactNode } from "react";
import { AxIcon } from "./AxIcon";
import { BrandSymbol } from "./branding/BrandSymbol";

export function AxModal({
  open,
  title,
  children,
  footer,
  onClose,
}: {
  open: boolean;
  title: string;
  children: ReactNode;
  footer?: ReactNode;
  onClose: () => void;
}) {
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className="ax-modal-overlay ax-anim" onClick={onClose}>
      <div className="ax-modal ax-anim" role="dialog" aria-modal="true" aria-label={title} onClick={(e) => e.stopPropagation()}>
        <header className="ax-modal-head">
          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            <BrandSymbol size={18} className="ax-brand-mark" />
            <h2>{title}</h2>
          </div>
          <button className="ax-modal-x" aria-label="Fechar" onClick={onClose}>
            <AxIcon name="x" size={18} />
          </button>
        </header>
        <div className="ax-modal-body">{children}</div>
        {footer && <footer className="ax-modal-foot">{footer}</footer>}
      </div>
    </div>
  );
}
