// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Draft release notes from the Conventional Commits since the last release.
// Work lands on main without pull requests, so GitHub's generated notes would
// hold only the compare link. The draft is edited by hand before publishing;
// the app shows the published notes and the Microsoft Store takes them as
// What's new.
//
// Usage: node scripts/release-notes.mjs <tag>

import { execFileSync } from "node:child_process";
import process from "node:process";

const SECTIONS = [
  { type: "feat", heading: "New" },
  { type: "fix", heading: "Fixed" },
];
// Scopes that change only how Screenwide is built or released.
const INTERNAL_SCOPES = new Set(["ci", "dev", "release"]);
const SUBJECT = /^(\w+)(?:\(([^)]+)\))?!?: (.+)$/;
const CLOSING = /^(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?) +#(\d+)\s*$/gim;

const git = (...args) => execFileSync("git", args, { encoding: "utf8" });

const tag = process.argv[2];
if (!tag) {
  console.error("Usage: node scripts/release-notes.mjs <tag>");
  process.exit(1);
}

// A release is compared with the last release, so its notes cover any
// prereleases between; a prerelease is compared with the tag before it.
const previousTag = (() => {
  const args = ["describe", "--tags", "--abbrev=0", "--match", "v*"];
  if (!tag.includes("-")) args.push("--exclude", "v*-*");
  try {
    return git(...args, `${tag}^`).trim();
  } catch {
    return null;
  }
})();

const log = git(
  "log",
  "--no-merges",
  "--format=%s%x1f%b%x1e",
  previousTag ? `${previousTag}..${tag}` : tag,
);

const entries = new Map(SECTIONS.map(({ type }) => [type, []]));
for (const record of log.split("\x1e")) {
  const [subject = "", body = ""] = record.trim().split("\x1f");
  const match = SUBJECT.exec(subject.trim());
  if (!match) continue;
  const [, type, scope, summary] = match;
  if (!entries.has(type) || INTERNAL_SCOPES.has(scope)) continue;
  const area = scope
    ? `${scope.charAt(0).toUpperCase()}${scope.slice(1).replaceAll("-", " ")}: `
    : "";
  const issues = [...body.matchAll(CLOSING)].map(([, number]) => `#${number}`);
  const refs = issues.length > 0 ? ` (${issues.join(", ")})` : "";
  entries.get(type).push(`- ${area}${summary}${refs}`);
}

const parts = SECTIONS.filter(({ type }) => entries.get(type).length > 0).map(
  ({ type, heading }) =>
    `## ${heading}\n\n${entries.get(type).toReversed().join("\n")}`,
);
if (previousTag) {
  const repository = process.env.GITHUB_REPOSITORY ?? "overpolish/screenwide";
  parts.push(
    `**Full Changelog**: https://github.com/${repository}/compare/${previousTag}...${tag}`,
  );
}
process.stdout.write(`${parts.join("\n\n")}\n`);
