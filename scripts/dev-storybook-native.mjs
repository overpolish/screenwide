// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { spawn } from "node:child_process";
import process, { argv, env, exit, platform } from "node:process";

const DEFAULT_STORY = "primitives-alert--paragraph";
const STORYBOOK_ORIGIN = "http://localhost:6006";
const command = platform === "win32" ? "pnpm.cmd" : "pnpm";

const arguments_ = argv.slice(2);
if (arguments_[0] === "--") arguments_.shift();
let story = DEFAULT_STORY;
const tauriArguments = [];

for (let index = 0; index < arguments_.length; index += 1) {
  const argument = arguments_[index];

  if (argument === "--") {
    tauriArguments.push(...arguments_.slice(index + 1));
    break;
  }

  if (argument === "--help" || argument === "-h") {
    console.log(
      `Usage: pnpm dev:storybook-native -- [options]\n\nOptions:\n  --story <id>  Storybook story ID (default: ${DEFAULT_STORY})\n  --            Forward remaining arguments to tauri dev\n\nThe native preview follows the operating system theme live.`,
    );
    exit(0);
  }

  if (argument === "--story") {
    story = arguments_[index + 1] ?? "";
    index += 1;
    continue;
  }

  throw new Error(`Unknown argument: ${argument}`);
}

if (!story) throw new Error("--story requires a Storybook story ID");

const sleep = (milliseconds) =>
  new Promise((resolve) => setTimeout(resolve, milliseconds));

async function readStorybookIndex() {
  const response = await fetch(`${STORYBOOK_ORIGIN}/index.json`);
  if (!response.ok) {
    throw new Error(`Storybook returned HTTP ${response.status}`);
  }
  return response.json();
}

async function waitForStorybook() {
  const deadline = Date.now() + 30_000;
  let lastError;

  while (Date.now() < deadline) {
    try {
      return await readStorybookIndex();
    } catch (error) {
      lastError = error;
      await sleep(250);
    }
  }

  throw new Error("Storybook did not become ready within 30 seconds", {
    cause: lastError,
  });
}

let storybook;
try {
  await readStorybookIndex();
  console.log(`Reusing Storybook at ${STORYBOOK_ORIGIN}`);
} catch {
  console.log(`Starting Storybook at ${STORYBOOK_ORIGIN}`);
  storybook = spawn(
    command,
    ["exec", "storybook", "dev", "-p", "6006", "--no-open"],
    { env, stdio: "inherit" },
  );
}

const stopStorybook = () => {
  if (storybook != null && !storybook.killed) storybook.kill("SIGTERM");
};

process.on("SIGINT", stopStorybook);
process.on("SIGTERM", stopStorybook);

try {
  const index = await waitForStorybook();
  if (index.entries?.[story] == null) {
    throw new Error(`Storybook story not found: ${story}`);
  }

  const previewUrl = new URL("/iframe.html", STORYBOOK_ORIGIN);
  previewUrl.searchParams.set("id", story);
  previewUrl.searchParams.set("viewMode", "story");
  previewUrl.searchParams.set("screenwide-native", "1");

  console.log(`Opening native preview for ${story}`);
  const tauri = spawn(
    command,
    [
      "tauri",
      "dev",
      "--config",
      "src-tauri/tauri.storybook.conf.json",
      ...tauriArguments,
    ],
    {
      env: {
        ...env,
        SCREENWIDE_STORYBOOK_NATIVE_URL: previewUrl.toString(),
      },
      stdio: "inherit",
    },
  );

  const status = await new Promise((resolve, reject) => {
    tauri.once("error", reject);
    tauri.once("exit", (code) => resolve(code ?? 1));
  });
  process.exitCode = status;
} finally {
  stopStorybook();
}
