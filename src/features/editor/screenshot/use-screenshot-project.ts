// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";

import {
  normalizedScreenshotWorkspaceOutput,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot-output";

/** Long enough that a drag or a run of nudges composes one preview, not many. */
const PREVIEW_SETTLE_MS = 1_000;

/**
 * Keeps the open screenshot's project in step with the editor: its canvas and
 * layers saved as they change, one save at a time and always the newest, so
 * the last change is written before the window can close; and its preview
 * composed once each change settles. Nothing is saved until the window has
 * seeded the screenshot, so the placeholder it starts with never lands in a
 * project.
 */
export function useScreenshotProject(
  screenshotId: number | undefined,
  workspace: ScreenshotWorkspaceOutputSettings,
) {
  const queueRef = useRef<{
    next: {
      artifactId: number;
      workspace: ScreenshotWorkspaceOutputSettings;
    } | null;
    saving: boolean;
  }>({ next: null, saving: false });

  useEffect(() => {
    if (screenshotId === undefined) return;
    const queue = queueRef.current;
    queue.next = { artifactId: screenshotId, workspace };
    const pump = () => {
      const next = queue.next;
      if (queue.saving || !next) return;
      queue.next = null;
      queue.saving = true;
      invoke<null>("set_screenshot_project_workspace", {
        artifactId: next.artifactId,
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
