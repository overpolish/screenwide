// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Clipboard, Clock, ImageDown, ScrollText } from "lucide-react";

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { IslandListRow } from "../../../primitives/island/island-controls";
import { Switch } from "../../../primitives/switch/switch";

import type { ScreenshotAction } from "../types";
import type { IslandReplay } from "./island-types";

// The parts the island grows into for a screenshot and for the recording's
// options. Each list is read top to bottom and chosen from with one press;
// nothing here asks for typing.

export function ScreenshotPanel({
  canScroll,
  onScreenshot,
}: {
  canScroll: boolean;
  onScreenshot: (action: ScreenshotAction | "delayed" | "save") => void;
}) {
  const items: {
    action: ScreenshotAction | "delayed" | "save";
    icon: typeof Clock;
    label: string;
  }[] = [
    {
      action: "save",
      icon: ImageDown,
      label: t("recording-controls-save-screenshot"),
    },
    {
      action: "clipboard",
      icon: Clipboard,
      label: t("recording-controls-copy-screenshot"),
    },
    {
      action: "delayed",
      icon: Clock,
      label: t("recording-controls-delayed-screenshot"),
    },
    ...(canScroll
      ? [
          {
            action: "scrolling" as const,
            icon: ScrollText,
            label: t("recording-controls-scrolling-screenshot"),
          },
        ]
      : []),
  ];
  return (
    <div className="flex flex-col">
      {items.map(({ action, icon: Icon, label }) => (
        <IslandListRow
          key={action}
          onPress={() => {
            onScreenshot(action);
          }}
        >
          <span className="flex text-fg-secondary">
            <Icon aria-hidden className="size-icon" />
          </span>
          {label}
        </IslandListRow>
      ))}
    </div>
  );
}

export function RecordPanel({
  canStartReplay,
  onReplayChange,
  replay,
}: {
  /** Whether Record could start now, which starting the buffer needs too. */
  canStartReplay: boolean;
  onReplayChange?: (isOn: boolean) => void;
  replay?: IslandReplay;
}) {
  if (!replay) return null;
  return (
    <ControlRow
      className="px-inset-sm"
      title={t("recording-controls-island-replay")}
    >
      {(controlProps) => (
        <Switch
          {...controlProps}
          // Starting it needs what recording would; turning it off is always
          // allowed, whatever the island is set to now.
          isDisabled={
            replay.status === "starting" ||
            (replay.status === "off" && !canStartReplay)
          }
          isSelected={replay.status !== "off"}
          onChange={onReplayChange}
        />
      )}
    </ControlRow>
  );
}
