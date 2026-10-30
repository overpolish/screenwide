// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FastForward, Hourglass, Pin, Snail } from "lucide-react";
import { useNumberFormatter } from "react-aria";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Badge } from "../../../../primitives/badge/badge";

import type { LullPlay } from "../../island/lull/lull-island";

// How a lull plays, on the top edge of its spring's box, so it takes no room
// above the beats; the box is cut away under it (`spring-badge-notch.ts`).

export function SpringBadge({
  badgeRef,
  isShown,
  play,
  speed,
}: {
  badgeRef: (element: HTMLSpanElement | null) => void;
  /** Whether it shows at rest; otherwise it waits for the pointer. */
  isShown: boolean;
  play: LullPlay;
  /** How fast it plays against real time, or null as recorded. */
  speed: number | null;
}) {
  return (
    <span
      aria-hidden
      className={cn(
        "pointer-events-none absolute inset-x-0 top-0 z-10 flex -translate-y-1/2 justify-center transition-opacity",
        !isShown && "opacity-0 group-hover:opacity-100",
      )}
    >
      <span className="flex" ref={badgeRef}>
        <Badge>
          <PlayGlyph play={play} speed={speed} />
          {speed === null ? null : <SpeedLabel speed={speed} />}
        </Badge>
      </span>
    </span>
  );
}

function PlayGlyph({ play, speed }: { play: LullPlay; speed: number | null }) {
  if (play.kind === "keep") return <Pin aria-hidden />;
  if (speed === null) return <Hourglass aria-hidden />;
  return speed > 1 ? <FastForward aria-hidden /> : <Snail aria-hidden />;
}

function SpeedLabel({ speed }: { speed: number }) {
  const digits = speed < 10 ? 1 : 0;
  const rate = useNumberFormatter({
    maximumFractionDigits: digits,
    minimumFractionDigits: digits,
  });
  return t("editor-timeline-rate", { rate: rate.format(speed) });
}
