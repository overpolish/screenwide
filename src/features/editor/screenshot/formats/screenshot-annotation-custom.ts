// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { proportionKey } from "../../recording/formats/format-scope";
import {
  isMainCanvas,
  mainDocument,
} from "../../recording/formats/recording-formats";

import {
  annotationIn,
  withAnnotation,
  withoutLayout,
  withOwnLayouts,
} from "./screenshot-annotation-layouts";
import { withScreenshotShownIn } from "./screenshot-format-sharing";
import { type InHand, settledFormats } from "./screenshot-formats";

import type { ScreenshotFormats } from "../../../../bindings/ScreenshotFormats";
import type { FormatSyncAction } from "../../recording/formats/format-sync";
import type { ScreenshotWorkspaceOutputSettings } from "../screenshot-output";

// What an annotation's right-click menu asks of its placement across the
// screenshot's formats, as a recording's does (`format-item-menu.ts`):
// custom on the platforms of the proportion in hand, or not; or its
// placement in hand made the one every shape uses. Each settles what was
// changed with the platform in hand first (`settledFormats`).

type Changed = {
  document: ScreenshotWorkspaceOutputSettings;
  formats: ScreenshotFormats;
};

/** The annotation `id` made custom on the platforms of the proportion in
 * hand, placed there as it is now; or, custom already, placed as the main
 * canvas places it again. Null on the main canvas. */
export function withScreenshotCustomToggled(
  formats: ScreenshotFormats,
  inHand: InHand,
  id: string,
): Changed | null {
  const settled = settledFormats(formats, inHand);
  const current = annotationIn(inHand.workspace, id);
  if (isMainCanvas(settled) || !current) return null;
  const key = proportionKey(inHand.workspace);
  if (Object.keys(settled.layouts?.[id] ?? {}).includes(key)) {
    const main = annotationIn(mainDocument(settled, inHand.workspace), id);
    return {
      document: main
        ? withAnnotation(inHand.workspace, main)
        : inHand.workspace,
      formats: {
        ...settled,
        layouts: withoutLayout(settled.layouts, { id, key }),
      },
    };
  }
  return {
    document: inHand.workspace,
    formats: {
      ...settled,
      layouts: withOwnLayouts(settled.layouts, { key, own: [current] }),
    },
  };
}

/** The annotation `id`'s placement in hand made the one every shape uses:
 * the main canvas's, with no proportion placing it its own way. Null where
 * the workspace in hand does not hold it. */
function withScreenshotLayoutEverywhere(
  formats: ScreenshotFormats,
  inHand: InHand,
  id: string,
): Changed | null {
  const settled = settledFormats(formats, inHand);
  const current = annotationIn(inHand.workspace, id);
  if (!current) return null;
  const layouts = withoutLayout(settled.layouts, { id, key: null });
  const [main, ...platforms] = settled.list;
  return {
    document: inHand.workspace,
    formats: {
      ...settled,
      layouts,
      list:
        isMainCanvas(settled) || !main.look
          ? settled.list
          : [
              { ...main, look: withAnnotation(main.look, current) },
              ...platforms,
            ],
    },
  };
}

/** What the pick `action` in the menu of the layer or annotation `id`
 * does (`screenshotFormatSync`). */
export const screenshotSynced = (
  formats: ScreenshotFormats,
  inHand: InHand,
  { action, id }: { action: FormatSyncAction; id: string },
): Changed | null => {
  switch (action.kind) {
    case "show":
      return withScreenshotShownIn(formats, inHand, {
        format: action.format,
        syncId: id,
      });
    case "custom":
      return withScreenshotCustomToggled(formats, inHand, id);
    case "layout-all":
      return withScreenshotLayoutEverywhere(formats, inHand, id);
  }
};
