// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ImageDown, SquareDot } from "lucide-react";

import { Button } from "../../components/base/button/button";
import { CheckOnClick } from "../../components/shared/check-on-click/check-on-click";
import { Dimensions } from "../../components/shared/dimensions/dimensions";
import { cn } from "../../lib/styling";
import { Region } from "../recording-sources/types";

import { wholePixelSize } from "./region-geometry";

export function ScreenshotRegionControls({
  onAspectChange,
  onCenter,
  onFinish,
  onSizeChange,
  region,
  regionPlaced,
  visible,
}: {
  onAspectChange: (aspect: number | undefined) => void;
  onCenter: () => void;
  onFinish: () => void;
  onSizeChange: (size: Region["size"]) => void;
  region: Region;
  regionPlaced: boolean;
  visible: boolean;
}) {
  const isMac = navigator.userAgent.includes("Mac");

  return (
    <div
      className={cn(
        "absolute left-1/2 flex -translate-x-1/2 items-center justify-center opacity-0 transition-opacity",
        isMac ? "top-12" : "top-2",
        visible && "opacity-100",
      )}
    >
      <div
        className={cn(
          "pointer-events-none flex items-center gap-2 rounded-md border border-muted/25 bg-content p-2 shadow-md",
          visible && "pointer-events-auto",
        )}
      >
        <CheckOnClick onPress={onCenter}>
          <Button isDisabled={!regionPlaced} size="compact" variant="ghost">
            <SquareDot aria-hidden size={14} />
            Center
          </Button>
        </CheckOnClick>
        <Dimensions
          height={region.size.height}
          onRatioChange={onAspectChange}
          setHeight={(height) => {
            // A ratio picked before a region exists is only the shape to draw
            // at; there is nothing to resize yet.
            if (!regionPlaced) return;
            onSizeChange({
              ...region.size,
              height: wholePixelSize(height),
            });
          }}
          setWidth={(width) => {
            // As above: no region, nothing to resize.
            if (!regionPlaced) return;
            onSizeChange({
              ...region.size,
              width: wholePixelSize(width),
            });
          }}
          width={region.size.width}
        />
        <Button
          color="primary"
          isDisabled={!regionPlaced}
          onPress={onFinish}
          size="compact"
        >
          <ImageDown aria-hidden size={18} />
          Capture
        </Button>
      </div>
    </div>
  );
}
