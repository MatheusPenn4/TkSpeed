import { useEffect } from "react";
import type { ReactNode } from "react";
import { Icon } from "./Icon";

/**
 * Modal V2 — camada flutuante de vidro para confirmação de ações destrutivas
 * (restaurar snapshot, reverter otimização). Fecha por overlay, botão ou Esc.
 */
export function Modal({
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
    <div className="v2-modal-overlay" onClick={onClose}>
      <div
        className="v2-modal glass"
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onClick={(e) => e.stopPropagation()}
      >
        <header className="v2-modal-head">
          <h2>{title}</h2>
          <button className="v2-modal-close" aria-label="Fechar" onClick={onClose}>
            <Icon name="x" size={18} />
          </button>
        </header>
        <div className="v2-modal-body">{children}</div>
        {footer && <footer className="v2-modal-foot">{footer}</footer>}
      </div>
    </div>
  );
}
