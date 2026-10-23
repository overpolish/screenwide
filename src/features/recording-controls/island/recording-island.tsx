// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { type ReactNode, useEffect, useState } from "react";

import { t } from "../../../i18n/i18n";
import { motionDurations, motionEasings } from "../../../lib/motion";
import { Island } from "../../../primitives/island/island";
import { IslandWarning } from "../../../primitives/island/island-controls";
import { IslandRow } from "../../../primitives/island/island-row";

import { Countdown, Finishing, Starting } from "./island-live";
import { PanelBody } from "./island-panel-body";
import { RecordingRow } from "./island-recording";
import { RestingRow } from "./island-resting";
import { islandWarning } from "./island-warning";

import type {
  IslandInputKind,
  IslandPanel,
  RecordingIslandProps,
} from "./island-types";
import type { IslandAnchor } from "../../../primitives/island/island-geometry";

/** Each part hangs under the controls that open it. */
const ANCHORS: Record<IslandPanel, IslandAnchor> = {
  camera: "start",
  microphone: "start",
  record: "end",
  screenshot: "end",
  source: "start",
  systemAudio: "start",
};

// The recording controls on the island: what is recorded, the inputs, a
// screenshot and Record, with what will be recorded shown beneath them. Any
// other choice opens in its place and every way of closing it steps back to
// what will be recorded; before recording the island is never just the row.
// Recording, it shows the recording in the same place, its controls showing
// while the pointer is on it, and while annotating the annotate tools hang
// under it as rows of their own.

export function RecordingIsland(props: RecordingIslandProps) {
  const { initialPanel = "source", isShown = true, notch, phase } = props;
  const [panel, setPanel] = useState<IslandPanel>(initialPanel);
  // Each time it comes out, it opens on what it will record again.
  const [wasShown, setWasShown] = useState(isShown);
  if (isShown !== wasShown) {
    setWasShown(isShown);
    if (isShown) setPanel(initialPanel);
  }
  const { onPanelChange } = props;
  useEffect(() => {
    onPanelChange?.(panel);
  }, [onPanelChange, panel]);
  const [isHovered, setIsHovered] = useState(false);
  // Pressing what is open steps back to the source.
  const toggle = (next: IslandPanel) => {
    setPanel((current) => (current === next ? "source" : next));
  };

  // Escape steps back to the source. Pressing elsewhere leaves the island as
  // it is, so a choice stays open while working beside it.
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setPanel("source");
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  // An input's glyph only switches it, so several can be flicked on and off
  // without the island changing shape; its chevron alone opens its devices.
  // Switching one off steps back to the source if its devices were showing.
  const toggleInput = (kind: IslandInputKind) => {
    const isOn = !props[kind].isOn;
    props.onInputChange(kind, isOn);
    if (!isOn) setPanel((current) => (current === kind ? "source" : current));
  };

  const warning = phase === "idle" ? islandWarning(props, setPanel) : undefined;
  const isLive = phase === "recording" || phase === "paused";
  const showsControls = phase === "paused" || isHovered;

  let row: ReactNode = null;
  let section: ReactNode = null;
  let below: ReactNode = null;
  let anchor: IslandAnchor = "center";
  if (phase === "idle") {
    row = (
      <RestingRow
        onToggleInput={toggleInput}
        panel={panel}
        props={props}
        toggle={toggle}
      />
    );
    anchor = ANCHORS[panel];
    section = (
      <>
        {/* What stops recording is said above the source. The parts below
            slide to make room for it, on the island's own curve, rather
            than jumping while the black around them grows. It takes the
            source's width rather than setting its own, so a long message
            wraps instead of widening the island. */}
        <AnimatePresence initial={false} mode="popLayout">
          {warning && panel === "source" ? (
            <IslandWarning
              className="w-0 min-w-full"
              key="warning"
              {...warning}
            />
          ) : null}
        </AnimatePresence>
        <motion.div
          className="relative"
          layout="position"
          transition={{
            layout: {
              duration: motionDurations.travel,
              ease: motionEasings.out,
            },
          }}
        >
          <AnimatePresence initial={false} mode="popLayout">
            <PanelBody key={panel} panel={panel} props={props} />
          </AnimatePresence>
        </motion.div>
      </>
    );
  } else if (phase === "countdown") {
    row = (
      <IslandRow
        end={
          <Countdown
            count={props.countdown ?? 3}
            onCancel={props.onCancelCountdown}
          />
        }
      />
    );
  } else if (phase === "starting") {
    row = <IslandRow end={<Starting onCancel={props.onCancelStarting} />} />;
  } else if (phase === "finishing") {
    row = <IslandRow end={<Finishing />} />;
  } else if (isLive) {
    row = <RecordingRow props={props} showsControls={showsControls} />;
    if (props.annotate) {
      // The annotate tools lie in the row's black under the recording's own
      // row, and their options hang from them as on the annotate island.
      below = props.annotate.tools;
      section = props.annotate.options;
      anchor = props.annotate.anchor;
    }
  }

  return (
    <Island
      anchor={anchor}
      aria-label={t("recording-controls-island")}
      below={below}
      isShown={isShown}
      notch={notch}
      onHidden={props.onHidden}
      onHoverChange={setIsHovered}
      onShapeChange={props.onShapeChange}
      row={row}
      section={section}
    />
  );
}
