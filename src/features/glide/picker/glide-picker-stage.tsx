// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { usePlayback } from "../glide-playback";
import { sampleAppIcon } from "../glide-sample-icon";

import { GlidePicker } from "./glide-picker";
import { type GlidePickerPick, HIDDEN, picks } from "./glide-picker-preview";

import type { GlidePickerProps, GlidePickerState } from "./glide-picker-types";

// The picker's stories: the picker centred on the canvas, where the gesture
// began, either holding one state or playing a pick through.

export type GlidePickerStageProps = Pick<
  GlidePickerProps,
  "kind" | "screens"
> & {
  /** Whether the glided app's icon resolves. */
  hasIcon: boolean;
  /** A pick to play through on a loop, in place of `state`. */
  pick?: GlidePickerPick;
  state?: GlidePickerState;
};

export function GlidePickerStage({
  hasIcon,
  pick,
  state = HIDDEN,
  ...props
}: GlidePickerStageProps) {
  const played = usePlayback<GlidePickerState>(
    pick ? picks[pick] : undefined,
    HIDDEN,
  );
  const showsIcon = hasIcon && (pick === undefined || played.icon);
  return (
    <div className="flex size-full items-center justify-center">
      <GlidePicker
        {...props}
        {...(pick ? played.state : state)}
        iconSrc={showsIcon ? sampleAppIcon : null}
        pulse={played.pulse}
      />
    </div>
  );
}
