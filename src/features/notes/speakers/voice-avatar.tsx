// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Volume2 } from "lucide-react";

import { t } from "../../../i18n/i18n";
import { IconToggleButton } from "../../../primitives/button/icon-button";
import { IconSwap } from "../../../primitives/icon-swap/icon-swap";

import { SpeakerAvatar } from "./speaker-avatar";

// A speaker's avatar in the speakers strip. While their voice can be heard,
// which lasts until they are named, the avatar is the button that plays it;
// named, the button gives way to their initials where it stood, so the pill
// keeps its size and nothing beside it moves.

export function VoiceAvatar({
  isPlaying,
  name,
  onHear,
}: {
  isPlaying: boolean;
  name: string;
  /** Plays or stops their voice; absent when it cannot be heard. */
  onHear?: () => void;
}) {
  return (
    <span className="relative size-7 shrink-0">
      <IconSwap swapKey={onHear ? "voice" : "avatar"}>
        {onHear ? (
          <IconToggleButton
            aria-label={t("notes-view-speaker-hear", { speaker: name })}
            className="size-7 rounded-full"
            isSelected={isPlaying}
            onChange={onHear}
          >
            <Volume2 aria-hidden className="size-icon-sm" />
          </IconToggleButton>
        ) : (
          <SpeakerAvatar name={name} />
        )}
      </IconSwap>
    </span>
  );
}
