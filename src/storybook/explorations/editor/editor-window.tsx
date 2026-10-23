// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pause, Play } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { IconButton } from "../../../primitives/button/icon-button";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { clock } from "../ideas/ideas-data";

import { Storyboard } from "./board/storyboard";
import {
  type Editor,
  type EditorKind,
  type EditorStart,
  useEditor,
  type View,
} from "./model/use-editor";
import { SoundView } from "./picture/sound-view";
import { WorkPicture } from "./picture/work-picture";
import { ActivityStrip } from "./strip/activity-strip";
import { TitleBar } from "./title-bar";
import { type Owner, ToolBar } from "./tools/tool-bar";
import { ToolPanel } from "./tools/tool-panel";

// The editor's content: the window draws its own material, so nothing here
// paints the window. The core tools sit in a slim column at the workspace's
// left edge, and what a tool needs opens beside its button. Below the
// picture is the recording as what was done, as one strip or as its steps.
//
// Keys: Space plays, Escape steps back (pen, More, panel, selection),
// Delete removes the selection, and for a beat or a step, its time.

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
      if (event.key === " ") {
        event.preventDefault();
        current.togglePlay();
      } else if (event.key === "Escape") current.back();
      else if (event.key === "Backspace" || event.key === "Delete")
        current.remove();
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, []);
}

function Transport({ editor }: { editor: Editor }) {
  return (
    <div className="flex w-32 shrink-0 flex-col justify-center gap-inline">
      <div className="flex items-center gap-inline">
        <IconButton
          aria-label={editor.isPlaying ? "Pause" : "Play"}
          color="primary"
          onPress={editor.togglePlay}
        >
          {editor.isPlaying ? <Pause /> : <Play />}
        </IconButton>
        <span className="flex flex-col">
          <span className="text-label tabular-nums">
            {clock(editor.map.toTime(editor.playhead))}
          </span>
          <span className="text-caption text-fg-tertiary tabular-nums">
            of {clock(editor.map.duration)}
          </span>
        </span>
      </div>
      {editor.kind === "recording" ? (
        <PillGroup
          aria-label="Timeline"
          items={[
            { id: "strip", label: "Strip" },
            { id: "board", label: "Steps" },
          ]}
          onSelectionChange={(view) => {
            editor.setView(view as View);
          }}
          selected={editor.view}
        />
      ) : null}
    </div>
  );
}

export function EditorWindow({
  isVoiced = true,
  kind = "recording",
  start,
}: {
  /** Whether the recording has a voice; a sound-only one always has. */
  isVoiced?: boolean;
  kind?: EditorKind;
  start?: EditorStart;
}) {
  const editor = useEditor({
    isVoiced: kind === "audio" || isVoiced,
    kind,
    start,
  });
  useKeys(editor);
  const anchorsRef = useRef(new Map<Owner, HTMLElement>());
  const [workspace, setWorkspace] = useState<HTMLElement | null>(null);
  return (
    <div className="flex size-full flex-col text-fg select-none">
      <TitleBar kind={kind} />
      <div className="flex min-h-0 grow flex-col gap-separate px-inset-sm pb-inset">
        <div className="flex min-h-0 grow gap-inline">
          <ToolBar anchors={anchorsRef} editor={editor} />
          <main
            className="relative flex min-h-0 min-w-0 grow items-center justify-center overflow-hidden rounded-panel bg-recess p-6"
            onPointerDown={(event) => {
              if (event.target === event.currentTarget) editor.select(null);
            }}
            ref={setWorkspace}
          >
            <ToolPanel
              anchors={anchorsRef}
              editor={editor}
              workspace={workspace}
            />
            {kind === "audio" ? (
              <SoundView playhead={editor.playhead} />
            ) : (
              <WorkPicture editor={editor} />
            )}
          </main>
        </div>
        {kind === "screenshot" ? null : (
          <section
            aria-label="Timeline"
            className="flex shrink-0 gap-separate px-inset-sm"
          >
            <Transport editor={editor} />
            {editor.view === "board" && kind === "recording" ? (
              <Storyboard editor={editor} />
            ) : (
              <ActivityStrip editor={editor} />
            )}
          </section>
        )}
      </div>
    </div>
  );
}
