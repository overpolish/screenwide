// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { PREVIEW_DISPLAYS } from "../../../../features/recording-controls/island/island-preview";
import { cn } from "../../../../lib/styling";

import { PICTURE } from "./scene-kit";

// The miniature screen Glide is shown at work on, above its rows in Tools,
// as tall as the other tools' pictures. With Glide off, the picture is the
// same screen with its work gone and its colour drained, so turning it off
// shows what goes with it.

export function ToolScreen({
  children,
  isOn,
}: {
  children: ReactNode;
  isOn: boolean;
}) {
  return (
    <div
      aria-hidden
      className="relative overflow-hidden rounded-control bg-recess"
      dir="ltr"
      inert
      style={{ height: PICTURE.height }}
    >
      <img
        alt=""
        className={cn(
          "absolute inset-0 size-full object-cover transition-[filter,opacity]",
          isOn ? "opacity-80" : "opacity-50 grayscale",
        )}
        src={PREVIEW_DISPLAYS[1].thumbnail}
      />
      {children}
    </div>
  );
}
