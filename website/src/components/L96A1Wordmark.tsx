import { FONT7, renderPxText } from "@/lib/pxfont";

const SIZES = {
  mini: 1.7, // ~26px total height — navbar / sidebar
  sm: 3.4,   // small badge — hero eyebrow, CTA
  md: 6.5,   // footer logo
  lg: 9.5,   // big display
} as const;

export type WordmarkSize = keyof typeof SIZES;

const LINES = [
  ...renderPxText("L96A1", FONT7, "#"),
  "",
  ...renderPxText("SNIPER", FONT7, "#"),
];

/**
 * L96A1 two-line block wordmark, rendered from pure text glyphs.
 * Never an image — monospaced `#` fills on black, pure white.
 */
export default function L96A1Wordmark({
  size = "mini",
  className = "",
}: {
  size?: WordmarkSize;
  className?: string;
}) {
  return (
    <pre
      role="img"
      aria-label="L96A1 Sniper"
      className={`font-mono select-none whitespace-pre text-heading ${className}`}
      style={{ fontSize: SIZES[size], lineHeight: 1.15 }}
    >
      {LINES.join("\n")}
    </pre>
  );
}
