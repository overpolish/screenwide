// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Writes the list the sticker picker shows and searches: every emoji in its
// default skin tone, in Unicode's groups and order, with its name and CLDR's
// keywords. The emoji themselves are drawn by the system's own emoji font,
// so only what that font can draw is listed: Emoji 15.0, which macOS 14 and
// Windows 11 both carry. Run against Unicode's emoji-test.txt and CLDR's
// English annotations; the output is committed, so a build never reaches the
// network.
//
//   pnpm stickers:prepare <emoji-test.txt> <annotations.json> <annotationsDerived.json>
//
// https://unicode.org/Public/emoji/latest/emoji-test.txt
// https://github.com/unicode-org/cldr-json: cldr-annotations-full/annotations/en/annotations.json
// and cldr-annotations-derived-full/annotationsDerived/en/annotations.json

import { readFileSync, writeFileSync } from "node:fs";
import process from "node:process";

const [order, annotationsFile, derivedFile] = process.argv.slice(2);
if (!order || !annotationsFile || !derivedFile) {
  console.error(
    "Usage: pnpm stickers:prepare <emoji-test.txt> <annotations.json> <annotationsDerived.json>",
  );
  process.exit(1);
}

const OUT = "src/features/editor/stickers/emoji.json";
// The newest emoji the system fonts the app runs on can all draw.
const NEWEST = 15.0;
// The skin tone modifiers: the picker lists each emoji in its default tone.
const TONES = /[\u{1F3FB}-\u{1F3FF}]/u;

const keywords = new Map();
for (const [file, root] of [
  [annotationsFile, "annotations"],
  [derivedFile, "annotationsDerived"],
]) {
  const entries = JSON.parse(readFileSync(file, "utf8"))[root].annotations;
  for (const [emoji, { default: words = [] }] of Object.entries(entries))
    keywords.set(emoji.replace(/\uFE0F/g, ""), words);
}

const groups = [];
let group = null;
for (const line of readFileSync(order, "utf8").split("\n")) {
  const heading = /^# group: (.+)$/.exec(line);
  if (heading) {
    // Unicode's own component group holds the skin tones and hair styles,
    // which are no emoji to place on their own.
    group =
      heading[1] === "Component" ? null : { emoji: [], group: heading[1] };
    if (group) groups.push(group);
    continue;
  }
  const entry = /; fully-qualified\s+# (\S+) E(\d+\.\d+) (.+)$/.exec(line);
  if (!group || !entry) continue;
  const [, emoji, version, name] = entry;
  if (Number(version) > NEWEST || TONES.test(emoji)) continue;
  const words = (keywords.get(emoji.replace(/\uFE0F/g, "")) ?? [])
    .map((word) => word.toLowerCase())
    .filter((word) => word !== name && !name.includes(word));
  group.emoji.push({ emoji, keywords: [...new Set(words)], name });
}

writeFileSync(
  OUT,
  `${JSON.stringify(groups.filter(({ emoji }) => emoji.length > 0))}\n`,
);
console.log(
  `Wrote ${String(groups.reduce((sum, { emoji }) => sum + emoji.length, 0))} emoji.`,
);
