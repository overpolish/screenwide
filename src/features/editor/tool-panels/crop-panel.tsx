// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dimensions } from "../../../components/shared/dimensions/dimensions";
import { EditorKind } from "../types";

import { useToolPanelSnapshot } from "./use-tool-panel-snapshot";

/**
 * The Crop tool's own controls: how much of the capture is kept.
 *
 * The numbers are source pixels - what the camera or the screen actually
 * recorded - rather than output pixels, because a crop chooses part of the
 * original and the finished picture's size is the Frame tool's business. A
 * size typed here moves the same rectangle the crop handles drag, about its
 * own centre, and the reset gives the whole capture back.
 */
export function CropPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { crop, isLocked } = snapshot;

  if (!crop) return null;

  const size = (next: { height?: number; width?: number }) => {
    change({ cropSize: next });
  };

  return (
    // `Dimensions` has no disabled state of its own: a save takes the whole
    // group out of reach the way the frame panel's does.
    <div
      className={`flex flex-col gap-section ${isLocked ? "pointer-events-none opacity-50" : ""}`}
    >
      <Dimensions
        height={crop.height}
        initialLinked
        label="Crop"
        layout="stacked"
        onReset={() => {
          change({ resetCrop: true });
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
        width={crop.width}
      />
    </div>
  );
}
