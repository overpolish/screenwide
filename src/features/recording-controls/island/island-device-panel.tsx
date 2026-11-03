// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { TrianglesCenterlineDashedVertical } from "lucide-react";
import { type ReactNode, useMemo } from "react";
import { useCollator, useMeter } from "react-aria";
import { ToggleButton } from "react-aria-components";

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";
import { ListBox } from "../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../primitives/listbox-item/listbox-item";
import { revealChosen } from "../../../primitives/scroll-area/reveal-chosen";
import { ScrollArea } from "../../../primitives/scroll-area/scroll-area";
import { Switch } from "../../../primitives/switch/switch";
import { Text } from "../../../primitives/text/text";

import { INPUT_GLYPHS, inputName } from "./island-names";

import type { IslandInput, IslandInputKind } from "./island-types";

// The part an input grows into: its switch, its level or its picture, and
// the devices to record from. The camera and microphone record one; system
// audio records the whole mix, or any number of apps on their own.

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
          picture; on, it is a held toggle, raised in the primary colour
          as one is everywhere, since the island's fill would not read
          over the picture. */}
      <ToggleButton
        aria-label={t("recording-controls-flip")}
        className={cn(
          "absolute end-2 bottom-2 flex size-island-control cursor-default items-center justify-center rounded-full backdrop-blur-sm transition-colors",
          isFlipped
            ? "bg-primary text-on-primary primary-raised data-[hovered]:bg-primary-hover data-[pressed]:bg-primary-pressed data-[pressed]:primary-raised-pressed"
            : "bg-island/60 text-fg data-[hovered]:bg-island/80",
          focusStyles,
          elementFocusVisible,
        )}
        isSelected={isFlipped}
        onChange={onFlipChange}
      >
        <TrianglesCenterlineDashedVertical aria-hidden className="size-icon" />
      </ToggleButton>
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

/** An input's level along the part, as its glyph shows it on the row. Off,
 * it stays at nothing, dimmed with the devices, so the part keeps its
 * shape when the input is switched. */
function PanelLevel({
  isOn,
  kind,
  level = 0,
}: {
  isOn: boolean;
  kind: "microphone" | "systemAudio";
  level?: number;
}) {
  const value = isOn ? level * 100 : 0;
  const { meterProps } = useMeter({
    "aria-label":
      kind === "microphone"
        ? t("recording-controls-microphone-level")
        : t("recording-controls-system-audio-level"),
    value,
  });
  return (
    <span
      {...meterProps}
      className={cn(
        "h-1 overflow-hidden rounded-full bg-fill-strong transition-opacity",
        !isOn && "opacity-40",
      )}
    >
      <span
        className="block h-full rounded-full bg-fg transition-[width] duration-75"
        style={{ width: `${String(value)}%` }}
      />
    </span>
  );
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
      <ControlRow title={inputName(kind)}>
        {(controlProps) => (
          <Switch
            {...controlProps}
            isSelected={input.isOn}
            onChange={onToggle}
          />
        )}
      </ControlRow>
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
      {/* It spans the part as the switch and the devices do. */}
      {kind !== "camera" ? (
        <PanelLevel isOn={input.isOn} kind={kind} level={input.level} />
      ) : null}
      {input.warning ? (
        <Text className="px-inset-sm text-error" variant="caption">
          {input.warning}
        </Text>
      ) : null}
      {/* About six devices show; more scroll, so a long list never pushes
          the island down the screen. */}
      <ScrollArea
        constrainHeight
        onInitialized={revealChosen}
        rootClassName="max-h-48"
      >
        {/* Chosen by the island's own rule rather than the list's: the
            camera and microphone take one, system audio the mix or any
            number of apps. So a press reads which device it was, from what
            the list would have changed, and the rule picks the next set. */}
        <ListBox
          aria-label={inputName(kind)}
          // Off, an input's devices are not chosen from; they show what it
          // would record when switched back on.
          disabledKeys={input.isOn ? [] : devices.map(({ id }) => id)}
          onSelectionChange={(keys) => {
            if (keys === "all") return;
            const pressed = devices.find(
              ({ id }) => keys.has(id) !== input.selectedIds.includes(id),
            );
            if (pressed) onDeviceChange(nextChoice(kind, input, pressed.id));
          }}
          selectedKeys={input.selectedIds}
          selectionBehavior="toggle"
          selectionMode="multiple"
        >
          {devices.map((device) => (
            <ListBoxItem
              id={device.id}
              key={device.id}
              textValue={device.label}
            >
              <span className="flex min-w-0 items-center gap-inline">
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
              </span>
            </ListBoxItem>
          ))}
        </ListBox>
      </ScrollArea>
    </div>
  );
}
