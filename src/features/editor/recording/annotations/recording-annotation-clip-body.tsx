// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Camera, Monitor } from "lucide-react";
import { KeyboardEvent, MouseEvent, PointerEvent } from "react";

import { RecordingAnnotationClip } from "./recording-annotations";

/**
 * The body of an annotation clip on the lane: its name, and in a recording
 * with a camera the picture it is drawn on, by the icon its track's row
 * carries, so the lane says which without the clip being opened. The menu
 * key opens the right-click menu from the keyboard, hung off the body.
 */
export function RecordingAnnotationClipBody({
  label,
  onClick,
  onMenu,
  onPointerDown,
  selected,
  showsLabel,
  showsLayer,
  trackId,
}: {
  label: string;
  onClick: (event: MouseEvent) => void;
  onMenu: (bounds: DOMRect) => void;
  onPointerDown: (event: PointerEvent) => void;
  selected: boolean;
  showsLabel: boolean;
  showsLayer: boolean;
  trackId: RecordingAnnotationClip["trackId"];
}) {
  const Icon = trackId === "camera" ? Camera : Monitor;
  const layerName = trackId === "camera" ? "on the camera" : "on the screen";
  const openMenu = (event: KeyboardEvent<HTMLButtonElement>) => {
    const menuKey =
      event.key === "ContextMenu" || (event.key === "F10" && event.shiftKey);
    if (!menuKey) return;
    event.preventDefault();
    event.stopPropagation();
    onMenu(event.currentTarget.getBoundingClientRect());
  };
  return (
    <button
      aria-label={showsLayer ? `${label}, ${layerName}` : label}
      aria-pressed={selected}
      className="flex h-full w-full items-center gap-control px-control-inset text-left focus-visible:outline-2 focus-visible:outline-primary"
      onClick={onClick}
      onKeyDown={openMenu}
      onPointerDown={onPointerDown}
      type="button"
    >
      {showsLayer ? (
        <Icon
          aria-hidden="true"
          className={`size-icon-mini shrink-0 ${selected ? "text-primary-fg" : "text-content-fg-secondary"}`}
        />
      ) : null}
      {showsLabel ? <span className="truncate">{label}</span> : null}
    </button>
  );
}
