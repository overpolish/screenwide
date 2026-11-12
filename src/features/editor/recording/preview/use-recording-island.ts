// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { type EditorIslandKind } from "../../island/host/editor-island-kind";
import { useEditorIslandWindow } from "../../island/host/use-editor-island-window";
import { useEditorWindowShortcuts } from "../../shortcuts/use-editor-window-shortcuts";

import type { StripSelection } from "../../timeline/strip/strip-selection";
import type { ToolPanelStripItem } from "../../tools/tool-panel-strip";

// The one island the recording editor shows over its picture, and which
// of three it is: the column's or title bar's (a pen, the chosen
// annotations or an open mode), the strip's (the one beat, lull, moment or
// range chosen there, or several beats and lulls gathered with Shift,
// standing over them at the picture's foot), or the Tracks island the
// transport's button brings out.
// Whichever was taken up last is out: taking one up lets go of the others,
// so Escape and Delete only ever reach the one in view. Shortcuts chosen
// on the strip are the Keyboard island's, which they open and which
// deletes them, so the two stay together.

export function useRecordingIsland({
  column,
  hasTracks,
  strip,
}: {
  column: {
    /** Select in hand with nothing chosen and no mode open. */
    backToSelect: () => void;
    island: { island: EditorIslandKind; tool: string } | null;
  };
  /** Whether the title bar offers Tracks, which its key opens. */
  hasTracks: boolean;
  strip: {
    island: ToolPanelStripItem | null;
    select: (next: StripSelection | null) => void;
    selection: StripSelection | null;
  };
}) {
  const [isTracksOpen, setIsTracksOpen] = useState(false);
  const stripKey = strip.island
    ? `${strip.island.kind}:${strip.island.id}`
    : null;
  const columnKey = column.island
    ? `${column.island.island}:${column.island.tool}`
    : null;
  const hasColumnIsland = columnKey !== null;
  // A strip choice or a column island newly taken up puts the Tracks island
  // away, in the same render, so the two are never out at once.
  const [seen, setSeen] = useState({ columnKey, stripKey });
  if (seen.columnKey !== columnKey || seen.stripKey !== stripKey) {
    setSeen({ columnKey, stripKey });
    if (
      (columnKey !== null && columnKey !== seen.columnKey) ||
      (stripKey !== null && stripKey !== seen.stripKey)
    )
      setIsTracksOpen(false);
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
  const putTracksAway = isTracksOpen
    ? () => {
        setIsTracksOpen(false);
      }
    : undefined;
  const openTracks = (isOpen: boolean) => {
    if (isOpen) {
      if (hasColumnIsland) column.backToSelect();
      strip.select(null);
    }
    setIsTracksOpen(isOpen);
  };
  useEditorWindowShortcuts({
    onDeselect: putTracksAway,
    // ⌘D reaches every choice at once, the open Tracks with the rest, as it
    // puts away an open mode.
    onDeselectAll: putTracksAway,
    onToggleTracks: hasTracks
      ? () => {
          openTracks(!isTracksOpen);
        }
      : undefined,
    // Out in front of everything else, Tracks is what Escape puts away.
    ownsEscape: isTracksOpen,
  });

  const shown: { anchorKey: string | null; island: EditorIslandKind } | null =
    strip.island
      ? { anchorKey: strip.island.id, island: strip.island.kind }
      : isTracksOpen
        ? { anchorKey: null, island: "tracks" }
        : column.island && { anchorKey: null, island: column.island.island };
  useEditorIslandWindow({
    anchorKey: shown?.anchorKey ?? null,
    island: shown?.island ?? null,
    workspace: "recording",
  });
  return {
    /** The title bar's Tracks button. */
    tracks: { isOpen: isTracksOpen, onOpenChange: openTracks },
  };
}
