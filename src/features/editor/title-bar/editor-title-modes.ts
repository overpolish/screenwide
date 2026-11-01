// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useLayoutEffect, useRef } from "react";
import { create } from "zustand";

import type {
  CaptureParts,
  EditorKind,
  EditorMode,
} from "../toolbar/editor-modes";

/** A recording's Tracks button: whether its island is out, and told when
 * it is pressed. */
export type EditorTracksToggle = {
  isOpen: boolean;
  onOpenChange: (isOpen: boolean) => void;
};

/** The modes the open editor offers and the one open, and a recording's
 * Tracks button, for the title bar, which stands outside the preview that
 * owns them; null while no editor's preview is mounted. */
export type EditorTitleModes = {
  kind: EditorKind;
  mode: EditorMode | null;
  /** Told the mode chosen, or null when the open one is pressed again. */
  onModeChange: (mode: EditorMode | null) => void;
  parts: CaptureParts;
  tracks: EditorTracksToggle | null;
};

export const useEditorTitleModes = create<{
  modes: EditorTitleModes | null;
}>(() => ({ modes: null }));

/** Publishes the preview's modes and Tracks to the title bar while it is
 * mounted. */
export function usePublishEditorModes({
  kind,
  mode,
  onModeChange,
  parts,
  tracks,
}: EditorTitleModes) {
  const changeRef = useRef({ onModeChange, tracks });
  useLayoutEffect(() => {
    changeRef.current = { onModeChange, tracks };
  });
  const { audio, camera, cursor, keyboard, microphone, picture, transcript } =
    parts;
  const hasTracks = tracks !== null;
  const isTracksOpen = tracks?.isOpen ?? false;
  useEffect(() => {
    useEditorTitleModes.setState({
      modes: {
        kind,
        mode,
        onModeChange: (next) => {
          changeRef.current.onModeChange(next);
        },
        parts: {
          audio,
          camera,
          cursor,
          keyboard,
          microphone,
          picture,
          transcript,
        },
        tracks: hasTracks
          ? {
              isOpen: isTracksOpen,
              onOpenChange: (isOpen) => {
                changeRef.current.tracks?.onOpenChange(isOpen);
              },
            }
          : null,
      },
    });
  }, [
    audio,
    camera,
    cursor,
    hasTracks,
    isTracksOpen,
    keyboard,
    kind,
    microphone,
    mode,
    picture,
    transcript,
  ]);
  useEffect(
    () => () => {
      useEditorTitleModes.setState({ modes: null });
    },
    [],
  );
}
