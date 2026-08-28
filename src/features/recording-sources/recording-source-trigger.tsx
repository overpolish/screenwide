// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import {
  AppWindowMac,
  AudioLines,
  Camera,
  ChevronDown,
  Monitor,
} from "lucide-react";
import { PressEvent } from "react-aria";

import { Button } from "../../components/base/button/button";

import { MonitorDetails, RecordingMode, WindowDetails } from "./types";

type RecordingSourceTriggerProps = {
  isExpanded: boolean;
  mode: RecordingMode;
  onPress: (event: PressEvent) => void;
  selectedMonitor: MonitorDetails | null;
  selectedWindow: WindowDetails | null;
};

export function RecordingSourceTrigger({
  isExpanded,
  mode,
  onPress,
  selectedMonitor,
  selectedWindow,
}: RecordingSourceTriggerProps) {
  const sourceSelectionAvailable = ["region", "screen", "window"].includes(
    mode,
  );

  return (
    <div className="grid min-w-0 grow">
      <Button
        isDisabled={!sourceSelectionAvailable}
        onPress={onPress}
        size="compact"
      >
        {mode === "camera" ? (
          <Camera aria-hidden className="size-icon-compact shrink-0" />
        ) : mode === "audio" ? (
          <AudioLines aria-hidden className="size-icon-compact shrink-0" />
        ) : mode === "window" ? (
          selectedWindow?.appIconPath ? (
            <img
              alt=""
              className="size-icon-compact shrink-0 object-contain"
              src={convertFileSrc(selectedWindow.appIconPath)}
            />
          ) : (
            <AppWindowMac aria-hidden className="size-icon-compact shrink-0" />
          )
        ) : (
          <Monitor aria-hidden className="size-icon-compact shrink-0" />
        )}
        <span className="truncate">
          {!sourceSelectionAvailable
            ? "Source selection not required"
            : mode === "window"
              ? (selectedWindow?.title ?? "Choose a window")
              : (selectedMonitor?.name ?? "Choose a display")}
        </span>
        {sourceSelectionAvailable ? (
          <ChevronDown
            aria-hidden
            className={`size-icon-compact transform-gpu transition-transform duration-200 ${isExpanded ? "rotate-180" : "rotate-0"}`}
          />
        ) : null}
      </Button>
    </div>
  );
}
