"use client";

import { useEffect, useRef, useState } from "react";
import { Moon, Sun } from "lucide-react";
import { themeStorageKey, type Theme } from "../../lib/theme";

export function ThemeToggle() {
  const [theme, setTheme] = useState<Theme>("dark");
  const preference = useRef<Theme | null>(null);

  function apply(value: Theme) {
    document.documentElement.dataset.theme = value;
    setTheme(value);
  }

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const readPreference = () => {
      try {
        const saved = localStorage.getItem(themeStorageKey);
        preference.current =
          saved === "light" || saved === "dark" ? saved : null;
      } catch {
        preference.current = null;
      }
    };
    const sync = () =>
      apply(preference.current ?? (media.matches ? "dark" : "light"));
    const systemChanged = () => {
      if (!preference.current) sync();
    };
    const storageChanged = (event: StorageEvent) => {
      if (event.key === themeStorageKey || event.key === null) {
        readPreference();
        sync();
      }
    };
    readPreference();
    sync();
    media.addEventListener("change", systemChanged);
    window.addEventListener("storage", storageChanged);
    return () => {
      media.removeEventListener("change", systemChanged);
      window.removeEventListener("storage", storageChanged);
    };
  }, []);

  function toggle() {
    const next =
      document.documentElement.dataset.theme === "light" ? "dark" : "light";
    preference.current = next;
    apply(next);
    try {
      localStorage.setItem(themeStorageKey, next);
    } catch {
      /* Keep the selection for this visit when storage is unavailable. */
    }
  }

  const label =
    theme === "light" ? "Switch to dark mode" : "Switch to light mode";
  return (
    <button
      className="icon-button theme-toggle"
      type="button"
      aria-label={label}
      title={label}
      onClick={toggle}
    >
      <Sun className="theme-icon-sun" size={19} aria-hidden="true" />
      <Moon className="theme-icon-moon" size={19} aria-hidden="true" />
    </button>
  );
}
