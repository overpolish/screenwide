// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { arrangeSlots } from "./glide-picker-arrange";
import { GlidePickerCard } from "./glide-picker-card";
import {
  CARD,
  CARD_GAP,
  describePick,
  type GlidePickerProps,
} from "./glide-picker-types";

// Every card of a pick laid out as the native windows place them
// (`picker_cards`), for the picker's stories: in the app each card is a
// window of its own.

export function GlidePicker(props: GlidePickerProps) {
  const { iconSrc, isShown, origin, pulse, screens, selected } = props;
  const { height, slots, width } = arrangeSlots(screens, CARD, CARD_GAP);
  // Each native card counts the rests it was chosen for, so only the chosen
  // card breathes; a rest here goes to the card chosen when it came.
  const [rests, setRests] = useState(() => ({
    counts: new Map<number, number>(),
    pulse,
  }));
  if (rests.pulse !== pulse) {
    const counts = new Map(rests.counts);
    counts.set(selected, (counts.get(selected) ?? 0) + 1);
    setRests({ counts, pulse });
  }
  return (
    <div
      aria-label={describePick(props.kind, selected, screens.length)}
      className="relative"
      dir="ltr"
      role="img"
      style={{ height, width }}
    >
      {slots.map((slot, index) => (
        <div
          className="absolute flex items-center justify-center"
          // A card's place is its screen's, so it is its key.
          // eslint-disable-next-line @eslint-react/no-array-index-key
          key={index}
          style={{
            height: CARD.height,
            insetInlineStart: slot.x,
            top: slot.y,
            width: CARD.width,
          }}
        >
          <GlidePickerCard
            iconSrc={iconSrc}
            isOrigin={index === origin}
            isSelected={index === selected}
            isShown={isShown}
            pulse={rests.counts.get(index) ?? 0}
          />
        </div>
      ))}
    </div>
  );
}
