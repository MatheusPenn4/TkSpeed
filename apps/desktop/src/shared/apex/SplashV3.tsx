import { BrandSymbol } from "./branding/BrandSymbol";
import { BrandLogo } from "./branding/BrandLogo";

/**
 * Splash V3 — abertura do produto. Símbolo central → logo → subtexto, fade-in
 * elegante (sem animação exagerada). Componente pronto para montar no boot;
 * a fiação ao ciclo de inicialização fica para a integração do shell (V3-2).
 */
export function SplashV3() {
  return (
    <div className="ax-splash">
      <div className="ax-splash-inner">
        <BrandSymbol size={76} className="ax-splash-sym" />
        <BrandLogo height={30} className="ax-splash-logo" />
        <p className="ax-splash-sub">Performance Engineering for Power Users</p>
      </div>
    </div>
  );
}
