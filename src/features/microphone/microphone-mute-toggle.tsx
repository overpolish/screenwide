// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MicOff, Mic } from "lucide-react";

import { t } from "../../i18n/i18n";
import { IconToggleButton } from "../../primitives/button/icon-button";
import { IconSwap } from "../../primitives/icon-swap/icon-swap";
import { LevelFill } from "../recording-controls/island/island-glyphs";

/** The microphone of a recording or of notes, on the island: its glyph
 * fills with its level while it is heard, so it can be seen to work, and is
 * crossed out while muted. Pressed, it mutes or unmutes it; the toggle is
 * on while muted, as Mute is what it names. */
export function MicrophoneMuteToggle({
  isMuted,
  level,
  onMutedChange,
}: {
  isMuted: boolean;
  /** The microphone's level, 0 to 1. */
  level: number;
  onMutedChange: (muted: boolean) => void;
}) {
  return (
    <IconToggleButton
      aria-label={t("microphone-mute")}
      isSelected={isMuted}
      onChange={onMutedChange}
    >
      {({ isSelected }) => (
        <IconSwap swapKey={isSelected ? "muted" : "live"}>
          {isSelected ? (
            <MicOff aria-hidden className="size-icon text-fg-secondary" />
          ) : (
            <LevelFill icon={Mic} level={level} />
          )}
        </IconSwap>
      )}
    </IconToggleButton>
  );
}
