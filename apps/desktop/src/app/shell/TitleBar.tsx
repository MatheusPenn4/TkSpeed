import { isTauri } from "@/shared/lib/tauri";

// Controles reais da janela Tauri (janela sem decoração nativa).
async function windowAction(action: "min" | "max" | "close") {
  if (!isTauri()) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  const w = getCurrentWindow();
  if (action === "min") await w.minimize();
  else if (action === "max") await w.toggleMaximize();
  else await w.close();
}

export function TitleBar() {
  return (
    // data-tauri-drag-region torna a barra arrastável (mover a janela).
    <header className="titlebar" data-tauri-drag-region>
      <div className="brand" data-tauri-drag-region>
        <span className="dot" />
        TkSpeed
      </div>
      <div className="win-btns">
        <button title="Minimizar" onClick={() => windowAction("min")}>—</button>
        <button title="Maximizar / Restaurar" onClick={() => windowAction("max")}>▢</button>
        <button className="close" title="Fechar" onClick={() => windowAction("close")}>✕</button>
      </div>
    </header>
  );
}
