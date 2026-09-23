// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import {
  AppWindowMac,
  AudioLines,
  Monitor,
  SquareDashed,
  Video,
} from "lucide-react";
import { useState } from "react";

import {
  PillGroup,
  type PillGroupItem,
} from "../../../components/base/pill-group/pill-group";
import { cn } from "../../../lib/styling";
import {
  MonitorDetails,
  RecordingMode,
  WindowDetails,
} from "../../recording-sources/types";

export function RecordingTypePicker({
  isDisabled,
  mode,
  monitorThumbnails,
  onChooseMonitor,
  onChooseWindow,
  onModeChange,
  selectedMonitor,
  selectedWindow,
}: {
  mode: RecordingMode;
  /** A still of each attached display by its id. */
  monitorThumbnails: Record<number, string>;
  onChooseMonitor: (anchor: DOMRect, fromKeyboard: boolean) => void;
  onChooseWindow: (anchor: DOMRect, fromKeyboard: boolean) => void;
  onModeChange: (mode: RecordingMode) => void;
  selectedMonitor: MonitorDetails | null;
  selectedWindow: WindowDetails | null;
  isDisabled?: boolean;
}) {
  const monitorThumbnail = selectedMonitor
    ? monitorThumbnails[selectedMonitor.id]
    : undefined;
  const isPortraitMonitor = selectedMonitor
    ? selectedMonitor.size.height > selectedMonitor.size.width
    : false;
  const items: PillGroupItem[] = [
    {
      ariaLabel: "Screen",
      // The chosen display shows what is actually on it, kept inside the
      // glyph box either way round, and falls back to the generic glyph until
      // the still has been captured.
      icon: monitorThumbnail ? (
        <img
          alt=""
          className={cn(
            "shrink-0 rounded-sm object-cover",
            isPortraitMonitor ? "h-icon-xl w-auto" : "w-icon-xl",
          )}
          src={monitorThumbnail}
          style={{
            aspectRatio: selectedMonitor
              ? selectedMonitor.size.width / selectedMonitor.size.height
              : undefined,
          }}
        />
      ) : (
        <Monitor />
      ),
      id: "screen",
      label: "Screen",
      onPress: onChooseMonitor,
    },
    {
      ariaLabel: "Window",
      icon: <WindowAppIcon window={selectedWindow} />,
      id: "window",
      label: "Window",
      onPress: onChooseWindow,
    },
    { icon: <SquareDashed />, id: "region", label: "Region" },
    // The camcorder, not the still camera: the still camera is the input
    // toggle beside this control, and one glyph must not mean two things.
    { icon: <Video />, id: "camera", label: "Camera" },
    { icon: <AudioLines />, id: "audio", label: "Audio" },
  ];

  return (
    <PillGroup
      aria-label="Recording type"
      display="icon"
      isDisabled={isDisabled}
      items={items}
      onSelectionChange={(id) => {
        onModeChange(id as RecordingMode);
      }}
      selected={mode}
      size="capture"
      variant="ghost"
    />
  );
}

/**
 * The remembered window's app icon, or the generic window glyph when there is
 * none or its cached file fails to load: without the fallback WebKit paints
 * its broken-image question mark. A new `window` object, which the bar writes
 * after re-extracting the icon, retries the load.
 */
function WindowAppIcon({ window }: { window: WindowDetails | null }) {
  const [failedWindow, setFailedWindow] = useState<WindowDetails | null>(null);

  if (!window?.appIconPath || failedWindow === window) return <AppWindowMac />;
  return (
    <img
      alt=""
      className="size-icon-xl shrink-0 object-contain"
      onError={() => {
        setFailedWindow(window);
      }}
      src={convertFileSrc(window.appIconPath)}
    />
  );
}
