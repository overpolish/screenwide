// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Runs every Rust test in a process of its own, so a test that crashes its
// process cannot hide the tests after it, and lists each one that fails. For
// finding the tests a machine without a GPU cannot run.

import { execFile, execFileSync } from "node:child_process";
import { appendFileSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { availableParallelism, tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { env } from "node:process";

const manifest = resolve("src-tauri", "Cargo.toml");
const timeoutMs = 300_000;
const outputTail = 4_000;
// Run outside Cargo, the binding tests miss `.cargo/config.toml`'s `[env]`
// and would write into each crate's folder; give them a scratch folder.
const bindings = mkdtempSync(join(tmpdir(), "screenwide-survey-bindings-"));
const testEnv = {
  ...env,
  RUST_BACKTRACE: "0",
  TS_RS_EXPORT_DIR: bindings,
  TS_RS_LARGE_INT: "number",
};

const messages = execFileSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    manifest,
    "--workspace",
    "--no-run",
    "--message-format=json-render-diagnostics",
  ],
  {
    encoding: "utf8",
    maxBuffer: 1 << 30,
    stdio: ["ignore", "pipe", "inherit"],
  },
)
  .split("\n")
  .filter((line) => line.startsWith("{"))
  .map((line) => JSON.parse(line));

const binaries = messages
  .filter(
    (message) =>
      message.reason === "compiler-artifact" &&
      message.profile?.test &&
      message.executable,
  )
  .map((message) => ({
    executable: message.executable,
    directory: dirname(message.manifest_path),
    target: `${message.target.name} (${message.target.kind.join(", ")})`,
  }));

const list = (binary, ignored) =>
  execFileSync(
    binary.executable,
    ["--list", "--format", "terse", ...(ignored ? ["--ignored"] : [])],
    { cwd: binary.directory, encoding: "utf8", maxBuffer: 1 << 26 },
  )
    .split("\n")
    .filter((line) => line.endsWith(": test"))
    .map((line) => line.slice(0, -": test".length));

const jobs = binaries.flatMap((binary) => {
  const ignored = new Set(list(binary, true));
  return list(binary, false)
    .filter((name) => !ignored.has(name))
    .map((name) => ({ binary, name }));
});
console.log(
  `Running ${jobs.length} tests from ${binaries.length} test programs one at a time`,
);

const run = ({ binary, name }) =>
  new Promise((done) => {
    execFile(
      binary.executable,
      ["--exact", name, "--test-threads=1", "--nocapture"],
      {
        cwd: binary.directory,
        env: { ...testEnv, CARGO_MANIFEST_DIR: binary.directory },
        maxBuffer: 1 << 26,
        timeout: timeoutMs,
      },
      (error, stdout, stderr) => {
        if (!error) return done(null);
        const output = `${stdout}${stderr}`;
        done({
          target: binary.target,
          name,
          outcome: error.killed
            ? "timed out"
            : error.code === 101
              ? "failed"
              : "crashed",
          code: error.code ?? error.signal,
          output: output.slice(-outputTail),
        });
      },
    );
  });

const failures = [];
let next = 0;
let finished = 0;
await Promise.all(
  Array.from({ length: availableParallelism() }, async () => {
    while (next < jobs.length) {
      const failure = await run(jobs[next++]);
      if (failure) {
        failures.push(failure);
        console.log(`${failure.outcome}: ${failure.target} ${failure.name}`);
      }
      if (++finished % 250 === 0) console.log(`${finished}/${jobs.length}`);
    }
  }),
);
rmSync(bindings, { force: true, recursive: true });

failures.sort((a, b) =>
  `${a.target} ${a.name}`.localeCompare(`${b.target} ${b.name}`),
);
writeFileSync(
  "rust-test-survey.json",
  `${JSON.stringify(failures, null, 2)}\n`,
);

const summary = [
  `${failures.length} of ${jobs.length} tests did not pass`,
  "",
  "| Test program | Test | Outcome | Code |",
  "| --- | --- | --- | --- |",
  ...failures.map(
    (failure) =>
      `| ${failure.target} | \`${failure.name}\` | ${failure.outcome} | ${failure.code} |`,
  ),
].join("\n");
console.log(`\n${summary}`);
if (env.GITHUB_STEP_SUMMARY)
  appendFileSync(env.GITHUB_STEP_SUMMARY, `${summary}\n`);
