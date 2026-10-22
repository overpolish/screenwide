// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { IdeaFrame } from "./idea-frame";
import {
  AutoZoomCard,
  CountdownCard,
  FileSizeCard,
  FrameRateCard,
  WindowGapCard,
} from "./settings-cards";

// Settings that show what they do. A row of a label and a switch asks the
// reader to imagine the result; each card here plays it, and is changed by
// touching what it shows.

export function LivingSettings() {
  return (
    <IdeaFrame
      hint="Tap the countdown, pick the smoother ball, click the file size, click the zooming screen, drag the gap between the windows."
      pitch="Settings that show what they do. Instead of a label and a switch, each setting plays its effect, and you change it by touching the effect: the countdown, the motion of each frame rate, the size in the recording bar, the space between windows."
      title="Settings you can see"
    >
      <div className="grid w-full max-w-[920px] grid-cols-3 gap-separate">
        <CountdownCard />
        <FrameRateCard />
        <FileSizeCard />
        <AutoZoomCard />
        <WindowGapCard />
      </div>
    </IdeaFrame>
  );
}
