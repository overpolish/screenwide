// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import { laidDocument } from "./format-document";
import { exportSizeOf, formatName } from "./format-name";
import { formatContext, resolvedTimelineEdit } from "./format-resolve";
import { formatsOfShape } from "./recording-formats";

import type { FormatDocument } from "./format-document";
import type { FrameFormats } from "../../../../bindings/FrameFormats";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

/** One file an export writes: a format's canvas and camera, its timeline
 * worked out from the shared edit, and the name it is written under; a
 * copy of the file written just before it where it would come out the
 * same. */
export type FormatSaveJob = FormatDocument & {
  fileStem: string;
  timelineEdit: RecordingTimelineEdit | null;
  isCopy?: boolean;
};

// A file name keeps no invisible direction marks: Fluent isolates each
// placeable in them, and they would stay in the name on disk.
const withoutIsolates = (text: string) => text.replace(/[\u2068\u2069]/gu, "");

/**
 * What an export writes: every format, shape by shape in the order the bar
 * shows them, each named after `fileStem` and its format, and sized as it
 * is published: a platform at its template's size, its shape's canvas laid
 * out on it (`laidDocument`), as a template chosen would size it; a
 * shape's own plain format at its canvas. With one format or `isAll` off,
 * the format in hand as it is shown, `edit`, under `fileStem`. `stored` is
 * the shared edit the others are worked out from, and `document` the shape
 * in hand's canvas and camera.
 */
export function formatSaveJobs({
  document,
  edit,
  fileStem,
  formats,
  isAll,
  stored,
}: {
  document: FormatDocument;
  edit: RecordingTimelineEdit | null;
  fileStem: string;
  formats: FrameFormats | null;
  isAll: boolean;
  stored: RecordingTimelineEdit | null;
}): FormatSaveJob[] {
  const ids = formats?.list.flatMap(formatsOfShape) ?? [];
  if (!formats || ids.length < 2 || !isAll)
    return [{ ...document, fileStem, timelineEdit: edit }];
  const canvas = document.recordingOutput.primary;
  // Formats of one shape that show the same, at one size, come out the
  // same: each is written right after the first, as a copy of it.
  const groups = new Map<string, FormatSaveJob[]>();
  for (const shape of formats.list) {
    const own = shape.id === formats.active ? document : (shape.look ?? null);
    if (!own) continue;
    for (const id of formatsOfShape(shape)) {
      const where = formatContext(formats, canvas, id);
      const timelineEdit =
        stored && resolvedTimelineEdit(stored, formats.edits, where);
      const preset = id === shape.id ? null : id;
      const size = exportSizeOf({ ...where.canvas, preset });
      const isResized =
        size.width !== where.canvas.width ||
        size.height !== where.canvas.height;
      const sameness = `${shape.id}:${String(size.width)}x${String(size.height)}:${JSON.stringify(timelineEdit)}`;
      const group = groups.get(sameness) ?? [];
      // A shape made for no platform is named by its size in a file name:
      // its proportions' colon is not allowed in one.
      const name =
        preset === null
          ? t("editor-frame-format-size", where.canvas)
          : formatName({ ...where.canvas, preset });
      group.push({
        ...(isResized
          ? laidDocument(own, size, own.recordingOutput.primary.clearArea)
          : own),
        fileStem: withoutIsolates(
          t("editor-export-format-file", { format: name, stem: fileStem }),
        ),
        isCopy: group.length > 0,
        timelineEdit,
      });
      groups.set(sameness, group);
    }
  }
  return [...groups.values()].flat();
}
