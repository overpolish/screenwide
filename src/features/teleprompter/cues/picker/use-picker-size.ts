// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { type RefObject, useLayoutEffect } from "react";

/** The picker's panel as it opens on the list, in points. */
export const PICKER_SIZE = { height: 440, width: 400 } as const;

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

/**
 * Sizes the picker's panel window: the list's height while choosing, and a
 * recording's moments as tall as they are, so the picture shows at its own
 * shape with no bands above and below it.
 */
export function usePickerSize(
  contentRef: RefObject<HTMLElement | null>,
  fitsContent: boolean,
) {
  useLayoutEffect(() => {
    if (!isTauri()) return;
    const window = getCurrentWindow();
    const content = contentRef.current;
    const size = (height: number) => {
      window
        .setSize(new LogicalSize(PICKER_SIZE.width, Math.ceil(height)))
        .catch(report("Could not size the cue picker"));
    };
    if (!fitsContent || !content) {
      size(PICKER_SIZE.height);
      return;
    }
    const observer = new ResizeObserver(() => {
      size(content.scrollHeight);
    });
    observer.observe(content);
    return () => {
      observer.disconnect();
    };
  }, [contentRef, fitsContent]);
}
