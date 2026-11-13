// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useLayoutEffect, useRef } from "react";
import { create } from "zustand";

import type {
  CaptureParts,
  EditorKind,
  EditorMode,
} from "../toolbar/editor-modes";

/** A button beside the modes for an island of its own (Tracks, the
 * Library): whether its island is out, and told when it is pressed. */
export type EditorIslandToggle = {
  isOpen: boolean;
  onOpenChange: (isOpen: boolean) => void;
};

/** The modes the open editor offers and the one open, and a recording's
 * Tracks and Library buttons, for the title bar, which stands outside the
 * preview that owns them; null while no editor's preview is mounted. */
export type EditorTitleModes = {
  kind: EditorKind;
  /** The Library button, null where there is no library to add from. */
  library: EditorIslandToggle | null;
  mode: EditorMode | null;
  /** Told the mode chosen, or null when the open one is pressed again. */
  onModeChange: (mode: EditorMode | null) => void;
  parts: CaptureParts;
  tracks: EditorIslandToggle | null;
};

export const useEditorTitleModes = create<{
  modes: EditorTitleModes | null;
}>(() => ({ modes: null }));

/** Publishes the preview's modes, Tracks and Library to the title bar while
 * it is mounted. */
export function usePublishEditorModes({
  kind,
  library,
  mode,
  onModeChange,
  parts,
  tracks,
}: EditorTitleModes) {
  const changeRef = useRef({ library, onModeChange, tracks });
  useLayoutEffect(() => {
    changeRef.current = { library, onModeChange, tracks };
  });
  const { audio, camera, cursor, keyboard, microphone, picture, transcript } =
    parts;
  const hasTracks = tracks !== null;
  const isTracksOpen = tracks?.isOpen ?? false;
  const hasLibrary = library !== null;
  const isLibraryOpen = library?.isOpen ?? false;
  useEffect(() => {
    useEditorTitleModes.setState({
      modes: {
        kind,
        library: hasLibrary
          ? {
              isOpen: isLibraryOpen,
              onOpenChange: (isOpen) => {
                changeRef.current.library?.onOpenChange(isOpen);
              },
            }
          : null,
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
    hasLibrary,
    hasTracks,
    isLibraryOpen,
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
