// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { sampleAppIcon } from "../../glide/glide-sample-icon";
import { GlideIsland } from "../../glide/island/glide-island";
import { MAP_SHAPE } from "../../glide/island/glide-island-shapes";
import { menuBarHeight } from "../preview/scenes/sample-system-bars";
import { paintAppWindow } from "../preview/scenes/sample-window";
import { type Box } from "../preview/scenes/scene-kit";
import { useDisplayNotches } from "../preview/use-display-notches";
import { useSystemCursor } from "../preview/use-system-cursor";

import {
  GESTURE_LOOP,
  type GlideGesture,
  glideFrame,
  type WorkArea,
} from "./glide-script";
import { StageDesktop } from "./stage-desktop";
import { StageMenuBar } from "./stage-menu-bar";
import { StageScreen } from "./stage-screen";
import { useClock } from "./use-clock";

import type { PictureSize } from "../preview/settings-preview-api";

// Glide at work on the sample desktop (`glide-script.ts`): the real island
// over two windows, one of them moved to the side or centred. The space
// around windows is the one set, the pointer comes with the window if it
// follows, and with the gesture off nothing comes and the window stays
// where it is. The screen is scaled down to leave the windows room to read
// as windows.

/** The screen's room: tall enough for a window's content. */
const ROOM = { height: 220, width: 440 };
/** Half the island's open size, which its centre is kept inside the
 * screen by. */
const REACH = {
  height: MAP_SHAPE.height / 2 + 6,
  width: MAP_SHAPE.width / 2 + 6,
};

const clamp = (value: number, low: number, high: number) =>
  Math.min(Math.max(value, low), high);

/** The two windows, painted whenever either moves. */
function Windows({
  moving,
  screen,
  still,
}: {
  moving: Box;
  screen: PictureSize;
  still: Box;
}) {
  const ref = useRef<HTMLCanvasElement>(null);
  const frame = JSON.stringify([moving, still]);
  useEffect(() => {
    const canvas = ref.current;
    const context = canvas?.getContext("2d");
    if (!canvas || !context) return;
    const scale = window.devicePixelRatio;
    canvas.width = Math.round(screen.width * scale);
    canvas.height = Math.round(screen.height * scale);
    context.scale(scale, scale);
    const [moved, other] = JSON.parse(frame) as [Box, Box];
    paintAppWindow(context, {
      box: other,
      button: t("settings-preview-window-button"),
      chart: true,
      heading: t("settings-preview-dashboard-title"),
    });
    paintAppWindow(context, {
      box: moved,
      heading: t("settings-preview-note-title"),
      lines: [0.9, 0.7, 0.8, 0.55],
      sidebar: true,
    });
  }, [frame, screen.height, screen.width]);
  return <canvas className="absolute inset-0 size-full" ref={ref} />;
}

function GlideScreen({
  cursorFollows,
  gap,
  gesture,
  isOn,
  screen,
}: {
  cursorFollows: boolean;
  gap: number;
  gesture: GlideGesture;
  isOn: boolean;
  screen: PictureSize;
}) {
  const cursor = useSystemCursor();
  const { island: notch } = useDisplayNotches();
  const time = useClock(`${gesture} ${String(isOn)}`);
  const top = menuBarHeight(notch);
  const work: WorkArea = { ...screen, height: screen.height - top, top };
  const frame = glideFrame({
    at: time % GESTURE_LOOP[gesture],
    cursorFollows,
    gap,
    gesture,
    isOn,
    work,
  });
  const { island, pointer } = frame;

  return (
    <StageDesktop isOn={isOn}>
      <StageMenuBar notch={notch} width={screen.width} />
      <Windows moving={frame.moving} screen={screen} still={frame.still} />
      {island ? (
        // Centred where the gesture began, and kept inside the screen as
        // the app keeps it inside the display.
        <div
          className="absolute"
          style={{
            insetInlineStart: clamp(
              frame.grip.x,
              REACH.width,
              screen.width - REACH.width,
            ),
            top: clamp(
              frame.grip.y,
              REACH.height,
              screen.height - REACH.height,
            ),
            transform: "translate(-50%, -50%)",
          }}
        >
          <GlideIsland
            fit={null}
            iconSrc={sampleAppIcon}
            pending={null}
            phase={island.phase}
            pulse={island.pulse}
            region={island.region}
          />
        </div>
      ) : null}
      {/* The system draws the pointer over every window, the island too. */}
      {pointer.hidden ? null : (
        <img
          alt=""
          className={cn(
            "absolute max-w-none origin-top-left",
            pointer.pressed && "scale-90",
          )}
          src={cursor.src}
          style={{
            height: cursor.height,
            insetInlineStart: pointer.x - cursor.hotspotX,
            top: pointer.y - cursor.hotspotY,
            width: cursor.width,
          }}
        />
      )}
    </StageDesktop>
  );
}

export function GlidePicture(props: {
  cursorFollows: boolean;
  /** The space around windows, in points. */
  gap: number;
  /** Which gesture the picture plays. */
  gesture: GlideGesture;
  isOn: boolean;
}) {
  return (
    <StageScreen fits={ROOM}>
      {(screen) => <GlideScreen {...props} screen={screen} />}
    </StageScreen>
  );
}
