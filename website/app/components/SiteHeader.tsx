"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useRef, useState } from "react";
import { ArrowUpRight, Menu, X } from "lucide-react";
import { githubUrl } from "../../lib/features";
import { ThemeToggle } from "./ThemeToggle";
import { BrandMark } from "./BrandMark";

const links = [
  { href: "/", label: "Overview" },
  { href: "/features", label: "Features" },
  { href: "/#workflow", label: "How it works" },
];

export function SiteHeader() {
  const pathname = usePathname();
  const [open, setOpen] = useState(false);
  const container = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!open) return;
    const dismiss = (event: PointerEvent) => {
      if (!container.current?.contains(event.target as Node)) setOpen(false);
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpen(false);
        trigger.current?.focus();
      }
    };
    document.addEventListener("pointerdown", dismiss);
    document.addEventListener("keydown", escape);
    return () => {
      document.removeEventListener("pointerdown", dismiss);
      document.removeEventListener("keydown", escape);
    };
  }, [open]);

  return (
    <header className="site-header">
      <div className="header-inner">
        <Link
          className="brand"
          href="/"
          aria-label="SunCode home"
          onClick={() => setOpen(false)}
        >
          <BrandMark size={28} />
          <span>SunCode</span>
        </Link>
        <nav className="desktop-nav" aria-label="Primary navigation">
          {links.map(({ href, label }) => (
            <Link
              key={href}
              href={href}
              aria-current={href === pathname ? "page" : undefined}
            >
              {label}
            </Link>
          ))}
        </nav>
        <a
          className="header-source"
          href={githubUrl}
          aria-label="View SunCode on GitHub"
          title="View SunCode on GitHub"
          target="_blank"
          rel="noreferrer"
        >
          <span className="header-source-label">GitHub</span>{" "}
          <ArrowUpRight size={15} aria-hidden="true" />
        </a>
        <ThemeToggle />
        <div className="mobile-nav" ref={container}>
          <button
            ref={trigger}
            className="icon-button"
            type="button"
            aria-label={open ? "Close navigation" : "Open navigation"}
            aria-expanded={open}
            aria-controls="mobile-navigation"
            onClick={() => setOpen(!open)}
          >
            {open ? <X size={21} /> : <Menu size={21} />}
          </button>
          {open && (
            <nav
              id="mobile-navigation"
              className="mobile-menu"
              aria-label="Mobile navigation"
            >
              {links.map(({ href, label }) => (
                <Link
                  key={href}
                  href={href}
                  aria-current={href === pathname ? "page" : undefined}
                  onClick={() => setOpen(false)}
                >
                  {label}
                </Link>
              ))}
              <a
                href={githubUrl}
                target="_blank"
                rel="noreferrer"
                onClick={() => setOpen(false)}
              >
                View source <ArrowUpRight size={15} aria-hidden="true" />
              </a>
            </nav>
          )}
        </div>
      </div>
    </header>
  );
}
