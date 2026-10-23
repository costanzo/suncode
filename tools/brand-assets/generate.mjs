import { readFile, writeFile, mkdir } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { resolve, dirname, relative } from "node:path";
import { createHash } from "node:crypto";
import sharp from "sharp";

const root = fileURLToPath(new URL("../../", import.meta.url));
const check = process.argv.includes("--check");
const source = "design-system/public/assets/logo.svg";
const artwork = await readFile(resolve(root, source), "utf8");
const outputs = new Map();
const put = (path, content) => outputs.set(path, Buffer.from(content));
const number = (n) => Number(n.toFixed(6));

// Flatten the reference's circular luminance masks into equivalent closed paths.
// Android VectorDrawable and native geometry consumers do not support SVG masks.
function extractMark(id) {
  const mask = artwork.match(
    new RegExp(`<mask\\s+id="${id}"[\\s\\S]*?</mask>`),
  )?.[0];
  if (!mask) throw new Error(`Missing reference mask: ${id}`);
  const radius = Number(mask.match(/<circle r="([\d.]+)"/)?.[1]);
  const cuts = [...mask.matchAll(/<path d="([^"]+)"/g)].map((m) => m[1]);
  const commands = cuts.map((d) =>
    d.replace(/-?\d+(?:\.\d+)?/g, "").replaceAll(" ", ""),
  );
  if (
    cuts.length !== 2 ||
    commands[0] !== "MHVHLZ" ||
    commands[1] !== "MHLHVZ"
  ) {
    throw new Error(`Unexpected cut geometry in ${id}; review the exporter`);
  }
  const [upper, lower] = cuts.map((d) =>
    d.match(/-?\d+(?:\.\d+)?/g).map(Number),
  );
  const [leftTop, top, , bottom, leftBottom] = upper;
  if (
    lower[1] !== -bottom ||
    lower[2] !== -leftBottom ||
    lower[3] !== -leftTop ||
    lower[4] !== -top
  ) {
    throw new Error(
      `Non-symmetric reference cuts in ${id}; review the exporter`,
    );
  }
  const xTop = number(Math.sqrt(radius ** 2 - top ** 2));
  const xBottom = number(Math.sqrt(radius ** 2 - bottom ** 2));
  const path = `M${xTop} ${top}A${radius} ${radius} 0 0 0 ${-xBottom} ${-bottom}H${-leftBottom}L${-leftTop} ${-top}H${-xTop}A${radius} ${radius} 0 0 0 ${xBottom} ${bottom}H${leftBottom}L${leftTop} ${top}Z`;
  return { radius, path };
}

const main = extractMark("suncode-main-mask");
const small24 = extractMark("suncode-24-mask");
const small16 = extractMark("suncode-16-mask");
const light = artwork.match(/<g fill="(#[\da-fA-F]+)">/)?.[1];
const dark = [...artwork.matchAll(/<g fill="(#[\da-fA-F]+)">/g)][1]?.[1];
const appGroup = artwork.match(/<g id="suncode-app-icon">([\s\S]*?)<\/g>/)?.[1];
const plate = appGroup?.match(/<rect[^>]+\/>/)?.[0];
const placement = appGroup?.match(
  /transform="translate\(([^)]+)\) scale\(([^)]+)\)"/,
);
if (!light || !dark || !plate || !placement)
  throw new Error("Incomplete reference artwork");
const svg = (viewBox, body) =>
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${viewBox}" fill="none">${body}</svg>\n`;
const mark = (geometry, color = "currentColor") =>
  svg(
    `${-geometry.radius} ${-geometry.radius} ${geometry.radius * 2} ${geometry.radius * 2}`,
    `<path fill="${color}" d="${geometry.path}"/>`,
  );
const app = svg(
  "0 0 224 224",
  `${plate}<path fill="${dark}" transform="translate(${placement[1]}) scale(${placement[2]})" d="${main.path}"/>`,
);
// macOS square icons occupy an 824px plate on a transparent 1024px canvas.
// Keep the same inset in the live Dock PNG and every ICNS representation.
const macApp = svg(
  "0 0 1024 1024",
  `<g transform="translate(100 100) scale(${824 / 224})">${plate}<path fill="${dark}" transform="translate(${placement[1]}) scale(${placement[2]})" d="${main.path}"/></g>`,
);
const png = async (input, size, background) => {
  let image = sharp(Buffer.from(input), { density: 384 }).resize(size, size);
  if (background) image = image.flatten({ background });
  return image.png().toBuffer();
};
const catalog = "design-system/src/assets/logos";
const desktop = "apps/desktop-avalonia/Assets";
put("design-system/src/assets/platform/suncode-macos.svg", macApp);
put(`${desktop}/suncode-macos-1024.png`, await png(macApp, 1024));
for (const base of [catalog, `${desktop}/logo`]) {
  put(`${base}/suncode-logo.svg`, app);
  put(`${base}/suncode-logo-small.svg`, app);
  for (const size of [32, 64, 128, 256, 512, 1024])
    put(`${base}/suncode-logo-${size}.png`, await png(app, size));
  for (const size of [32, 64, 128, 256])
    put(`${base}/suncode-logo-small-${size}.png`, await png(app, size));
}
for (const [name, geometry] of [
  ["", main],
  ["-24", small24],
  ["-16", small16],
]) {
  put(`${catalog}/suncode-mark${name}.svg`, mark(geometry));
  for (const [theme, color] of [
    ["light", light],
    ["dark", dark],
  ]) {
    put(`${catalog}/suncode-mark${name}-${theme}.svg`, mark(geometry, color));
  }
}
put(`${desktop}/suncode-logo-256.png`, await png(app, 256));
put(
  `${desktop}/logo/suncode-mark.axaml`,
  `<ResourceDictionary xmlns="https://github.com/avaloniaui" xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"><StreamGeometry x:Key="SunCodeBrandGeometry">${main.path}</StreamGeometry></ResourceDictionary>\n`,
);

// PNG-compressed, multi-resolution ICO and ICNS require no platform SDK tools.
const iconSizes = [16, 24, 32, 48, 64, 128, 256];
const iconPngs = await Promise.all(iconSizes.map((size) => png(app, size)));
let offset = 6 + 16 * iconSizes.length;
const icoHeader = Buffer.alloc(offset);
icoHeader.writeUInt16LE(1, 2);
icoHeader.writeUInt16LE(iconSizes.length, 4);
for (const [index, size] of iconSizes.entries()) {
  const entry = 6 + index * 16;
  icoHeader[entry] = size % 256;
  icoHeader[entry + 1] = size % 256;
  icoHeader.writeUInt16LE(1, entry + 4);
  icoHeader.writeUInt16LE(32, entry + 6);
  icoHeader.writeUInt32LE(iconPngs[index].length, entry + 8);
  icoHeader.writeUInt32LE(offset, entry + 12);
  offset += iconPngs[index].length;
}
put(`${desktop}/suncode.ico`, Buffer.concat([icoHeader, ...iconPngs]));
const chunks = [];
for (const [type, size] of [
  ["icp4", 16],
  ["icp5", 32],
  ["icp6", 64],
  ["ic07", 128],
  ["ic08", 256],
  ["ic09", 512],
  ["ic10", 1024],
  ["ic11", 32],
  ["ic12", 64],
  ["ic13", 256],
  ["ic14", 512],
]) {
  const data = await png(macApp, size);
  const header = Buffer.alloc(8);
  header.write(type);
  header.writeUInt32BE(data.length + 8, 4);
  chunks.push(header, data);
}
const icnsHeader = Buffer.alloc(8);
icnsHeader.write("icns");
icnsHeader.writeUInt32BE(
  8 + chunks.reduce((n, chunk) => n + chunk.length, 0),
  4,
);
const icns = Buffer.concat([icnsHeader, ...chunks]);
put("design-system/src/assets/platform/suncode-desktop.icns", icns);
put(`${desktop}/suncode-desktop.icns`, icns);
for (const size of [16, 24, 32, 48, 64, 128, 256, 512]) {
  put(
    `apps/desktop-avalonia/packaging/linux/icons/hicolor/${size}x${size}/apps/suncode.png`,
    await png(app, size),
  );
}
put(
  "apps/desktop-avalonia/packaging/linux/icons/hicolor/scalable/apps/suncode.svg",
  app,
);

const favicon = svg(
  "-8 -8 16 16",
  `<style>path{fill:${light}}@media(prefers-color-scheme:dark){path{fill:${dark}}}</style><path d="${small16.path}"/>`,
);
for (const base of [
  "website/public",
  "apps/web/public",
  "design-system/public/assets",
]) {
  put(`${base}/suncode-logo.svg`, app);
  put(`${base}/suncode-logo-small.svg`, app);
  put(`${base}/favicon.svg`, favicon);
  put(`${base}/favicon.ico`, Buffer.concat([icoHeader, ...iconPngs]));
  put(`${base}/apple-touch-icon.png`, await png(app, 180, "#121519"));
  for (const size of [16, 24, 40]) {
    const geometry = size === 16 ? small16 : size === 24 ? small24 : main;
    for (const [theme, color] of [
      ["light", light],
      ["dark", dark],
    ])
      put(`${base}/suncode-mark-${size}-${theme}.svg`, mark(geometry, color));
  }
}

const android = "apps/mobile/androidApp/src/main/res";
for (const [density, size] of [
  ["mdpi", 48],
  ["hdpi", 72],
  ["xhdpi", 96],
  ["xxhdpi", 144],
  ["xxxhdpi", 192],
]) {
  put(`${android}/mipmap-${density}/ic_launcher.png`, await png(app, size));
  const round = svg(
    "0 0 224 224",
    `<circle cx="112" cy="112" r="112" fill="#121519"/><path fill="${dark}" transform="translate(${placement[1]}) scale(${placement[2]})" d="${main.path}"/>`,
  );
  put(
    `${android}/mipmap-${density}/ic_launcher_round.png`,
    await png(round, size),
  );
}
const vector = (body) =>
  `<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="108dp" android:height="108dp" android:viewportWidth="108" android:viewportHeight="108">${body}</vector>\n`;
const foreground = vector(
  `<group android:translateX="54" android:translateY="54" android:scaleX="0.4" android:scaleY="0.4"><path android:fillColor="${dark}" android:pathData="${main.path}"/></group>`,
);
put(`${android}/drawable-v24/ic_launcher_foreground.xml`, foreground);
put(
  `${android}/drawable/ic_launcher_background.xml`,
  vector(
    '<path android:fillColor="#121519" android:pathData="M0 0H108V108H0Z"/>',
  ),
);
for (const name of ["ic_launcher", "ic_launcher_round"]) {
  put(
    `${android}/mipmap-anydpi-v26/${name}.xml`,
    '<?xml version="1.0" encoding="utf-8"?>\n<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android"><background android:drawable="@drawable/ic_launcher_background"/><foreground android:drawable="@drawable/ic_launcher_foreground"/></adaptive-icon>\n',
  );
  put(
    `${android}/mipmap-anydpi-v33/${name}.xml`,
    '<?xml version="1.0" encoding="utf-8"?>\n<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android"><background android:drawable="@drawable/ic_launcher_background"/><foreground android:drawable="@drawable/ic_launcher_foreground"/><monochrome android:drawable="@drawable/ic_launcher_foreground"/></adaptive-icon>\n',
  );
}
put(
  "apps/mobile/shared/src/commonMain/composeResources/drawable/suncode_mark.xml",
  `<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="24dp" android:height="24dp" android:viewportWidth="24" android:viewportHeight="24"><group android:translateX="12" android:translateY="12"><path android:fillColor="#000000" android:pathData="${small24.path}"/></group></vector>\n`,
);
// iOS owns corner masking; its source is opaque and has no pre-rounded corners.
const ios = svg(
  "0 0 224 224",
  `<rect width="224" height="224" fill="#121519"/><path fill="${dark}" transform="translate(${placement[1]}) scale(${placement[2]})" d="${main.path}"/>`,
);
put(
  "apps/mobile/iosApp/iosApp/Assets.xcassets/AppIcon.appiconset/app-icon-1024.png",
  await png(ios, 1024, "#121519"),
);

const iosIcons = "apps/mobile/iosApp/iosApp/Assets.xcassets/AppIcon.appiconset";
const tinted = svg(
  "0 0 224 224",
  `<rect width="224" height="224" fill="#000000"/><path fill="#FFFFFF" transform="translate(${placement[1]}) scale(${placement[2]})" d="${main.path}"/>`,
);
put(`${iosIcons}/app-icon-tinted-1024.png`, await png(tinted, 1024, "#000000"));
put(
  `${iosIcons}/Contents.json`,
  JSON.stringify(
    {
      images: [
        {
          filename: "app-icon-1024.png",
          idiom: "universal",
          platform: "ios",
          size: "1024x1024",
        },
        {
          filename: "app-icon-1024.png",
          appearances: [{ appearance: "luminosity", value: "dark" }],
          idiom: "universal",
          platform: "ios",
          size: "1024x1024",
        },
        {
          filename: "app-icon-tinted-1024.png",
          appearances: [{ appearance: "luminosity", value: "tinted" }],
          idiom: "universal",
          platform: "ios",
          size: "1024x1024",
        },
      ],
      info: { author: "xcode", version: 1 },
    },
    null,
    2,
  ) + "\n",
);

const social = svg(
  "0 0 1200 630",
  `<rect width="1200" height="630" fill="#0D0F12"/><path d="M0 84H1200M0 546H1200M96 0V630M1104 0V630" stroke="#292F36"/><rect x="96" y="84" width="1008" height="462" fill="#121519" stroke="#46505B"/><path fill="${dark}" transform="translate(167 155) scale(0.2564)" d="${main.path}"/><text x="214" y="164" fill="#EDF0F3" font-family="Arial, sans-serif" font-size="28" font-weight="700">SunCode</text><text x="144" y="304" fill="#EDF0F3" font-family="Arial, sans-serif" font-size="68" font-weight="700">A coding agent</text><text x="144" y="382" fill="${dark}" font-family="Arial, sans-serif" font-size="68" font-weight="700">you can review.</text><text x="146" y="454" fill="#A7AFB9" font-family="Arial, sans-serif" font-size="22">Local-first software work with visible, scoped, reversible machine access.</text><circle cx="1020" cy="162" r="7" fill="#9FB3C3"/><text x="1040" y="170" fill="#9FB3C3" font-family="monospace" font-size="13">OPEN SOURCE</text>`,
);
put("website/public/og-image.svg", social);
put(
  "website/public/og-image.png",
  await sharp(Buffer.from(social)).png().toBuffer(),
);

put(
  "tools/brand-assets/manifest.json",
  JSON.stringify(
    {
      source,
      sourceSha256: createHash("sha256").update(artwork).digest("hex"),
      outputs: [...outputs].map(([path, data]) => ({
        path,
        sha256: createHash("sha256").update(data).digest("hex"),
      })),
    },
    null,
    2,
  ) + "\n",
);
let failures = 0;
for (const [path, content] of outputs) {
  const destination = resolve(root, path);
  if (check) {
    const existing = await readFile(destination).catch(() => null);
    if (!existing?.equals(content)) {
      console.error(`Outdated or missing: ${path}`);
      failures++;
    }
  } else {
    await mkdir(dirname(destination), { recursive: true });
    await writeFile(destination, content);
  }
}
if (failures) process.exitCode = 1;
console.log(
  `${check ? "Checked" : "Generated"} ${outputs.size} brand resources from ${relative(root, resolve(root, source))}`,
);
