import { useEffect, useState } from "react";

const normalizeHash = () => {
  const raw = window.location.hash.replace(/^#/, "") || "/";
  const withSlash = raw.startsWith("/") ? raw : `/${raw}`;
  return withSlash.length > 1 ? withSlash.replace(/\/+$/, "") : withSlash;
};

export function useHashRoute() {
  const [path, setPath] = useState(normalizeHash);

  useEffect(() => {
    const handleChange = () => {
      setPath(normalizeHash());
      window.scrollTo({ top: 0, behavior: "instant" });
    };

    window.addEventListener("hashchange", handleChange);
    return () => window.removeEventListener("hashchange", handleChange);
  }, []);

  return path;
}
