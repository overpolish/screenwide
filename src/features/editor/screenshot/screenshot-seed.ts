// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { EditorArtifact } from "../types";

import { blankLayerOutput } from "./screenshot-blank";
import {
  defaultScreenshotOutput,
  resetScreenshotLayout,
  ScreenshotOutputSettings,
  screenshotOutputTemplate,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot-output";

type ScreenshotArtifactItem = Extract<
  EditorArtifact,
  { kind: "screenshot" }
>["items"][number];

/**
 * A layer's first output: the look borrowed from the template, laid out for the
 * item's own size, carrying the annotations the item arrived with. Those are
 * the live overlay's, drawn over the shot when it was taken; the annotations on
 * the layers already in the workspace are never copied onto a new one. A blank
 * layer takes none of the look, so nothing of it shows.
 */
export const seedScreenshotItemOutput = (
  template: ScreenshotOutputSettings,
  item: ScreenshotArtifactItem,
  options: { fit?: boolean } = {},
): ScreenshotOutputSettings => {
  const output = {
    ...resetScreenshotLayout(screenshotOutputTemplate(template), item, options),
    annotations: item.annotations,
  };
  return item.isBlank ? blankLayerOutput(output) : output;
};

/**
 * The workspace a screenshot artifact opens with. The remembered look is a
 * template, so it brings the colours and the corners and none of the
 * annotations, and a new capture starts at its own native canvas dimensions
 * rather than the previous artifact's aspect ratio or manually enlarged canvas.
 */
export const seedScreenshotWorkspace = ({
  artifact,
  backgroundRadius,
  persisted,
  radius,
}: {
  artifact: EditorArtifact | null;
  backgroundRadius: number;
  persisted: ScreenshotOutputSettings | null;
  radius: number;
}): ScreenshotWorkspaceOutputSettings => {
  const defaults = defaultScreenshotOutput(
    artifact?.width ?? 1,
    artifact?.height ?? 1,
    { background: backgroundRadius, screenshot: radius },
  );
  if (artifact?.kind !== "screenshot") return { ...defaults, items: [] };
  const first = persisted
    ? resetScreenshotLayout(
        {
          ...defaults,
          ...screenshotOutputTemplate(persisted),
          backgroundRadiusPercent: backgroundRadius,
          height: defaults.height,
          radiusPercent: radius,
          recenterInsetColor: null,
          width: defaults.width,
        },
        artifact,
      )
    : resetScreenshotLayout(defaults, artifact);
  return {
    ...first,
    items: artifact.items.map((item) => ({
      id: item.id,
      output: seedScreenshotItemOutput(first, item),
    })),
  };
};

/**
 * A screenshot project's own saved workspace, as it was left: the canvas and
 * each saved layer stand. A layer taken since the last save has none, and is
 * placed as a layer added to an open editor is, at its real size.
 */
export const reopenedScreenshotWorkspace = (
  artifact: Extract<EditorArtifact, { kind: "screenshot" }>,
  saved: ScreenshotWorkspaceOutputSettings,
): ScreenshotWorkspaceOutputSettings => {
  const { items: savedItems, ...canvas } = saved;
  const workspace = {
    ...defaultScreenshotOutput(artifact.width, artifact.height),
    ...canvas,
    recenterInsetColor: null,
  };
  return {
    ...workspace,
    // A layer deleted before it was saved stays deleted.
    items: artifact.items
      .filter((item) => !artifact.deletedItemIds.includes(item.id))
      .map(
        (item) =>
          savedItems.find((savedItem) => savedItem.id === item.id) ?? {
            id: item.id,
            output: seedScreenshotItemOutput(workspace, item, { fit: false }),
          },
      ),
  };
};
