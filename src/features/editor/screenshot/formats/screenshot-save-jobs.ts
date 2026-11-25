// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { formatName } from "../../recording/formats/format-name";
import { formatFileStem } from "../../recording/formats/format-save-jobs";
import {
  formatsOfShape,
  mainDocument,
} from "../../recording/formats/recording-formats";

import {
  shownWorkspace,
  sizeScaleOf,
  withSizeScale,
} from "./screenshot-format-resolve";
import { type InHand, settledFormats } from "./screenshot-formats";
import { leftOutOfAll, publishedWorkspace } from "./screenshot-show-in";

import type { ScreenshotFormats } from "../../../../bindings/ScreenshotFormats";
import type { ScreenshotShape } from "../../../../bindings/ScreenshotShape";
import type { ScreenshotWorkspaceOutputSettings } from "../screenshot-output";

type Workspace = ScreenshotWorkspaceOutputSettings;

/** One file an export of every format of a screenshot writes: the
 * workspace its format is published as, and the name it is written under;
 * a copy of the file written just before it where it comes out the same. */
export type ScreenshotSaveJob = {
  fileStem: string;
  workspace: Workspace;
  isCopy?: boolean;
};

/** The workspace `shape` shows, of the main canvas `main`, with `inHand`
 * the one the window shows of the shape in hand. */
const workspaceOf = (
  formats: ScreenshotFormats,
  {
    inHand,
    main,
    shape,
  }: { inHand: InHand; main: Workspace; shape: ScreenshotShape },
) => {
  if (shape.id === formats.active) return inHand.workspace;
  return shownWorkspace(formats, { blank: inHand.blank, main, shape });
};

/**
 * What an export of every format of a screenshot writes: each format, shape
 * by shape in the order the bar shows them, named after `fileStem` and its
 * format, drawn as it is published, without what it leaves out. Formats
 * that come out the same are written one after another, each after the
 * first a copy of its file. Null with one format.
 */
export function screenshotSaveJobs({
  fileStem,
  formats,
  inHand,
}: {
  fileStem: string;
  formats: ScreenshotFormats | null;
  inHand: InHand;
}): ScreenshotSaveJob[] | null {
  if (!formats || formats.list.flatMap(formatsOfShape).length < 2) return null;
  const settled = settledFormats(formats, inHand);
  const main = mainDocument(settled, inHand.workspace);
  const groups = new Map<string, ScreenshotSaveJob[]>();
  for (const shape of settled.list) {
    const shown = workspaceOf(settled, { inHand, main, shape });
    const scale = sizeScaleOf(settled, { blank: inHand.blank, main, shape });
    for (const id of formatsOfShape(shape)) {
      const workspace = withSizeScale(
        publishedWorkspace(shown, leftOutOfAll(settled, [id])),
        scale,
      );
      const sameness = JSON.stringify(workspace);
      const group = groups.get(sameness) ?? [];
      // The main canvas is named by its size: its proportions' colon is
      // not allowed in a file name.
      const name =
        id === shape.id
          ? t("editor-frame-format-size", {
              height: workspace.height,
              width: workspace.width,
            })
          : formatName({ ...workspace, preset: id });
      group.push({
        fileStem: formatFileStem(name, fileStem),
        isCopy: group.length > 0,
        workspace,
      });
      groups.set(sameness, group);
    }
  }
  return [...groups.values()].flat();
}
