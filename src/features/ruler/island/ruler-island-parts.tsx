// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Copy,
  Crosshair,
  ScanSearch,
  SquareCenterlineDashedVertical,
  Undo2,
  X,
} from "lucide-react";
import { type Ref } from "react";

import { t } from "../../../i18n/i18n";
import {
  IconButton,
  IconToggleButton,
} from "../../../primitives/button/icon-button";

import { RULER_TOOLS, type RulerStampKeys } from "./ruler-island-tools";

import type { RulerIslandAction } from "../../../bindings/RulerIslandAction";
import type { RulerIslandState } from "../../../bindings/RulerIslandState";
import type { RulerTolerance } from "../../../bindings/RulerTolerance";
import type { RulerTool } from "../../../bindings/RulerTool";

// The parts the Ruler's island is made of: the tools to one side, what can
// be done besides measuring to the other, and what hangs under them.

/** What the Ruler's island takes: the session as it is, and what each part
 * asks of it. */
export type RulerIslandParts = {
  onAction: (action: RulerIslandAction) => void;
  /** Leaves the Ruler, the way Escape does. */
  onClose: () => void;
  onTolerance: (tolerance: RulerTolerance) => void;
  onTool: (tool: RulerTool) => void;
  /** The keys that stamp a measurement, once the settings are read. */
  stampKeys: RulerStampKeys | null;
  state: RulerIslandState;
};

/** The tools, the one in hand chosen. */
export function RulerToolButtons({
  onPick,
  onTool,
  tool,
}: {
  onPick: () => void;
  onTool: (tool: RulerTool) => void;
  tool: RulerTool;
}) {
  return RULER_TOOLS.map(({ icon: Icon, id, name }) => (
    <IconToggleButton
      aria-label={name()}
      isSelected={id === tool}
      key={id}
      onChange={() => {
        if (id !== tool) onTool(id);
        onPick();
      }}
    >
      <Icon aria-hidden className="size-icon" />
    </IconToggleButton>
  ));
}

/** The edge tolerance, which opens its choices, the two aids that switch on
 * and off, then undo, copy and close. */
export function RulerActions({
  edgesRef,
  onAction,
  onClose,
  onToggleEdges,
  showsEdges,
  state,
}: Pick<RulerIslandParts, "onAction" | "onClose" | "state"> & {
  /** The edge tolerance's button, which its choices hang under. */
  edgesRef: Ref<HTMLButtonElement>;
  onToggleEdges: () => void;
  showsEdges: boolean;
}) {
  return (
    <>
      <IconButton
        aria-expanded={showsEdges}
        aria-label={t("ruler-island-edges")}
        onPress={onToggleEdges}
        ref={edgesRef}
        variant="ghost"
      >
        <ScanSearch aria-hidden className="size-icon" />
      </IconButton>
      <IconToggleButton
        aria-label={t("ruler-island-centerlines")}
        isSelected={state.centerlines}
        onChange={() => {
          onAction("toggleCenterlines");
        }}
      >
        <SquareCenterlineDashedVertical aria-hidden className="size-icon" />
      </IconToggleButton>
      <IconToggleButton
        aria-label={t("ruler-island-crosshair")}
        isSelected={state.crosshair}
        onChange={() => {
          onAction("toggleCrosshair");
        }}
      >
        <Crosshair aria-hidden className="size-icon" />
      </IconToggleButton>
      <IconButton
        aria-label={t("ruler-island-undo")}
        isDisabled={!state.canUndo}
        onPress={() => {
          onAction("undo");
        }}
        variant="ghost"
      >
        <Undo2 aria-hidden className="size-icon rtl:-scale-x-100" />
      </IconButton>
      <IconButton
        aria-label={t("ruler-island-copy")}
        isDisabled={!state.canCopy}
        onPress={() => {
          onAction("copyMeasurement");
        }}
        variant="ghost"
      >
        <Copy aria-hidden className="size-icon" />
      </IconButton>
      <IconButton
        aria-label={t("ruler-island-close")}
        onPress={onClose}
        variant="ghost"
      >
        <X aria-hidden className="size-icon" />
      </IconButton>
    </>
  );
}
