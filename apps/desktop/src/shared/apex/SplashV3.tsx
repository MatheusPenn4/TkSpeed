import { BrandSymbol } from "./branding/BrandSymbol";

export function SplashV3() {
  return (
    <div className="ax-splash" aria-label="Inicializando TkSpeed" role="status">
      <div className="ax-splash-grid" aria-hidden />

      <div className="ax-splash-inner">
        <div className="ax-splash-sym-wrap" aria-hidden>
          <div className="ax-splash-ring ax-splash-ring--outer" />
          <div className="ax-splash-ring ax-splash-ring--inner" />
          <div className="ax-splash-ring ax-splash-ring--pulse" />
          <BrandSymbol size={88} className="ax-splash-sym" />
        </div>

        <div className="ax-splash-brand">
          <span className="ax-splash-name">TkSpeed</span>
          <span className="ax-splash-tagline">Engenharia de Performance</span>
        </div>

        <div className="ax-splash-footer" aria-hidden>
          <div className="ax-splash-bar-track">
            <div className="ax-splash-bar" />
          </div>
          <span className="ax-splash-status">inicializando</span>
        </div>
      </div>
    </div>
  );
}
