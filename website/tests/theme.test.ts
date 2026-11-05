import assert from "node:assert/strict";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { themeBootstrap, themeStorageKey } from "../lib/theme.ts";

function firstPaint(
  saved: string | null,
  systemDark: boolean,
  blocked = false,
) {
  const attributes: Record<string, string> = {};
  runInNewContext(themeBootstrap, {
    localStorage: {
      getItem(key: string) {
        assert.equal(key, themeStorageKey);
        if (blocked) throw new Error("Storage blocked");
        return saved;
      },
    },
    window: {
      matchMedia(query: string) {
        assert.equal(query, "(prefers-color-scheme: dark)");
        return { matches: systemDark };
      },
    },
    document: {
      documentElement: {
        setAttribute(key: string, value: string) {
          attributes[key] = value;
        },
      },
    },
  });
  return attributes["data-theme"];
}

test("first paint follows the system without a valid saved choice", () => {
  assert.equal(firstPaint(null, false), "light");
  assert.equal(firstPaint(null, true), "dark");
  assert.equal(firstPaint("invalid", true), "dark");
});

test("a saved theme takes precedence over the system before hydration", () => {
  assert.equal(firstPaint("light", true), "light");
  assert.equal(firstPaint("dark", false), "dark");
});

test("blocked browser storage still produces a usable system theme", () => {
  assert.equal(firstPaint(null, false, true), "light");
  assert.equal(firstPaint(null, true, true), "dark");
});
