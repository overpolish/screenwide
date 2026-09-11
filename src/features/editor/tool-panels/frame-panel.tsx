// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dimensions } from "../../../components/shared/dimensions/dimensions";
import { EditorKind } from "../types";

import { useToolPanelSnapshot } from "./use-tool-panel-snapshot";

/**
 * The Frame tool's own controls: how big the finished picture is.
 *
 * The numbers are the output canvas, the same ones the size readout under the
 * preview shows and the same ones a frame drag moves, so a value typed here
 * and a corner dragged out there mean the same thing. The reset puts the
 * canvas back to the size the capture arrived at.
 */
export function FramePanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { frame, isSaving } = snapshot;

  if (!frame) return null;

  const size = (next: { height?: number; width?: number }) => {
    change({ frameSize: next });
  };

  return (
    // `Dimensions` has no disabled state of its own: a save takes the whole
    // group out of reach the way the selection panel's does.
    <div className={isSaving ? "pointer-events-none opacity-50" : ""}>
      <Dimensions
        height={frame.height}
        initialLinked
        label="Size"
        layout="stacked"
        onReset={() => {
          change({ resetFrame: true });
        }}
        setDimensions={(width, height) => {
          size({ height, width });
        }}
        setHeight={(height) => {
          size({ height });
        }}
        setWidth={(width) => {
          size({ width });
        }}
        width={frame.width}
      />
    </div>
  );
}
