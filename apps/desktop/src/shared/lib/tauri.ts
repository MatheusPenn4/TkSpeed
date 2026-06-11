// Utilitários de integração com o Tauri, isolados num único lugar.

/** Detecta se estamos rodando dentro do shell Tauri (e não no navegador puro). */
export function isTauri(): boolean {
  return (
    typeof window !== "undefined" &&
    ("__TAURI_INTERNALS__" in window || "__TAURI__" in window)
  );
}

/** invoke() tipado com import dinâmico (não quebra o preview no navegador). */
export async function invokeCmd<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}
