// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SquareDot } from "lucide-react";

import { Button } from "../../components/base/button/button";
import { AspectRatio } from "../../components/shared/aspect-ratio/aspect-ratio";
import { wholePixel, wholePixelSize } from "../region-selector/region-geometry";

import { useRecordingSourceStore } from "./store";

export function RegionSourceControls() {
  const {
    region,
    regionAspectRatio,
    selectedMonitor,
    setRegion,
    setRegionAspectRatio,
  } = useRecordingSourceStore((state) => state);

  const centerRegion = () => {
    if (!selectedMonitor) return;
    setRegion({
      ...region,
      position: {
        x: wholePixel((selectedMonitor.size.width - region.size.width) / 2),
        y: wholePixel((selectedMonitor.size.height - region.size.height) / 2),
      },
    });
  };

  const setRegionSize = (width: number, height: number) => {
    setRegion({
      ...region,
      size: {
        height: wholePixelSize(height),
        width: wholePixelSize(width),
      },
    });
  };

  return (
    <div className="flex h-full min-w-0 items-center gap-1 overflow-hidden">
      <Button
        aria-label="Center region"
        className="h-full shrink-0"
        icon
        isDisabled={!selectedMonitor}
        onPress={centerRegion}
        showFocus={false}
        size="sm"
        variant="ghost"
      >
        <SquareDot aria-hidden size={14} />
      </Button>
      <AspectRatio
        className="min-w-0"
        height={region.size.height}
        initialLinked={regionAspectRatio !== undefined}
        onRatioChange={setRegionAspectRatio}
        setDimensions={setRegionSize}
        setHeight={(height) => {
          setRegionSize(region.size.width, height);
        }}
        setWidth={(width) => {
          setRegionSize(width, region.size.height);
        }}
        width={region.size.width}
      />
    </div>
  );
}
