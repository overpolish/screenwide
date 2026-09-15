// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { ShortcutAction } from "../features/settings/types";

type Phase =
  | "listenerReady"
  | "listenerStopped"
  | "listenerFailed"
  | "received"
  | "duplicateIgnored"
  | "completed"
  | "failed";

// Diagnostics probe, added to chase one specific bug: a single press producing
// two `received` entries in the same millisecond from one dispatch. `contextId`
// is minted once per JS context, so every window that loads this module has its
// own; `listenerId` is minted per `listen()` subscription. Together they tell
// the three candidate causes apart, and `liveListeners` says how many
// subscriptions this context is holding at the moment of the report. Remove all
// three, and `trackShortcutListener`, once the duplicate is understood.
const contextId = `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
let nextListenerId = 0;
let liveListeners = 0;

export interface ShortcutListenerHandle {
  /** Distinguishes this subscription from every other one in this context. */
  listenerId: number;
  /** Idempotent: drops this subscription out of the live count. */
  release: () => void;
}

/**
 * Claims an identity for one `listen()` subscription.
 *
 * Call it where the subscription is requested, not where it resolves, so the
 * live count covers the window in which events can already arrive.
 */
export function trackShortcutListener(): ShortcutListenerHandle {
  nextListenerId += 1;
  liveListeners += 1;
  const listenerId = nextListenerId;
  let released = false;
  return {
    listenerId,
    release: () => {
      if (released) return;
      released = true;
      liveListeners -= 1;
    },
  };
}

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

interface ShortcutDiagnosticDetails {
  action?: ShortcutAction;
  error?: unknown;
  listenerId?: number;
}

/** Best effort: a diagnostics failure must never stop a shortcut. */
export function reportShortcutDiagnostic(
  phase: Phase,
  details: ShortcutDiagnosticDetails = {},
) {
  void invoke("report_shortcut_diagnostic", {
    action: details.action,
    contextId,
    error: errorDescription(details.error),
    listenerId: details.listenerId,
    // Read at report time, so a `listenerStopped` reported after `release`
    // shows how many subscriptions this context still holds.
    liveListeners,
    phase,
  }).catch(() => undefined);
}
