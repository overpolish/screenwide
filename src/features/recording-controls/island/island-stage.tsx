// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLocale } from "react-aria";

import { ISLAND } from "../../../primitives/island/island-geometry";

import { RecordingIsland } from "./recording-island";
import { useIslandStage } from "./use-island-stage";

import type { IslandStageProps } from "./island-stage-props";

export function IslandStage(props: IslandStageProps) {
  const island = useIslandStage(props);
  const { direction } = useLocale();
  // The stage is the screen, which stays left to right; the island on it
  // follows the language, as it does in its own window.
  return (
    <div className="relative size-full" dir="ltr">
      <div
        className="absolute inset-x-0"
        dir={direction}
        style={{ top: props.notch ? 0 : ISLAND.screenGap }}
      >
        <RecordingIsland {...island} />
      </div>
    </div>
  );
}
