// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  sharedSpotlightBlurs,
  withSharedBlur,
} from "../annotations/spotlight-blur";

import type { ScreenshotWorkspaceOutputSettings } from "./screenshot-output";

/**
 * The workspace with every spotlight on every layer carrying one Blur
 * setting: a still shows them all together, under one shade. `previous` is
 * the workspace before the edit, which tells a switched setting from a
 * spotlight just drawn. A workspace that needs no change is handed back as
 * it was.
 */
export const withScreenshotSpotlightBlurShared = (
  workspace: ScreenshotWorkspaceOutputSettings,
  previous: ScreenshotWorkspaceOutputSettings,
): ScreenshotWorkspaceOutputSettings => {
  const spotlights = (settings: ScreenshotWorkspaceOutputSettings) =>
    settings.items.flatMap((item) =>
      item.output.annotations.filter(
        (annotation) => annotation.shape.kind === "spotlight",
      ),
    );
  const changes = sharedSpotlightBlurs(
    [spotlights(workspace)],
    new Map(spotlights(previous).map((spotlight) => [spotlight.id, spotlight])),
  );
  if (changes.size === 0) return workspace;
  return {
    ...workspace,
    items: workspace.items.map((item) =>
      item.output.annotations.some(({ id }) => changes.has(id))
        ? {
            ...item,
            output: {
              ...item.output,
              annotations: item.output.annotations.map((annotation) =>
                withSharedBlur(annotation, changes),
              ),
            },
          }
        : item,
    ),
  };
};
