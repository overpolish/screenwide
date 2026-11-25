// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { useFrameFormats } from "../../recording/formats/frame-formats-context";
import { screenshotAnnotationOutput } from "../screenshot-preview-events";

import { withSizeScale } from "./screenshot-format-resolve";
import {
  drawnWorkspace,
  type LeftOut,
  withLeftOutKept,
} from "./screenshot-show-in";

import type { ScreenshotWorkspaceOutputSettings } from "../screenshot-output";
import type { ScreenshotAnnotationChangeEvent } from "../use-screenshot-preview-surface";

type Workspace = ScreenshotWorkspaceOutputSettings;

/** `event`, which the preview made without what the format leaves out,
 * with that kept. */
const keptIn = (
  event: ScreenshotAnnotationChangeEvent,
  { leftOut, workspace }: { leftOut?: LeftOut; workspace?: Workspace },
): ScreenshotAnnotationChangeEvent => {
  const item = workspace?.items[event.paneIndex];
  if (!item || !leftOut) return event;
  const previous = item.output.annotations;
  // Nothing is drawn on a layer left out, so nothing there changes.
  if (leftOut.layers.has(item.id)) return { ...event, annotations: previous };
  return {
    ...event,
    annotations: withLeftOutKept(event.annotations, {
      leftOut: leftOut.annotations,
      previous,
    }),
  };
};

/**
 * The screenshot's workspace as the preview draws it in the format in hand
 * (`drawnWorkspace`), and what keeps the window's own whole: a layer the
 * format leaves out is never pressed, and an annotation change from the
 * preview, which never held what the format leaves out, keeps it.
 */
export function useLeftOut(workspace: Workspace | undefined) {
  const snapshot = useFrameFormats()?.snapshot;
  const leftOut = snapshot?.leftOut;
  const sizeScale = snapshot?.sizeScale ?? 1;
  const drawn = useMemo(
    () =>
      workspace &&
      withSizeScale(
        leftOut ? drawnWorkspace(workspace, leftOut) : workspace,
        sizeScale,
      ),
    [leftOut, sizeScale, workspace],
  );
  return {
    /** The layer output the change `event` makes, with what the format
     * leaves out kept. */
    changed: (event: ScreenshotAnnotationChangeEvent) =>
      screenshotAnnotationOutput(
        workspace,
        keptIn(event, { leftOut, workspace }),
      ),
    drawn,
    /** `items` with the layers the format leaves out never pressed, as a
     * blank canvas's own layer is not. */
    pressable: <Item extends { id: number; isBlank?: boolean }>(
      items: readonly Item[],
    ) =>
      leftOut?.layers.size
        ? items.map((item) =>
            leftOut.layers.has(item.id) ? { ...item, isBlank: true } : item,
          )
        : [...items],
  };
}
