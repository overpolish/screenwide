// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AudioLines, Music } from "lucide-react";
import { useEffect, useRef } from "react";

import { t } from "../../../../i18n/i18n";
import { IslandFill } from "../../../../primitives/island/island-fill";
import { Text } from "../../../../primitives/text/text";
import { type ToolPanelAddingSounds } from "../../tools/tool-panel-strip";
import { EditorIsland } from "../editor-island";

import type { AudioClipKind } from "../../../../bindings/AudioClipKind";

// Songs or sound effects on their way to the timeline, asking which they
// are: the editor dims under two islands over the timeline, Sound effect
// over its left half and Song over its right, each saying where its kind
// lands. Dragged in, the editor follows the drag and says which half it is
// over (`use-sound-drag.ts`), so the window lets the drag through; chosen in
// the Sound island, the islands are pressed, and a press anywhere else, or
// Escape, adds nothing. With no room after the last song, Song says so and
// takes nothing.

const KINDS: readonly AudioClipKind[] = ["effect", "music"];

export function SoundKindChooser({
  adding,
  onChoose,
}: {
  adding: ToolPanelAddingSounds;
  onChoose: (kind: AudioClipKind | null) => void;
}) {
  const takes = (kind: AudioClipKind) =>
    kind === "effect" || adding.hasSongRoom;
  const choose = (kind: AudioClipKind | null) => {
    onChoose(kind !== null && takes(kind) ? kind : null);
  };
  const chooseRef = useRef(choose);
  chooseRef.current = choose;
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") chooseRef.current(null);
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  const island = (kind: AudioClipKind) => {
    const isSong = kind === "music";
    const Glyph = isSong ? Music : AudioLines;
    const isTaken = takes(kind);
    return (
      <div className="flex flex-1 items-center justify-center" key={kind}>
        <div className="pointer-events-auto">
          <EditorIsland
            content={{
              key: kind,
              section: (
                <Text
                  className="px-inset-sm text-fg-secondary"
                  variant="caption"
                >
                  {!isTaken
                    ? t("editor-island-audio-no-song-room")
                    : isSong
                      ? t("editor-island-add-song-where")
                      : t("editor-island-add-effect-where")}
                </Text>
              ),
              title: (
                <button
                  className="group relative isolate flex cursor-default items-center gap-inline rounded-full px-inset-sm py-1 text-body text-fg outline-none disabled:text-fg-disabled"
                  disabled={!isTaken}
                  onClick={() => {
                    choose(kind);
                  }}
                  type="button"
                >
                  <IslandFill
                    isChosen={false}
                    // A drag's island is the one under it, which the editor
                    // follows; a pointer's is the one it hovers.
                    isHovered={
                      adding.isCarried
                        ? adding.over === kind && isTaken
                        : undefined
                    }
                  />
                  <Glyph aria-hidden className="size-icon-lg" />
                  {isSong
                    ? t("editor-island-audio-song")
                    : t("editor-island-audio-effect")}
                </button>
              ),
            }}
            isShown
          />
        </div>
      </div>
    );
  };

  const { area } = adding;
  return (
    <div
      className="absolute inset-0 bg-island/50"
      onPointerDown={(event) => {
        if (event.target === event.currentTarget) choose(null);
      }}
    >
      {/* The timeline reads left to right in every language, and so do the
        halves the islands stand over. */}
      <div
        className="pointer-events-none absolute flex"
        dir="ltr"
        style={
          area
            ? {
                height: area.height,
                insetInlineStart: area.left,
                top: area.top,
                width: area.width,
              }
            : { inset: 0 }
        }
      >
        {KINDS.map(island)}
      </div>
    </div>
  );
}
