// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pause, Play } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { IconButton } from "../../../../primitives/button/icon-button";
import { IconSwap } from "../../../../primitives/icon-swap/icon-swap";
import { Waveform } from "../../waveform/waveform";

import type { MomentIslandProps, MomentNote } from "./moment-island";

// A moment's voice note, on the line under its row: play and pause, the
// glyphs crossfading as it turns over, beside its waveform, which fills
// with the kind's colour as it plays. It sets the island's width, which
// what was said grows to fill once it is known.

export function MomentPlayer({
  color,
  name,
  note,
  onTogglePlay,
}: MomentIslandProps & { note: MomentNote }) {
  return (
    // Inset as the row's content is, so play sits under the flag, with room
    // beneath before the black ends or what was said hangs on.
    <div className="flex w-64 items-center gap-inline px-1 pb-inset-sm">
      <IconButton
        aria-label={
          note.isPlaying
            ? t("editor-island-moment-pause")
            : t("editor-island-moment-play")
        }
        className="shrink-0"
        onPress={onTogglePlay}
      >
        <IconSwap swapKey={note.isPlaying ? "pause" : "play"}>
          {/* Filled, as the editor's transport draws play and pause. */}
          {note.isPlaying ? (
            <Pause aria-hidden className="size-icon fill-current" />
          ) : (
            <Play aria-hidden className="size-icon fill-current" />
          )}
        </IconSwap>
      </IconButton>
      <Waveform
        aria-label={t("editor-island-moment-note", { moment: name })}
        className="h-island-control min-w-0 grow"
        color={color}
        peaks={note.peaks}
        progress={note.progress}
      />
    </div>
  );
}
