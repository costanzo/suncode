import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { test } from "node:test";

const catalog = JSON.parse(
  readFileSync(new URL("../lib/feature-catalog.json", import.meta.url), "utf8"),
);

test("the public guide covers every durable feature exactly once", () => {
  const expected = readdirSync(
    new URL("../../.agents/features/", import.meta.url),
    { withFileTypes: true },
  )
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort();
  const actual = catalog.features
    .map((feature: { id: string }) => feature.id)
    .sort();
  assert.deepEqual(actual, expected);
  assert.equal(new Set(actual).size, actual.length);
});

test("all features have a valid group, availability, behavior, and scope", () => {
  const groups = new Set(
    catalog.groups.map((group: { id: string }) => group.id),
  );
  assert.equal(groups.size, catalog.groups.length);
  for (const feature of catalog.features) {
    assert.ok(groups.has(feature.group), feature.id);
    assert.ok(
      ["implemented", "preview", "release", "tooling"].includes(feature.status),
      feature.id,
    );
    for (const field of ["title", "description", "note"])
      assert.ok(feature[field]?.trim().length > 10, `${feature.id}: ${field}`);
    assert.ok(
      feature.details.length > 0 &&
        feature.details.every((detail: string) => detail.trim().length > 10),
      feature.id,
    );
  }
});
