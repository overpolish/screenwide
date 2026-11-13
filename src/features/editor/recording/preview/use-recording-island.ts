// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { type EditorIslandKind } from "../../island/host/editor-island-kind";
import { useEditorIslandWindow } from "../../island/host/use-editor-island-window";
import { useEditorWindowShortcuts } from "../../shortcuts/use-editor-window-shortcuts";
import { usePublishEditorModes } from "../../title-bar/editor-title-modes";

import type { StripSelection } from "../../timeline/strip/strip-selection";
import type {
  CaptureParts,
  EditorKind,
  EditorMode,
} from "../../toolbar/editor-modes";
import type { ToolPanelStripItem } from "../../tools/tool-panel-strip";

// The one island the recording editor shows over its picture, and which
// of three it is: the column's or title bar's (a pen, the chosen
// annotations or an open mode), the strip's (the one beat, lull, moment or
// range chosen there, or several beats and lulls gathered with Shift,
// standing over them at the picture's foot), or one of the title bar's
// islands for the whole timeline, Tracks or the Library.
// Whichever was taken up last is out: taking one up lets go of the others,
// so Escape and Delete only ever reach the one in view. Shortcuts chosen
// on the strip are the Keyboard island's, which they open and which
// deletes them, so the two stay together. The title bar is told the modes
// and which of Tracks and the Library it offers.

export function useRecordingIsland({
  column,
  hasLibrary,
  hasTracks,
  kind,
  parts,
  strip,
}: {
  column: {
    /** Select in hand with nothing chosen and no mode open. */
    backToSelect: () => void;
    island: { island: EditorIslandKind; tool: string } | null;
    mode: EditorMode | null;
    onModeChange: (mode: EditorMode | null) => void;
  };
  /** Whether the title bar offers Library, which its key opens: songs and
   * sound effects can be laid under any recording. */
  hasLibrary: boolean;
  /** Whether the title bar offers Tracks, which its key opens. */
  hasTracks: boolean;
  kind: EditorKind;
  parts: CaptureParts;
  strip: {
    island: ToolPanelStripItem | null;
    select: (next: StripSelection | null) => void;
    selection: StripSelection | null;
  };
}) {
  const [aside, setAside] = useState<"library" | "tracks" | null>(null);
  const stripKey = strip.island
    ? `${strip.island.kind}:${strip.island.id}`
    : null;
  const columnKey = column.island
    ? `${column.island.island}:${column.island.tool}`
    : null;
  const hasColumnIsland = columnKey !== null;
  // A strip choice or a column island newly taken up puts Tracks or the
  // Library away, in the same render, so the two are never out at once.
  const [seen, setSeen] = useState({ columnKey, stripKey });
  if (seen.columnKey !== columnKey || seen.stripKey !== stripKey) {
    setSeen({ columnKey, stripKey });
    if (
      (columnKey !== null && columnKey !== seen.columnKey) ||
      (stripKey !== null && stripKey !== seen.stripKey)
    )
      setAside(null);
  }
  // Something chosen on the strip puts the column's island away; Select
  // stays in hand under it, as a beat has no tool.
  useEffect(() => {
    if (stripKey !== null && hasColumnIsland) column.backToSelect();
    // eslint-disable-next-line @eslint-react/exhaustive-deps -- only a new strip choice takes over; `column` is remade each render.
  }, [stripKey]);
  // A pen, an annotation or a mode taken up lets go of the strip's choice,
  // but for shortcuts under the Keyboard island they opened.
  const keepsShortcuts =
    column.island?.island === "keyboard" &&
    strip.selection?.kind === "shortcuts";
  useEffect(() => {
    if (columnKey !== null && !keepsShortcuts) strip.select(null);
    // eslint-disable-next-line @eslint-react/exhaustive-deps -- only a new column island takes over; `strip` is remade each render.
  }, [columnKey]);
  const putAsideAway =
    aside === null
      ? undefined
      : () => {
          setAside(null);
        };
  const openAside = (which: "library" | "tracks", isOpen: boolean) => {
    if (isOpen) {
      if (hasColumnIsland) column.backToSelect();
      strip.select(null);
    }
    setAside((open) => (isOpen ? which : open === which ? null : open));
  };
  const toggleOf = (which: "library" | "tracks") => ({
    isOpen: aside === which,
    onOpenChange: (isOpen: boolean) => {
      openAside(which, isOpen);
    },
  });
  useEditorWindowShortcuts({
    onDeselect: putAsideAway,
    // ⌘D reaches every choice at once, the open Tracks or Library with the
    // rest, as it puts away an open mode.
    onDeselectAll: putAsideAway,
    onToggleLibrary: hasLibrary
      ? () => {
          openAside("library", aside !== "library");
        }
      : undefined,
    onToggleTracks: hasTracks
      ? () => {
          openAside("tracks", aside !== "tracks");
        }
      : undefined,
    // Out in front of everything else, Tracks or the Library is what Escape
    // puts away.
    ownsEscape: aside !== null,
  });

  const shown: { anchorKey: string | null; island: EditorIslandKind } | null =
    strip.island
      ? { anchorKey: strip.island.id, island: strip.island.kind }
      : aside
        ? { anchorKey: null, island: aside }
        : column.island && { anchorKey: null, island: column.island.island };
  useEditorIslandWindow({
    anchorKey: shown?.anchorKey ?? null,
    island: shown?.island ?? null,
    workspace: "recording",
  });
  usePublishEditorModes({
    kind,
    library: hasLibrary ? toggleOf("library") : null,
    mode: column.mode,
    onModeChange: column.onModeChange,
    parts,
    tracks: hasTracks ? toggleOf("tracks") : null,
  });
}
