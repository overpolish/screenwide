// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pause, Play } from "lucide-react";

import { IconButton } from "../../../primitives/button/icon-button";
import { IconSwap } from "../../../primitives/icon-swap/icon-swap";

/** Plays a sound on the island, or stops it: the glyphs crossfade as it
 * turns over, filled, as the editor's transport draws play and pause. */
export function IslandPlayButton({
  isPlaying,
  label,
  onPress,
}: {
  isPlaying: boolean;
  /** What pressing it does now, for assistive technology. */
  label: string;
  onPress: () => void;
}) {
  return (
    <IconButton aria-label={label} className="shrink-0" onPress={onPress}>
      <IconSwap swapKey={isPlaying ? "pause" : "play"}>
        {isPlaying ? (
          <Pause aria-hidden className="size-icon fill-current" />
        ) : (
          <Play aria-hidden className="size-icon fill-current" />
        )}
      </IconSwap>
    </IconButton>
  );
}
