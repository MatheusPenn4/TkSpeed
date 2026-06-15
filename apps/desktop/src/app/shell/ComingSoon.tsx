import { AxEmptyState, type AxIconName } from "@/shared/apex";

/** Shell premium "Em breve" para telas cujo backend ainda não existe (V3). */
export function ComingSoon({
  title,
  description,
  icon = "bolt",
}: {
  title: string;
  description?: string;
  icon?: AxIconName;
}) {
  return (
    <div style={{ maxWidth: "var(--ax-content-max)", margin: "0 auto" }}>
      <h1 style={{ font: "600 22px/1.2 var(--ax-font-display)", color: "var(--ink-hi)", marginBottom: 16 }}>{title}</h1>
      <div className="ax-surface ax-card">
        <AxEmptyState
          variant="soon"
          icon={icon}
          title="Em breve"
          description={description ?? "Esta funcionalidade estará disponível em breve."}
        />
      </div>
    </div>
  );
}
