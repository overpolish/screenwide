// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Region } from "../recording-sources/types";

export function RegionShade({ region }: { region: Region }) {
  return (
    <svg aria-hidden className="pointer-events-none absolute size-full">
      <defs>
        <mask id="region-cutout">
          <rect className="fill-white" height="100%" width="100%" />
          <rect
            className="fill-black"
            height={region.size.height}
            width={region.size.width}
            x={region.position.x}
            y={region.position.y}
          />
        </mask>
      </defs>
      <rect
        className="fill-black/50"
        height="100%"
        mask="url(#region-cutout)"
        width="100%"
      />
    </svg>
  );
}
