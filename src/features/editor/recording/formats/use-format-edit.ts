// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useMemo } from "react";

import { formatDifferences, NO_DIFFERENCES } from "./format-differences";
import { formatContext, resolvedTimelineEdit } from "./format-resolve";
import { everyLayoutKey, layoutKeys } from "./format-scope";
import {
  type FormatSync,
  formatSync,
  type FormatSyncAction,
  withCustomToggled,
  withShownToggled,
} from "./format-sync";
import { formatVersions } from "./format-versions";
import { writtenTimelineEdit } from "./format-write";
import { withoutItemLayouts } from "./format-write-parts";
import { formatsOfShape } from "./recording-formats";

import type { EditScope } from "./format-scope";
import type { FrameFormats } from "../../../../bindings/FrameFormats";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

/**
 * The timeline the editor shows and plays for the format in hand, worked
 * out from the one edit every format shares, and the way back: a change to
 * it written into that edit and into what the formats do differently, where
 * `scope` says. A recording made in one shape is its edit as it is.
 */
export function useFormatEdit({
  canvas,
  commit,
  edit,
  formats,
  onFormatsChange,
  scope,
}: {
  canvas: { height: number; width: number };
  commit: (edit: RecordingTimelineEdit) => void;
  edit: RecordingTimelineEdit | null;
  formats: FrameFormats | null;
  onFormatsChange: (formats: FrameFormats) => void;
  scope: EditScope;
}) {
  const { height, width } = canvas;
  const resolved = useMemo(() => {
    if (!edit || !formats)
      return {
        context: null,
        differs: NO_DIFFERENCES,
        edit,
        versions: [],
        where: null,
      };
    const where = formatContext(formats, { height, width });
    return {
      context: where.context,
      differs: formatDifferences(edit, formats.edits, where.context),
      edit: resolvedTimelineEdit(edit, formats.edits, where),
      versions: formatVersions({
        contextOf: (id) =>
          formatContext(formats, { height, width }, id).context,
        edit,
        edits: formats.edits ?? {},
        formats: formats.list.flatMap(formatsOfShape),
        isPlaced: (shape) =>
          formats.list.some((each) => each.id === shape && each.placed),
      }),
      where,
    };
  }, [edit, formats, height, width]);
  const commitResolved = useCallback(
    (next: RecordingTimelineEdit) => {
      if (!edit || !formats || !resolved.context) {
        commit(next);
        return;
      }
      const written = writtenTimelineEdit({
        context: resolved.context,
        edits: formats.edits ?? {},
        next,
        previous: resolved.edit,
        scope,
        stored: edit,
      });
      commit(written.edit);
      onFormatsChange({ ...formats, edits: written.edits });
    },
    [commit, edit, formats, onFormatsChange, resolved, scope],
  );
  const edits = formats?.edits;
  /** `id`'s layout in this shape made the one every shape uses: written as
   * a change from the layout it would share to its own, made for the
   * shared layout, and every shape's own taken away. */
  const spreadLayout = (id: string) => {
    const { context, edit: next, where } = resolved;
    if (!context || !edit || !formats) return;
    const plain = withoutItemLayouts(edits ?? {}, id, layoutKeys(context));
    const written = writtenTimelineEdit({
      context,
      edits: plain,
      next,
      previous: resolvedTimelineEdit(edit, plain, where),
      scope: "all",
      stored: edit,
    });
    commit(written.edit);
    onFormatsChange({
      ...formats,
      edits: withoutItemLayouts(written.edits, id, everyLayoutKey(context)),
    });
  };
  return {
    commit: commitResolved,
    differs: resolved.differs,
    edit: resolved.edit,
    sync: (action: FormatSyncAction, id: string) => {
      const { context, edit: shown } = resolved;
      if (!context || !edit || !formats) return;
      const own = edits ?? {};
      if (action.kind === "layout-all") spreadLayout(id);
      else
        onFormatsChange({
          ...formats,
          edits:
            action.kind === "custom"
              ? withCustomToggled({ context, edits: own, id, shown })
              : withShownToggled(action.format, {
                  context,
                  edit,
                  edits: own,
                  id,
                }),
        });
    },
    /** What `id` offers in its menu to set it apart by format. */
    syncOf: (id: string): FormatSync | null =>
      resolved.context && edit
        ? formatSync({
            context: resolved.context,
            edit,
            edits: edits ?? {},
            id,
          })
        : null,
    versions: resolved.versions,
  };
}
