// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Ref } from "react";

import { IslandCrossfade } from "../../../primitives/island/island-crossfade";

import { DevicePanel } from "./island-device-panel";
import { RecordPanel, ScreenshotPanel } from "./island-panels";
import { SourcePanel } from "./island-source-panel";
import { canRecord } from "./island-warning";

import type { IslandPanel, RecordingIslandProps } from "./island-types";

// Whichever part the island has grown into, faded across as it changes.

export function PanelBody({
  panel,
  props,
  ref,
}: {
  panel: IslandPanel;
  props: RecordingIslandProps;
  ref?: Ref<HTMLDivElement>;
}) {
  return (
    <IslandCrossfade ref={ref}>
      {panel === "source" ? (
        <SourcePanel {...props} />
      ) : panel === "screenshot" ? (
        <ScreenshotPanel
          canScroll={props.mode === "region"}
          onScreenshot={(action) => props.onScreenshot?.(action)}
        />
      ) : panel === "record" ? (
        <RecordPanel
          canStartReplay={canRecord(props)}
          onReplayChange={props.onReplayChange}
          replay={props.replay}
        />
      ) : (
        <DevicePanel
          input={props[panel]}
          isFlipped={Boolean(props.isCameraFlipped)}
          kind={panel}
          onDeviceChange={(ids) => {
            props.onDeviceChange(panel, ids);
          }}
          onFlipChange={props.onCameraFlipChange}
          onToggle={(isOn) => {
            props.onInputChange(panel, isOn);
          }}
          preview={panel === "camera" ? props.cameraPreview : undefined}
        />
      )}
    </IslandCrossfade>
  );
}
