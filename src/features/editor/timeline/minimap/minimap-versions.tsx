// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Monitor } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";
import { useListFormatter } from "react-aria";
import { Button } from "react-aria-components";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { useSize } from "../../../../primitives/island/island-size";
import { useFormatMarkMotion } from "../formats/format-mark-motion";
import { type TakenOut, takenTotal } from "../strip/taken-out";
import {
  recordedTime,
  type TimelineScale,
  type TimeMap,
  timelineScale,
} from "../timeline-scale";

import type { VersionMark } from "../../recording/formats/format-versions";

// Under the overview, a line for each version the recording comes out in
// (`format-versions.ts`), its name written on the line itself at its start,
// the line stopping short of the name, and marked where versions differ,
// so formats that do not match, and where, are seen without switching
// between them. The version in hand is drawn in the primary ink; pressing
// another puts its first format in hand. The lines span the overview's
// whole width and lay out what plays as its ticks do, so a mark sits under
// the stretch it marks, and run on dashed under what was squeezed or cut;
// under the name, a mark is cut away as the line is.

const share = (part: number) => `${String(part * 100)}%`;

/** Room, in px, the line leaves after the name. */
const NAME_AIR_PX = 4;

export type MinimapVersion = {
  /** The format a press puts in hand. */
  formatId: string;
  isInHand: boolean;
  /** Whether its screen is set its own way, said by its name. */
  isPlaced: boolean;
  /** Stays the same while the version does, so its line keeps its place. */
  key: string;
  marks: readonly VersionMark[];
  /** The formats it covers, named. */
  names: readonly string[];
};

export function MinimapVersions({
  map,
  onSwitch,
  taken,
  versions,
}: {
  map: TimeMap;
  onSwitch: (formatId: string) => void;
  /** What is taken out of the recording, which the overview draws past
   * what plays. */
  taken: TakenOut;
  versions: readonly MinimapVersion[];
}) {
  const { transition } = useFormatMarkMotion();
  const played =
    map.duration / Math.max(map.duration + takenTotal(taken), 1e-6);
  // Played seconds, laid along what plays as the strip's lanes are.
  const scale = timelineScale(recordedTime(map.duration));
  return (
    <AnimatePresence initial={false}>
      {versions.length > 0 ? (
        <motion.div
          animate={{ height: "auto", opacity: 1 }}
          aria-label={t("editor-timeline-versions")}
          className="overflow-hidden"
          dir="ltr"
          exit={{ height: 0, opacity: 0 }}
          initial={{ height: 0, opacity: 0 }}
          key="versions"
          role="group"
          transition={transition}
        >
          {/* Apart from the minimap by more than its view window reaches
              below it, so the window's frame never crosses the first line. */}
          <div className="flex flex-col pt-1.5">
            <AnimatePresence initial={false}>
              {versions.map((version) => (
                <motion.div
                  animate={{ height: "auto", opacity: 1 }}
                  className="overflow-hidden"
                  exit={{ height: 0, opacity: 0 }}
                  initial={{ height: 0, opacity: 0 }}
                  key={version.key}
                  transition={transition}
                >
                  <VersionLine
                    onSwitch={onSwitch}
                    played={played}
                    scale={scale}
                    version={version}
                  />
                </motion.div>
              ))}
            </AnimatePresence>
          </div>
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}

function VersionLine({
  onSwitch,
  played,
  scale,
  version,
}: {
  onSwitch: (formatId: string) => void;
  /** The share of the overview's width what plays takes. */
  played: number;
  scale: TimelineScale;
  version: MinimapVersion;
}) {
  const list = useListFormatter({ style: "short", type: "unit" });
  const { mark } = useFormatMarkMotion();
  const [name, setName] = useState<HTMLSpanElement | null>(null);
  const cut = (useSize(name)?.width ?? 0) + NAME_AIR_PX;
  const marks = (
    <AnimatePresence initial={false}>
      {version.marks.map((each) => (
        <motion.span
          aria-hidden
          className={cn(
            "absolute inset-y-0 min-w-1 rounded-full",
            version.isInHand
              ? "bg-primary"
              : "bg-fg-tertiary group-hover:bg-fg-secondary",
          )}
          key={each.id}
          // Placed at once, as the overview's ticks are: only a mark coming
          // or going is animated.
          style={{
            insetInlineStart: share(scale.toX(each.start)),
            width: share(
              (each.end === null ? 1 : scale.toX(each.end, true)) -
                scale.toX(each.start),
            ),
          }}
          {...mark()}
        />
      ))}
    </AnimatePresence>
  );
  /** The name's end, as a mask keeping a layer to one side of it: past it
   * (`after`), or under it. */
  const cutAt = (after: boolean) =>
    `linear-gradient(to right, ${after ? "transparent" : "#000"} ${String(cut)}px, ${after ? "#000" : "transparent"} ${String(cut)}px)`;
  return (
    <Button
      aria-current={version.isInHand || undefined}
      className="group relative flex h-4 w-full items-center text-caption leading-none outline-none"
      onPress={() => {
        onSwitch(version.formatId);
      }}
    >
      {/* The line and its marks stop short of the name, so nothing strikes
          through it; a mark that falls under the name underlines it. Past
          what plays, where the overview draws what was squeezed and cut,
          the line runs on dashed, as nothing there plays to differ. */}
      <span
        className="absolute inset-x-0 top-1/2 h-0.5 -translate-y-1/2"
        style={{ maskImage: cutAt(true) }}
      >
        <span
          className="absolute inset-y-0 start-0"
          style={{ width: share(played) }}
        >
          <span className="absolute inset-x-0 top-1/2 h-px -translate-y-1/2 bg-fill-strong" />
          {marks}
        </span>
        {played < 1 ? (
          <span
            className="absolute end-0 top-1/2 -translate-y-1/2 border-t border-dashed border-fill-strong"
            style={{ insetInlineStart: share(played) }}
          />
        ) : null}
      </span>
      <span
        className="absolute start-0 bottom-0 h-0.5"
        style={{ maskImage: cutAt(false), width: share(played) }}
      >
        {marks}
      </span>
      <span
        className={cn(
          "relative flex max-w-full items-center gap-inline transition-colors",
          version.isInHand
            ? "text-fg"
            : "text-fg-tertiary group-hover:text-fg-secondary",
        )}
        ref={setName}
      >
        <span className="truncate" dir="auto">
          {list.format(version.names)}
        </span>
        {/* Its own screen holds for the whole recording, so it is said by
            the name rather than marked along the line over everything. */}
        {version.isPlaced ? (
          <>
            <Monitor aria-hidden className="size-icon-xs shrink-0" />
            <span className="sr-only">
              {t("editor-timeline-version-own-screen")}
            </span>
          </>
        ) : null}
      </span>
    </Button>
  );
}
