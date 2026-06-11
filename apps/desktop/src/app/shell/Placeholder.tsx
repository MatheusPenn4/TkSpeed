export function Placeholder({ title }: { title: string }) {
  return (
    <div>
      <h1 style={{ fontFamily: "var(--font-display)", fontSize: 28, marginBottom: 8 }}>{title}</h1>
      <p style={{ color: "var(--text-mid)" }}>
        Tela em desenvolvimento — wireframe e especificação em <span className="mono">docs/</span>.
      </p>
      <div className="glass" style={{ marginTop: 24, padding: 48, display: "grid", placeItems: "center", color: "var(--text-low)" }}>
        {title} · MVP / Fase 1–2
      </div>
    </div>
  );
}
