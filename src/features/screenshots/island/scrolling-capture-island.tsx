// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScrollText } from "lucide-react";

import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { Island, type IslandProps } from "../../../primitives/island/island";
import { IslandFlow } from "../../../primitives/island/island-flow";
import { IslandKeyHint } from "../../../primitives/island/island-key-hint";
import { IslandRow } from "../../../primitives/island/island-row";
import { IslandSwap } from "../../../primitives/island/island-swap";
import { IslandRowTitle } from "../../../primitives/island/row-title/island-row-title";
import { IslandRowPresence } from "../../editor/island/island-row-presence";
import { IslandWorkLine } from "../../editor/island/island-work-line";

import type { ScrollingCaptureIslandState } from "../../../bindings/ScrollingCaptureIslandState";
import type { ScrollingCapturePhase } from "../../../bindings/ScrollingCapturePhase";

// A scrolling capture on the island, which comes out while the capture drives
// the user's window for several seconds: what it is doing at the row's start,
// each phase clearing into the next, with Cancel at its end while it can
// still stop; under it how far it has got, as an export's work is shown, and
// the key that stops it where Escape was claimed. The capture cannot tell how
// far a page scrolls before it ends, so the bar sweeps.

export type ScrollingCaptureIslandProps = Pick<
  IslandProps,
  "isShown" | "notch" | "onHidden" | "onShapeChange"
> & {
  onCancel: () => void;
  state: ScrollingCaptureIslandState;
};

const TITLES: Record<ScrollingCapturePhase, () => string> = {
  capturing: () => t("screenshots-scrolling-capturing"),
  finishing: () => t("screenshots-scrolling-finishing"),
  stitching: () => t("screenshots-scrolling-stitching"),
  working: () => t("screenshots-scrolling-working"),
};

const NOTES: Record<ScrollingCapturePhase, () => string> = {
  capturing: () => t("screenshots-scrolling-capturing-note"),
  finishing: () => t("screenshots-scrolling-finishing-note"),
  stitching: () => t("screenshots-scrolling-stitching-note"),
  working: () => t("screenshots-scrolling-working-note"),
};

export function ScrollingCaptureIsland({
  isShown = true,
  notch,
  onCancel,
  onHidden,
  onShapeChange,
  state,
}: ScrollingCaptureIslandProps) {
  const { escapeClaimed, phase } = state;
  // Only the scrolling can be stopped: once the frames are being joined the
  // capture runs to its end.
  const canCancel = phase === "working" || phase === "capturing";
  return (
    <Island
      aria-label={t("screenshots-scrolling-island")}
      isShown={isShown}
      notch={notch}
      onHidden={onHidden}
      onShapeChange={onShapeChange}
      row={
        <IslandRow
          end={
            <IslandRowPresence isShown={canCancel}>
              <Button onPress={onCancel} variant="ghost">
                {t("screenshots-scrolling-cancel")}
              </Button>
            </IslandRowPresence>
          }
          start={
            <IslandSwap swapKey={phase}>
              <IslandRowTitle
                glyph={<ScrollText aria-hidden />}
                title={<span role="status">{TITLES[phase]()}</span>}
              />
            </IslandSwap>
          }
        />
      }
      section={
        <IslandFlow className="flex flex-col gap-inline">
          <IslandWorkLine key="work" label={NOTES[phase]()} value={null} />
          {canCancel && escapeClaimed ? (
            <IslandKeyHint
              binding="Escape"
              key="hint"
              message={(key) => t("screenshots-scrolling-cancel-hint", { key })}
            />
          ) : null}
        </IslandFlow>
      }
    />
  );
}
