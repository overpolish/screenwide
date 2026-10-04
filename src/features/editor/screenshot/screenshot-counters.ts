// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { renumberedCounterLists } from "../annotations/annotation-counters";

import type { ScreenshotWorkspaceOutputSettings } from "./screenshot-output";

/**
 * The workspace with its counters numbered as one run across every layer, so
 * a counter's number is the order it was placed in the whole picture rather
 * than in its own layer. A fresh counter is numbered after every other one
 * already placed, and a delete anywhere closes the gap everywhere. A
 * workspace whose numbers already run is handed back as it was.
 */
export const withScreenshotCountersNumbered = (
  workspace: ScreenshotWorkspaceOutputSettings,
): ScreenshotWorkspaceOutputSettings => {
  const lists = renumberedCounterLists(
    workspace.items.map((item) => item.output.annotations),
  );
  if (
    lists.every((list, at) => list === workspace.items[at].output.annotations)
  )
    return workspace;
  return {
    ...workspace,
    items: workspace.items.map((item, at) =>
      lists[at] === item.output.annotations
        ? item
        : { ...item, output: { ...item.output, annotations: lists[at] } },
    ),
  };
};
