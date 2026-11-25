// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";

import {
  normalizedScreenshotWorkspaceOutput,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot-output";

import type { ScreenshotFormats } from "../../../bindings/ScreenshotFormats";

/** Long enough that a drag or a run of nudges composes one preview, not many. */
const PREVIEW_SETTLE_MS = 1_000;

/**
 * Keeps the open screenshot's project in step with the editor: its canvas,
 * layers and formats saved as they change, one save at a time and always
 * the newest, so the last change is written before the window can close;
 * and its preview composed once each change settles. Nothing is saved until
 * the window has seeded the screenshot, so the placeholder it starts with
 * never lands in a project. Every save after the one that lays a screenshot
 * out is an edit, which a screenshot not yet in Projects asks about before
 * it closes.
 */
export function useScreenshotProject(
  screenshotId: number | undefined,
  workspace: ScreenshotWorkspaceOutputSettings,
  formats: ScreenshotFormats | null,
) {
  const queueRef = useRef<{
    next: {
      artifactId: number;
      edited: boolean;
      formats: ScreenshotFormats | null;
      workspace: ScreenshotWorkspaceOutputSettings;
    } | null;
    saving: boolean;
  }>({ next: null, saving: false });
  const laidOutRef = useRef<{
    formats: ScreenshotFormats | null;
    id: number;
    workspace: ScreenshotWorkspaceOutputSettings;
  } | null>(null);

  useEffect(() => {
    if (screenshotId === undefined) return;
    // Compared by the workspace and formats too, so an effect run again
    // with nothing changed (StrictMode's remount) is not taken for an edit.
    const laidOut = laidOutRef.current;
    const edited =
      laidOut?.id === screenshotId &&
      (laidOut.workspace !== workspace || laidOut.formats !== formats);
    if (laidOut?.id !== screenshotId) {
      laidOutRef.current = { formats, id: screenshotId, workspace };
    }
    const queue = queueRef.current;
    queue.next = {
      artifactId: screenshotId,
      edited:
        edited ||
        (queue.next?.artifactId === screenshotId && queue.next.edited),
      formats,
      workspace,
    };
    const pump = () => {
      const next = queue.next;
      if (queue.saving || !next) return;
      queue.next = null;
      queue.saving = true;
      invoke<null>("set_screenshot_project_workspace", {
        artifactId: next.artifactId,
        edited: next.edited,
        formats: next.formats && {
          ...next.formats,
          list: next.formats.list.map((shape) =>
            shape.look
              ? {
                  ...shape,
                  look: normalizedScreenshotWorkspaceOutput(shape.look),
                }
              : shape,
          ),
        },
        // Time orders the saves: Rust keeps whichever is newest.
        revision: Date.now(),
        workspace: normalizedScreenshotWorkspaceOutput(next.workspace),
      })
        .catch((cause: unknown) => {
          console.error("Could not save the screenshot project", cause);
        })
        .finally(() => {
          queue.saving = false;
          pump();
        });
    };
    pump();
  }, [screenshotId, workspace, formats]);

  useEffect(() => {
    if (screenshotId === undefined) return;
    const timer = window.setTimeout(() => {
      // A preview is a convenience: one that cannot be made now is made at
      // the next change, and the browser shows the first picture meanwhile.
      invoke<null>("save_screenshot_project_still", {
        artifactId: screenshotId,
        screenshotOutput: normalizedScreenshotWorkspaceOutput(workspace),
      }).catch(() => undefined);
    }, PREVIEW_SETTLE_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [screenshotId, workspace]);
}
