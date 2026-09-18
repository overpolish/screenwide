// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  defaultScreenshotOutput,
  resetScreenshotLayout,
  ScreenshotOutputSettings,
  screenshotOutputTemplate,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot-output";
import { EditorArtifact } from "./types";

type ScreenshotArtifactItem = Extract<
  EditorArtifact,
  { kind: "screenshot" }
>["items"][number];

/**
 * A layer's first output: the look borrowed from the template, laid out for the
 * item's own size, carrying the annotations the item arrived with. Those are
 * the live overlay's, drawn over the shot when it was taken; the annotations on
 * the layers already in the workspace are never copied onto a new one.
 */
export const seedScreenshotItemOutput = (
  template: ScreenshotOutputSettings,
  item: ScreenshotArtifactItem,
  options: { fit?: boolean } = {},
): ScreenshotOutputSettings => ({
  ...resetScreenshotLayout(screenshotOutputTemplate(template), item, options),
  annotations: item.annotations,
});

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
