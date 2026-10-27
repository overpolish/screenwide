// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { EditorIsland } from "./island/editor-island";
import { chooseNeighbour } from "./model/strip-items";
import {
  type Editor,
  type EditorKind,
  type EditorStart,
  useEditor,
} from "./model/use-editor";
import { SoundView } from "./picture/sound-view";
import { WorkPicture } from "./picture/work-picture";
import { type Recording } from "./recording";
import { WALKTHROUGH } from "./recordings/walkthrough";
import { ActivityStrip } from "./strip/activity-strip";
import { TitleBar } from "./title-bar";
import { ToolBar } from "./tools/tool-bar";
import { Transport } from "./transport";

// The editor's content: the window draws its own material, so nothing here
// paints the window. The core tools sit in a slim column at the workspace's
// left edge; what is in hand opens on an island beside the tool or over the
// part of the timeline it belongs to. Below the picture is the recording as
// one strip of what was done, with its scenes along the top.
//
// Keys: Space plays, Escape steps back (pen, mode, selection), Delete
// removes the selection, and for a beat, a pause or a stretch drawn with ⌥,
// its time. ⌘Z undoes and ⇧⌘Z redoes (Ctrl on Windows). The left and right
// arrows choose the beat or pause beside the chosen one, and the lens moves
// with them.

function useKeys(editor: Editor) {
  const ref = useRef(editor);
  useEffect(() => {
    ref.current = editor;
  });
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement;
      if (target.closest("input, textarea, [contenteditable]")) return;
      const current = ref.current;
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "z") {
        event.preventDefault();
        if (event.shiftKey) current.redo();
        else current.undo();
      } else if (event.ctrlKey && event.key.toLowerCase() === "y") {
        event.preventDefault();
        current.redo();
      } else if (event.key === " ") {
        event.preventDefault();
        current.togglePlay();
      } else if (event.key === "Escape") current.back();
      else if (event.key === "Backspace" || event.key === "Delete")
        current.remove();
      else if (
        (event.key === "ArrowLeft" || event.key === "ArrowRight") &&
        // Controls that take the arrows themselves keep them.
        !target.closest("[role='radiogroup'], [role='slider'], [role='menu']")
      ) {
        event.preventDefault();
        chooseNeighbour(current, event.key === "ArrowLeft" ? -1 : 1);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, []);
}

export function EditorWindow({
  isVoiced = true,
  kind = "recording",
  recording = WALKTHROUGH,
  start,
}: {
  /** Whether the recording has a voice; a sound-only one always has. */
  isVoiced?: boolean;
  kind?: EditorKind;
  recording?: Recording;
  start?: EditorStart;
}) {
  const editor = useEditor({
    isVoiced: kind === "audio" || isVoiced,
    kind,
    recording,
    start,
  });
  useKeys(editor);
  const [workspace, setWorkspace] = useState<HTMLElement | null>(null);
  return (
    <div
      className="flex size-full flex-col text-fg select-none"
      data-editor-window
    >
      <TitleBar editor={editor} />
      <div className="flex min-h-0 grow flex-col gap-separate px-inset-sm pb-inset">
        <div className="flex min-h-0 grow gap-inline">
          <ToolBar editor={editor} />
          <main
            className="relative flex min-h-0 min-w-0 grow items-center justify-center overflow-hidden rounded-panel bg-recess p-6"
            onPointerDown={(event) => {
              if (event.target === event.currentTarget) editor.select(null);
            }}
            ref={setWorkspace}
          >
            <EditorIsland editor={editor} workspace={workspace} />
            {kind === "audio" ? (
              <SoundView
                playhead={editor.playhead}
                speech={editor.recording.speech}
              />
            ) : (
              <WorkPicture editor={editor} />
            )}
          </main>
        </div>
        {kind === "screenshot" ? null : (
          // Laid out as the row above: the transport in the tool column's
          // width, the strip under the workspace.
          <section aria-label="Timeline" className="flex shrink-0 gap-inline">
            <Transport editor={editor} />
            <ActivityStrip editor={editor} />
          </section>
        )}
      </div>
    </div>
  );
}
