// Arredonda para 3 casas: Math.cos/sin podem serializar o último dígito
// diferente entre Node (SSR) e o browser, quebrando a hidratação.
const r3 = (n: number) => Math.round(n * 1000) / 1000;

const TICKS = Array.from({ length: 12 }).map((_, i) => {
  const a = ((i * 30 - 90) * Math.PI) / 180;
  const cardinal = i % 3 === 0;
  const r1 = cardinal ? 83 : 88;
  const r2 = 94;
  return {
    key: i,
    x1: r3(100 + r1 * Math.cos(a)),
    y1: r3(100 + r1 * Math.sin(a)),
    x2: r3(100 + r2 * Math.cos(a)),
    y2: r3(100 + r2 * Math.sin(a)),
    cardinal,
  };
});

const MIL_OFFSETS = [26, 42, 58];

/**
 * Animated sniper-scope reticle — brand-safe HUD for empty + loading states.
 * Pure SVG/CSS (transform/opacity only), strict monochrome, 60fps friendly.
 */
export default function ScopeReticle({
  size = 190,
  scanning = false,
  labels = true,
  className = "",
}: {
  /** overall square size in px */
  size?: number;
  /** faster sweep + brighter elements while results stream in */
  scanning?: boolean;
  /** small mono HUD labels around the scope */
  labels?: boolean;
  className?: string;
}) {
  return (
    <div
      className={`relative select-none ${className}`}
      style={{ width: size, height: size }}
      role="img"
      aria-label={scanning ? "Scope reticle actively scanning" : "Sniper scope reticle"}
    >
      <svg viewBox="0 0 200 200" className="h-full w-full" aria-hidden>
        {/* corner brackets */}
        <g stroke="#424242" strokeWidth="1.5" fill="none">
          <path d="M 8 26 L 8 8 L 26 8" />
          <path d="M 174 8 L 192 8 L 192 26" />
          <path d="M 192 174 L 192 192 L 174 192" />
          <path d="M 26 192 L 8 192 L 8 174" />
        </g>

        {/* outer ring + ticks */}
        <circle cx="100" cy="100" r="96" fill="none" stroke="#2e2e2e" strokeWidth="1" />
        {TICKS.map((t) => (
          <line
            key={t.key}
            x1={t.x1}
            y1={t.y1}
            x2={t.x2}
            y2={t.y2}
            stroke={t.cardinal ? "#525252" : "#2e2e2e"}
            strokeWidth={t.cardinal ? 1.5 : 1}
          />
        ))}

        {/* slow counter-rotating dashed ring */}
        <g className="scope-rot-slow" style={{ transformOrigin: "100px 100px" }}>
          <circle cx="100" cy="100" r="78" fill="none" stroke="#424242" strokeWidth="1" strokeDasharray="2 6" />
        </g>

        {/* radar sweep wedge + leading edge */}
        <g
          className={scanning ? "scope-rot-sweep-fast" : "scope-rot-sweep"}
          style={{ transformOrigin: "100px 100px" }}
        >
          <path d="M 100 100 L 100 22 A 78 78 0 0 1 163 50 Z" fill="#ececec" opacity={scanning ? 0.055 : 0.035} />
          <line x1="100" y1="100" x2="100" y2="22" stroke="#ececec" strokeWidth="1" opacity={scanning ? 0.55 : 0.3} />
        </g>

        {/* crosshair with center gap */}
        <g stroke="#a1a1a1" strokeWidth="1">
          <line x1="100" y1="14" x2="100" y2="84" />
          <line x1="100" y1="116" x2="100" y2="186" />
          <line x1="14" y1="100" x2="84" y2="100" />
          <line x1="116" y1="100" x2="186" y2="100" />
        </g>

        {/* mil-dots */}
        <g fill="#737373">
          {MIL_OFFSETS.flatMap((o) => [
            <circle key={`t${o}`} cx="100" cy={100 - o} r="1.6" />,
            <circle key={`b${o}`} cx="100" cy={100 + o} r="1.6" />,
            <circle key={`l${o}`} cx={100 - o} cy="100" r="1.6" />,
            <circle key={`r${o}`} cx={100 + o} cy="100" r="1.6" />,
          ])}
        </g>

        {/* center gap ring */}
        <circle cx="100" cy="100" r="16" fill="none" stroke="#2e2e2e" strokeWidth="1" />

        {/* expanding ping from center (SMIL) */}
        <circle cx="100" cy="100" r="5" fill="none" stroke="#ececec" strokeWidth="1">
          <animate attributeName="r" values="5;58" dur={scanning ? "1.6s" : "2.8s"} repeatCount="indefinite" />
          <animate attributeName="opacity" values="0.55;0" dur={scanning ? "1.6s" : "2.8s"} repeatCount="indefinite" />
        </circle>

        {/* pulsing center dot */}
        <circle className="pulse-scope" cx="100" cy="100" r="2.2" fill="#ffffff" style={{ transformOrigin: "100px 100px" }} />
      </svg>

      {/* HUD labels */}
      {labels && (
        <>
          <span className="absolute -top-1 left-0 font-mono text-[9px] tracking-[0.18em] text-mute">L96 OPTIC</span>
          <span className="absolute -top-1 right-0 font-mono text-[9px] tracking-[0.18em] text-mute">5-25×56</span>
          <span className="absolute -bottom-1 left-0 flex items-center gap-1.5 font-mono text-[9px] tracking-[0.18em] text-mute">
            <span
              className={`inline-block size-1.5 rounded-full ${scanning ? "bg-found blink" : "bg-iconbox"}`}
            />
            {scanning ? "TRK ACTIVE" : "TRK IDLE"}
          </span>
          <span className="absolute -bottom-1 right-0 font-mono text-[9px] tracking-[0.18em] text-mute">
            RNG 748m
          </span>
        </>
      )}
    </div>
  );
}
