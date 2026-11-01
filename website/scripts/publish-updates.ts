import path from "node:path";
import { parseArgs } from "node:util";
import { publishRelease, websiteRoot } from "../lib/updates/publisher.ts";

try {
  const { values } = parseArgs({ options: {
    release: { type: "string" }, "private-key": { type: "string" }, "key-id": { type: "string" },
    distribution: { type: "string", default: path.join(websiteRoot, "../apps/desktop-avalonia/Updates/update-distribution.json") },
    help: { type: "boolean", short: "h" },
  } });
  if (values.help) {
    console.log("pnpm updates:publish --release /path/release.json --private-key /outside/repository/private.pem --key-id <pinned-key-id> [--distribution /path/update-distribution.json]");
  } else {
    if (!values.release || !values["private-key"] || !values["key-id"]) throw new Error("Provide --release, --private-key and --key-id. Use --help for usage.");
    const result = await publishRelease({ releaseFile: path.resolve(values.release), privateKeyFile: path.resolve(values["private-key"]), keyId: values["key-id"], distributionFile: path.resolve(values.distribution!) });
    console.log(JSON.stringify(result, null, 2));
  }
} catch (error) {
  console.error(`Update publication failed: ${error instanceof Error ? error.message : "unknown error"}`);
  process.exitCode = 1;
}
