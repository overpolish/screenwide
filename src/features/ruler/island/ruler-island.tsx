// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence } from "motion/react";
import { useEffect, useState } from "react";

import { t } from "../../../i18n/i18n";
import { Island, type IslandProps } from "../../../primitives/island/island";
import { IslandRow } from "../../../primitives/island/island-row";

import {
  RulerActions,
  type RulerIslandParts,
  RulerToolButtons,
} from "./ruler-island-parts";
import { RulerSectionBody } from "./ruler-island-section";
import { toolSection } from "./ruler-island-tools";

// The Ruler's tools on the island: the tools to one side, and what can be
// done besides measuring to the other; on a Mac with a notch, the notch
// between them. Under the tool in hand hangs how it is used, and the edge
// tolerances hang under their control while they are open.

export type RulerIslandProps = RulerIslandParts &
  Pick<IslandProps, "isShown" | "notch" | "onHidden" | "onShapeChange"> & {
    /** Whether the edge tolerances hang open when the island first shows. */
    initialShowsEdges?: boolean;
  };

export function RulerIsland({
  initialShowsEdges = false,
  isShown = true,
  notch,
  onAction,
  onClose,
  onHidden,
  onShapeChange,
  onTolerance,
  onTool,
  stampKeys,
  state,
}: RulerIslandProps) {
  const [showsEdges, setShowsEdges] = useState(initialShowsEdges);
  const [edgesButton, setEdgesButton] = useState<HTMLButtonElement | null>(
    null,
  );
  const section = showsEdges ? "edges" : toolSection(state.tool, stampKeys);
  // The section folds away where it hung, so its place outlives it.
  const [placed, setPlaced] = useState(section);
  if (section && section !== placed) setPlaced(section);
  const isUnderEdges = (section ?? placed) === "edges";

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setShowsEdges(false);
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  return (
    <Island
      anchor="start"
      anchorControl={isUnderEdges ? edgesButton : null}
      aria-label={t("ruler-island-tools")}
      isShown={isShown}
      notch={notch}
      onHidden={onHidden}
      onShapeChange={onShapeChange}
      row={
        <IslandRow
          end={
            <RulerActions
              edgesRef={setEdgesButton}
              onAction={onAction}
              onClose={onClose}
              onToggleEdges={() => {
                setShowsEdges((current) => !current);
              }}
              showsEdges={showsEdges}
              state={state}
            />
          }
          start={
            <RulerToolButtons
              onPick={() => {
                setShowsEdges(false);
              }}
              onTool={onTool}
              tool={state.tool}
            />
          }
        />
      }
      section={
        section ? (
          <AnimatePresence initial={false} mode="popLayout">
            <RulerSectionBody
              key={section}
              onTolerance={onTolerance}
              section={section}
              stampKeys={stampKeys}
              tolerance={state.tolerance}
            />
          </AnimatePresence>
        ) : undefined
      }
    />
  );
}
