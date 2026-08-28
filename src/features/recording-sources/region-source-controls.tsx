// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dimensions } from "../../components/shared/dimensions/dimensions";
import { wholePixelSize } from "../region-selector/region-geometry";

import { useRecordingSourceStore } from "./store";

export function RegionSourceControls() {
  const { region, regionAspectRatio, setRegion, setRegionAspectRatio } =
    useRecordingSourceStore((state) => state);

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
    <div className="gap-control flex h-full min-w-0 items-center overflow-visible">
      <Dimensions
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
