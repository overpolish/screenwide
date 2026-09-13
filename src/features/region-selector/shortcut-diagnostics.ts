// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { ShortcutAction } from "../settings/types";

type Phase =
  | "listenerReady"
  | "listenerStopped"
  | "listenerFailed"
  | "received"
  | "duplicateIgnored"
  | "completed"
  | "failed";

function errorDescription(error: unknown): string | undefined {
  if (error === undefined) return undefined;
  if (error instanceof Error) return error.message.slice(0, 2000);
  if (typeof error === "string") return error.slice(0, 2000);
  try {
    const serialized: unknown = JSON.stringify(error);
    return typeof serialized === "string"
      ? serialized.slice(0, 2000)
      : "Unserializable shortcut error";
  } catch {
    return "Unserializable shortcut error";
  }
}

/** Best effort: a diagnostics failure must never stop a screenshot. */
export function reportShortcutDiagnostic(
  phase: Phase,
  action?: ShortcutAction,
  error?: unknown,
) {
  void invoke("report_shortcut_diagnostic", {
    action,
    error: errorDescription(error),
    phase,
  }).catch(() => undefined);
}
