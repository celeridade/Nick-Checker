import Link from "next/link";
import L96A1Wordmark from "@/components/L96A1Wordmark";

const COLS: Array<{ title: string; links: Array<{ label: string; href: string }> }> = [
  {
    title: "PRODUCT",
    links: [
      { label: "Features", href: "/#features" },
      { label: "Pricing", href: "/pricing" },
      { label: "Security", href: "/#security" },
    ],
  },
  {
    title: "SEARCH",
    links: [
      { label: "OSINT Tools", href: "/username-search" },
      { label: "Username", href: "/username-search" },
      { label: "Email", href: "/username-search" },
      { label: "Phone", href: "/username-search" },
    ],
  },
  {
    title: "COMPANY",
    links: [
      { label: "About", href: "/#about" },
      { label: "Blog", href: "/#blog" },
      { label: "Contact", href: "/#contact" },
    ],
  },
  {
    title: "LEGAL",
    links: [
      { label: "Opt Out", href: "/#opt-out" },
      { label: "Privacy", href: "/#privacy" },
      { label: "Terms", href: "/#terms" },
    ],
  },
];

export default function SiteFooter() {
  return (
    <footer className="border-t border-line bg-surface">
      <div className="mx-auto grid max-w-[1100px] gap-10 px-6 py-14 sm:grid-cols-2 lg:grid-cols-6">
        <div className="lg:col-span-2">
          <L96A1Wordmark size="sm" />
          <p className="mt-4 text-sm text-mute">Made in the USA</p>
          <p className="mt-1 font-mono text-xs text-mute">© 2026 L96A1</p>
        </div>
        {COLS.map((col) => (
          <div key={col.title}>
            <p className="eyebrow">{col.title}</p>
            <ul className="mt-4 space-y-2.5">
              {col.links.map((l) => (
                <li key={l.label}>
                  <Link href={l.href} className="text-sm text-sub transition-colors hover:text-heading">
                    {l.label}
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    </footer>
  );
}
