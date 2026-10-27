// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pause, Play } from "lucide-react";
import { useEffect, useRef } from "react";

import { cn } from "../../../../lib/styling";
import { IslandButton } from "../../../../primitives/island/island-controls";
import { type Editor } from "../model/use-editor";
import { type Beat } from "../recording";

// What the island holds for a flagged moment: a play button that plays the
// moment, from where it starts to where it settles, and stops there; and
// what was said in it, each word lit as it is heard. A moment is flagged
// for what happened in it, so it is heard and read before it is kept or cut.

export function MomentPlay({ beat, editor }: { beat: Beat; editor: Editor }) {
  // Set while this button's play is running, so it stops at the moment's
  // end and leaves a play from the transport alone.
  const isOwnRef = useRef(false);
  const isPlayingMoment = editor.isPlaying && isOwnRef.current;
  useEffect(() => {
    if (!editor.isPlaying) isOwnRef.current = false;
    else if (isOwnRef.current && editor.playhead >= beat.end) {
      isOwnRef.current = false;
      editor.togglePlay();
    }
  }, [beat.end, editor]);
  const Icon = isPlayingMoment ? Pause : Play;
  return (
    <IslandButton
      aria-label={isPlayingMoment ? "Pause" : "Play the moment"}
      onPress={() => {
        if (editor.isPlaying) {
          editor.togglePlay();
          return;
        }
        isOwnRef.current = true;
        editor.seek(beat.start);
        editor.togglePlay();
      }}
    >
      <Icon aria-hidden className="size-icon" />
    </IslandButton>
  );
}

export function MomentSection({
  beat,
  editor,
}: {
  beat: Beat;
  editor: Editor;
}) {
  const words = editor.recording.words.filter(
    ({ at }) => at >= beat.start && at < beat.end,
  );
  // The row's title sits further in than a section's edge, past the row's
  // inset and the space a glyph would take, so the words start under it.
  if (words.length === 0)
    return (
      <p className="w-64 ps-2.5 text-caption text-fg-tertiary">
        Nothing was said.
      </p>
    );
  const isHeard = (at: number) =>
    editor.playhead >= beat.start &&
    editor.playhead <= beat.end &&
    at <= editor.playhead;
  return (
    <p className="w-64 ps-2.5 text-body">
      {words.map(({ at, word }) => (
        <span
          className={cn(
            "transition-colors",
            isHeard(at) ? "text-fg" : "text-fg-tertiary",
          )}
          key={at}
        >
          {`${word} `}
        </span>
      ))}
    </p>
  );
}
