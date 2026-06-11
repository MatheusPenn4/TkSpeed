import { useEffect, useRef, useState } from "react";
import { AxIcon, type AxIconName } from "./AxIcon";
import { BrandSymbol } from "./branding/BrandSymbol";

export type AxCommand = {
  id: string;
  label: string;
  icon?: AxIconName;
  group?: string;
  run: () => void;
};

/** Liga ⌘K / Ctrl+K globalmente. */
export function useCmdkHotkey(onOpen: () => void) {
  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        onOpen();
      }
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, [onOpen]);
}

/** Command Palette (assinatura premium) — navegar telas + executar ações. */
export function AxCommandPalette({
  open,
  onClose,
  commands,
}: {
  open: boolean;
  onClose: () => void;
  commands: AxCommand[];
}) {
  const [q, setQ] = useState("");
  const [sel, setSel] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const filtered = commands.filter((c) => c.label.toLowerCase().includes(q.toLowerCase()));

  useEffect(() => {
    if (open) {
      setQ("");
      setSel(0);
      const t = window.setTimeout(() => inputRef.current?.focus(), 0);
      return () => window.clearTimeout(t);
    }
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const h = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
      else if (e.key === "ArrowDown") {
        e.preventDefault();
        setSel((s) => Math.min(s + 1, filtered.length - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSel((s) => Math.max(s - 1, 0));
      } else if (e.key === "Enter") {
        const c = filtered[sel];
        if (c) {
          onClose();
          c.run();
        }
      }
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, [open, filtered, sel, onClose]);

  if (!open) return null;

  return (
    <div className="ax-cmdk-overlay ax-anim" onClick={onClose}>
      <div className="ax-cmdk ax-anim" role="dialog" aria-modal="true" onClick={(e) => e.stopPropagation()}>
        <div className="ax-cmdk-input-row">
          <BrandSymbol size={16} className="ax-brand-mark" />
          <input
            ref={inputRef}
            className="ax-cmdk-input"
            placeholder="Buscar telas, executar ações…"
            value={q}
            onChange={(e) => {
              setQ(e.target.value);
              setSel(0);
            }}
          />
          <span className="ax-kbd">ESC</span>
        </div>
        <div className="ax-cmdk-list">
          {filtered.length === 0 ? (
            <div className="ax-cmdk-empty">Nada encontrado.</div>
          ) : (
            filtered.map((c, i) => (
              <div
                key={c.id}
                className={`ax-cmdk-item${i === sel ? " sel" : ""}`}
                onMouseEnter={() => setSel(i)}
                onClick={() => {
                  onClose();
                  c.run();
                }}
              >
                {c.icon && <AxIcon className="ax-cmdk-ico" name={c.icon} size={16} />}
                <span>{c.label}</span>
                {c.group && <span className="ax-cmdk-grp">{c.group}</span>}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
