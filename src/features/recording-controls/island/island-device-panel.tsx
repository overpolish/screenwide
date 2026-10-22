// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, TrianglesCenterlineDashedVertical } from "lucide-react";
import { type ReactNode, useMemo } from "react";
import { useCollator } from "react-aria";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { IslandListRow } from "../../../primitives/island/island-controls";
import { ScrollArea } from "../../../primitives/scroll-area/scroll-area";
import { Switch } from "../../../primitives/switch/switch";
import { Text } from "../../../primitives/text/text";

import { INPUT_GLYPHS, inputName } from "./island-names";

import type { IslandInput, IslandInputKind } from "./island-types";

// The part an input grows into: its switch, its level or its picture, and
// the devices to record from. The camera and microphone record one; system
// audio records the whole mix, or any number of apps on their own.

/** Centres the chosen device in its list's viewport. Only the list moves:
 * `scrollIntoView` would also scroll the island's clipping boxes, which can
 * be scrolled though they show no scrollbar. */
function revealChosen(viewport: HTMLElement) {
  const chosen = viewport.querySelector<HTMLElement>('[aria-pressed="true"]');
  if (!chosen) return;
  const offset =
    chosen.getBoundingClientRect().top -
    viewport.getBoundingClientRect().top +
    viewport.scrollTop;
  viewport.scrollTop =
    offset - (viewport.clientHeight - chosen.offsetHeight) / 2;
}

/** The largest the camera's picture is drawn, in px: the part's width, and
 * a height that keeps the island on the screen for a portrait camera. */
const CAMERA_FRAME = { height: 216, width: 320 };

/** The camera's picture in its own shape, as large as fits the frame, so
 * nothing of it is cut away; a portrait camera stands narrow in the middle.
 * Flip sits on the picture it mirrors, so the result is seen as it is set. */
function CameraFrame({
  aspectRatio = 16 / 9,
  children,
  isFlipped,
  onFlipChange,
}: {
  children: ReactNode;
  isFlipped: boolean;
  aspectRatio?: number;
  onFlipChange?: (isFlipped: boolean) => void;
}) {
  const width = Math.min(CAMERA_FRAME.width, CAMERA_FRAME.height * aspectRatio);
  return (
    <div
      className="relative self-center overflow-hidden rounded-control bg-fill"
      style={{ height: width / aspectRatio, width }}
    >
      {children}
      {/* Translucent black, the island's own colour, reads over any
          picture; on, it is chosen and takes the primary colour. */}
      <button
        aria-label={t("recording-controls-flip")}
        aria-pressed={isFlipped}
        className={cn(
          "absolute end-2 bottom-2 flex size-island-control items-center justify-center rounded-full backdrop-blur-sm transition-colors",
          isFlipped
            ? "bg-primary text-on-primary hover:bg-primary-hover"
            : "bg-island/60 text-fg hover:bg-island/80",
        )}
        onClick={() => onFlipChange?.(!isFlipped)}
        title={t("recording-controls-flip")}
        type="button"
      >
        <TrianglesCenterlineDashedVertical aria-hidden className="size-icon" />
      </button>
    </div>
  );
}

/** What pressing a device chooses. The camera and microphone take the one
 * pressed. System audio takes the whole mix alone, or adds and removes apps,
 * falling back to the whole mix rather than to nothing. */
function nextChoice(kind: IslandInputKind, input: IslandInput, id: string) {
  const pinned = input.devices
    .filter(({ isPinned }) => isPinned)
    .map((device) => device.id);
  if (kind !== "systemAudio" || pinned.includes(id)) return [id];
  const apps = input.selectedIds.filter((chosen) => !pinned.includes(chosen));
  const next = apps.includes(id)
    ? apps.filter((chosen) => chosen !== id)
    : [...apps, id];
  return next.length > 0 ? next : pinned.slice(0, 1);
}

export function DevicePanel({
  input,
  isFlipped = false,
  kind,
  onDeviceChange,
  onFlipChange,
  onToggle,
  preview,
}: {
  input: IslandInput;
  kind: IslandInputKind;
  onDeviceChange: (ids: string[]) => void;
  onToggle: (isOn: boolean) => void;
  isFlipped?: boolean;
  onFlipChange?: (isFlipped: boolean) => void;
  preview?: ReactNode;
}) {
  // Pinned devices, such as system audio's whole mix, lead; the rest are in
  // the order of the reader's language.
  const collator = useCollator({ sensitivity: "base" });
  const devices = useMemo(
    () => [
      ...input.devices.filter(({ isPinned }) => isPinned),
      ...input.devices
        .filter(({ isPinned }) => !isPinned)
        .sort((a, b) => collator.compare(a.label, b.label)),
    ],
    [collator, input.devices],
  );
  const hasIcons = devices.some(({ icon }) => icon !== undefined);
  const Glyph = INPUT_GLYPHS[kind].on;
  return (
    // The camera's part is as wide as its picture; the others take only the
    // width their longest device needs, up to the same.
    <div
      className={cn(
        "flex flex-col gap-inline",
        kind === "camera" ? "w-[320px]" : "w-max max-w-[320px]",
      )}
    >
      <div className="flex items-center justify-between gap-separate px-inset-sm">
        <Text variant="label">{inputName(kind)}</Text>
        <Switch
          aria-label={inputName(kind)}
          isSelected={input.isOn}
          onChange={onToggle}
        />
      </div>
      {kind === "camera" && input.isOn ? (
        <CameraFrame
          aspectRatio={
            devices.find(({ id }) => input.selectedIds.includes(id))
              ?.aspectRatio
          }
          isFlipped={isFlipped}
          onFlipChange={onFlipChange}
        >
          {preview}
        </CameraFrame>
      ) : null}
      {/* Off, the level bar stays and is disabled with the devices, so the
          part keeps its shape when the input is switched. */}
      {kind !== "camera" ? (
        <span
          className={cn(
            "mx-inset-sm h-1 overflow-hidden rounded-full bg-fill-strong transition-opacity",
            !input.isOn && "opacity-50",
          )}
        >
          <span
            className="block h-full rounded-full bg-fg transition-[width] duration-75"
            style={{
              width: `${String(input.isOn ? (input.level ?? 0) * 100 : 0)}%`,
            }}
          />
        </span>
      ) : null}
      {input.warning ? (
        <Text className="px-inset-sm text-error" variant="caption">
          {input.warning}
        </Text>
      ) : null}
      {/* About six devices show; more scroll, so a long list never pushes
          the island down the screen. */}
      <ScrollArea
        className="flex flex-col"
        constrainHeight
        onInitialized={revealChosen}
        rootClassName={cn(
          "max-h-48 transition-opacity",
          !input.isOn && "opacity-50",
        )}
      >
        {devices.map((device) => {
          const isChosen = input.selectedIds.includes(device.id);
          return (
            <IslandListRow
              isChosen={isChosen}
              key={device.id}
              onPress={() => {
                onDeviceChange(nextChoice(kind, input, device.id));
              }}
            >
              <span className="flex w-icon shrink-0 justify-center">
                {isChosen ? (
                  <Check aria-hidden className="size-icon-sm" />
                ) : null}
              </span>
              {/* Apps show their icons; the whole mix, which has none,
                  shows the input's own glyph in their place. */}
              {hasIcons ? (
                device.icon ? (
                  <img
                    alt=""
                    className="size-icon shrink-0"
                    draggable={false}
                    src={device.icon}
                  />
                ) : (
                  <span className="flex shrink-0 text-fg-secondary">
                    <Glyph aria-hidden className="size-icon" />
                  </span>
                )
              ) : null}
              {/* The scrollbar is drawn over the list, so the name keeps
                  clear of it; the row's highlight still runs to the edge. */}
              <span className="truncate pe-inset" dir="auto">
                {device.label}
              </span>
            </IslandListRow>
          );
        })}
      </ScrollArea>
    </div>
  );
}
