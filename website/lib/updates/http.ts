import { createHash } from "node:crypto";
import { readFile, stat } from "node:fs/promises";
import path from "node:path";
import { targets, maximumCatalogBytes } from "./protocol.ts";

function problem(status: 400 | 404 | 503, code: string, title: string, head: boolean) {
  return new Response(head ? null : JSON.stringify({ type: "about:blank", title, status, code }), {
    status,
    headers: { "Content-Type": "application/problem+json", "Cache-Control": "no-store" },
  });
}

export async function catalogResponse(request: Request, channel: string, target: string,
  directory = path.join(process.cwd(), "updates")) {
  const head = request.method === "HEAD";
  if (channel !== "stable" || !targets.includes(target))
    return problem(400, "invalid_request", "Choose a supported channel and target", head);
  try {
    const file = path.join(directory, "v1", channel, target, "latest.json");
    if ((await stat(file)).size > maximumCatalogBytes) throw new Error("Catalog exceeds the supported size");
    const bytes = await readFile(file);
    if (bytes.length > maximumCatalogBytes) throw new Error("Catalog exceeds the supported size");
    const envelope = JSON.parse(bytes.toString("utf8"));
    if (envelope.signed?.channel !== channel || envelope.signed?.target !== target
      || !Array.isArray(envelope.signatures) || envelope.signatures.length === 0)
      throw new Error("Invalid catalog envelope");
    const etag = `"${createHash("sha256").update(bytes).digest("hex")}"`;
    const headers = { ETag: etag, "Cache-Control": "public, max-age=300, must-revalidate", "Content-Type": "application/json" };
    const matches = request.headers.get("if-none-match")?.split(",").some(value => {
      const tag = value.trim().replace(/^W\//, "");
      return tag === "*" || tag === etag;
    });
    return new Response(head || matches ? null : bytes.toString("utf8"), { status: matches ? 304 : 200, headers });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT")
      return problem(404, "catalog_not_found", "No published update catalog", head);
    return problem(503, "service_unavailable", "Update catalog temporarily unavailable", head);
  }
}
