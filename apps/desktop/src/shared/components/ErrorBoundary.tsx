import React from "react";
import "./error-boundary.css";

/** Tela de erro fatal, reutilizada pelo ErrorBoundary (UI) e pela falha de bootstrap (backend). */
export function FatalErrorView({ title, message }: { title: string; message: string }) {
  return (
    <div className="fatal">
      <div className="glass fatal-card">
        <div className="fatal-icon">⚠</div>
        <h2>{title}</h2>
        <p>{message}</p>
        <button className="fatal-btn" onClick={() => window.location.reload()}>
          Recarregar
        </button>
      </div>
    </div>
  );
}

type Props = { children: React.ReactNode };
type State = { hasError: boolean };

/** Captura erros de render do React e evita a "tela branca". */
export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: unknown, info: unknown) {
    // Detalhe técnico só no console (M7) — nunca exibido ao usuário.
    console.error("[TkSpeed] erro de interface:", error, info);
  }

  render() {
    if (this.state.hasError) {
      return (
        <FatalErrorView
          title="Algo deu errado"
          message="Ocorreu um erro inesperado na interface. Recarregue para continuar."
        />
      );
    }
    return this.props.children;
  }
}
