// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ZoomIn } from "lucide-react";
import { memo, ReactNode } from "react";

import { ButtonGroup } from "../../../components/base/button-group/button-group";
import { NumberField } from "../../../components/base/input-fields/number-field";

import { MINIMUM_ZOOM_CEILING } from "./preview-transform";

/**
 * Memoized: the zoom field and the tool buttons are react-aria trees that cost
 * more to re-render than the whole native preview pane, and none of their props
 * change while a canvas-resize gesture updates the output draft at pointer rate.
 */
export const PreviewToolbar = memo(function PreviewToolbar({
  maximumZoomPercent = MINIMUM_ZOOM_CEILING * 100,
  onZoomChange,
  tools,
  zoomPercent,
}: {
  onZoomChange: (zoomPercent: number) => void;
  zoomPercent: number;
  /**
   * Content-aware ceiling for the zoom field. A workspace that fits far below
   * actual pixels - a tall scrolling capture - raises it so the user can still
   * reach its own pixels.
   */
  maximumZoomPercent?: number;
  tools?: ReactNode;
}) {
  return (
    <div className="relative flex shrink-0 items-center justify-between gap-section bg-transparent px-section py-control-inset">
      <ButtonGroup
        aria-label="Editor tools"
        className="min-w-0 items-center gap-control"
      >
        {tools}
      </ButtonGroup>
      <NumberField
        aria-label="Preview zoom"
        className="w-28"
        leftSection={<ZoomIn className="size-icon-default" />}
        maxValue={maximumZoomPercent}
        minValue={10}
        onChange={(value) => {
          onZoomChange(Math.round(value));
        }}
        rightSection="%"
        showSteppers={false}
        step={1}
        value={zoomPercent}
      />
    </div>
  );
});
