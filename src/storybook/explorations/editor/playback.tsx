// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Brackets,
  ClipboardCopy,
  Layers,
  Magnet,
  Minus,
  MoveHorizontal,
  Play,
  Plus,
  Scissors,
} from "lucide-react";
import { type ReactNode, useState } from "react";

import {
  IconButton,
  IconToggleButton,
} from "../../../primitives/button/icon-button";
import { ListBoxItem } from "../../../primitives/listbox-item/listbox-item";
import { Select } from "../../../primitives/select/select";
import { Toolbar } from "../../../primitives/toolbar/toolbar";

import { clock, DURATION_S, PLAYHEAD_S } from "./editor-data";

// The timeline's controls, each kept beside the timeline: playback just
// above it, its tools in a column at its side, its zoom at its head, and the
// picture's zoom on the picture.

type Choice = { id: string; label: string };

// A choice whose list opens in the native panel: only its trigger is drawn.
function Choose({
  className,
  items,
  label,
  selected,
}: {
  className: string;
  items: Choice[];
  label: string;
  selected: string;
}) {
  return (
    <Select
      aria-label={label}
      className={className}
      defaultValue={selected}
      items={items}
    >
      {(item: Choice) => <ListBoxItem id={item.id}>{item.label}</ListBoxItem>}
    </Select>
  );
}

const SPEEDS = ["0.5×", "1×", "1.5×", "2×"].map((label) => ({
  id: label,
  label,
}));

const ZOOMS = ["50%", "100%", "200%", "Fit"].map((label) => ({
  id: label,
  label,
}));

// Play, the time, the speed and Copy Frame. Play is the timeline's one
// primary action.
function PlaybackControls({ hasPicture }: { hasPicture: boolean }) {
  return (
    <span className="flex items-center gap-inline">
      <IconButton aria-label="Play" className="rounded-full" color="primary">
        <Play className="ms-0.5 fill-current" />
      </IconButton>
      <span className="px-1 text-label tabular-nums">
        {clock(PLAYHEAD_S)}
        <span className="text-fg-tertiary"> / {clock(DURATION_S)}</span>
      </span>
      <Choose className="w-20" items={SPEEDS} label="Speed" selected="1×" />
      {hasPicture ? (
        <IconButton aria-label="Copy Frame" variant="ghost">
          <ClipboardCopy />
        </IconButton>
      ) : null}
    </span>
  );
}

// The bar over a timeline at the window's foot: playback, the handle the
// timeline is resized from, and its zoom.
export function PlaybackBar({ hasPicture }: { hasPicture: boolean }) {
  return (
    <div className="relative flex h-10 items-center">
      <PlaybackControls hasPicture={hasPicture} />
      <span className="absolute top-0 start-1/2 h-1 w-8 -translate-x-1/2 rounded-full bg-fill-strong rtl:translate-x-1/2" />
      <Toolbar aria-label="Timeline zoom" className="ms-auto">
        <IconButton aria-label="Zoom timeline out" variant="ghost">
          <Minus />
        </IconButton>
        <IconButton aria-label="Zoom timeline in" variant="ghost">
          <Plus />
        </IconButton>
        <IconButton aria-label="Fit timeline" variant="ghost">
          <MoveHorizontal />
        </IconButton>
      </Toolbar>
    </div>
  );
}

// Blade and Range are tools, one in hand at a time; Snap is a switch that
// stays on, so it fills as anything on does. Tracks opens its menu beside
// it, upward over the picture.
export function TimelineTools({
  menu,
}: {
  /** The open Tracks menu. */
  menu?: ReactNode;
}) {
  const [tool, setTool] = useState<string | null>(null);
  const [isSnapping, setIsSnapping] = useState(true);
  return (
    <Toolbar
      aria-label="Timeline tools"
      className="mt-10 shrink-0"
      orientation="vertical"
    >
      {[
        { icon: Scissors, label: "Blade" },
        { icon: Brackets, label: "Range" },
      ].map(({ icon: Icon, label }) => (
        <IconToggleButton
          aria-label={label}
          isSelected={tool === label}
          key={label}
          onChange={(isSelected) => {
            setTool(isSelected ? label : null);
          }}
        >
          <Icon />
        </IconToggleButton>
      ))}
      <IconToggleButton
        aria-label="Snap to edges"
        isSelected={isSnapping}
        onChange={setIsSnapping}
      >
        <Magnet />
      </IconToggleButton>
      <span className="relative">
        <IconButton
          aria-expanded={menu !== undefined}
          aria-label="Tracks"
          className={menu ? "bg-fill" : undefined}
          variant="ghost"
        >
          <Layers />
        </IconButton>
        {menu ? (
          <span className="absolute start-full bottom-0 z-10 ms-inline">
            {menu}
          </span>
        ) : null}
      </span>
    </Toolbar>
  );
}

// The picture's zoom, in the workspace's corner.
export function PreviewZoom() {
  return (
    <Choose
      className="w-24"
      items={ZOOMS}
      label="Preview zoom"
      selected="100%"
    />
  );
}
