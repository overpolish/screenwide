// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronsLeftRight, Folder, User } from "lucide-react";
import { type ReactNode } from "react";

import { cn } from "../../../lib/styling";
import { Badge } from "../../../primitives/badge/badge";
import { TextField } from "../../../primitives/input-fields/text-field";
import { ListBoxItem } from "../../../primitives/listbox-item/listbox-item";
import { Select } from "../../../primitives/select/select";
import { Scene, Waveform } from "../shared/picture";

import { FILE_NAME } from "./export-data";

type Choice = { id: string; label: string };

/** A choice whose list opens in the native panel: only its trigger is
 * drawn. */
export function Choose({
  className,
  icon,
  items,
  label,
  selected,
}: {
  className: string;
  items: Choice[];
  label: string;
  icon?: ReactNode;
  /** The item chosen at first; the first one when not given. */
  selected?: string;
}) {
  return (
    <Select
      aria-label={label}
      className={className}
      defaultValue={selected ?? items[0]?.id}
      items={items}
      leftSection={icon}
    >
      {(item: Choice) => <ListBoxItem id={item.id}>{item.label}</ListBoxItem>}
    </Select>
  );
}

/** Where it goes and the name to save under, on one line as a path reads:
 * the folder, then the name typed in place, then the format, with the size
 * the file comes out at when no quality choice already says it. */
export function FileLine({
  extension,
  size,
}: {
  extension: string;
  size?: string;
}) {
  return (
    <div className="flex items-center gap-inline">
      <Choose
        className="w-32 shrink-0"
        icon={<Folder />}
        items={[
          { id: "desktop", label: "Desktop" },
          { id: "movies", label: "Movies" },
        ]}
        label="Save to"
      />
      <TextField
        aria-label="File name"
        className="min-w-0 grow"
        defaultValue={FILE_NAME}
      />
      <Badge>{extension.toUpperCase()}</Badge>
      {size ? <Badge>{size}</Badge> : null}
    </div>
  );
}

/** The frame under the playhead, the original against the export, split
 * where the divider sits. The divider's knob is a knob, light on any
 * picture. */
export function ComparePreview({
  isComparing = true,
}: {
  isComparing?: boolean;
}) {
  return (
    <div
      className="relative overflow-hidden rounded-panel inset-ring inset-ring-divider"
      dir="ltr"
    >
      <Scene className="w-full" />
      {isComparing ? (
        <>
          {/* The export's side, a touch softer, as compression leaves it. */}
          <span className="absolute inset-y-0 end-0 w-1/2 backdrop-blur-[0.8px]" />
          <span className="absolute inset-y-0 start-1/2 w-0.5 -translate-x-1/2 bg-white/90" />
          <span className="absolute start-1/2 top-1/2 flex size-6 -translate-1/2 items-center justify-center rounded-full text-on-knob control-knob">
            <ChevronsLeftRight className="size-icon-sm" />
          </span>
          <PreviewTag className="start-inset-sm">Original</PreviewTag>
          <PreviewTag className="end-inset-sm">Export</PreviewTag>
        </>
      ) : null}
    </div>
  );
}

// Over a picture, which is light or dark in itself, so the tag keeps the
// dark appearance on a shade of its own.
function PreviewTag({
  children,
  className,
}: {
  children: ReactNode;
  className: string;
}) {
  return (
    <span
      className={cn(
        "absolute top-inset-sm rounded-part bg-black/50 appearance-dark",
        className,
      )}
    >
      <Badge className="bg-transparent text-fg">{children}</Badge>
    </span>
  );
}

/** The camera's own frame, for a camera saved as its own file. */
export function CameraFrame() {
  return (
    <span className="flex aspect-video items-end justify-center overflow-hidden rounded-panel bg-linear-to-b from-[#5b6b7d] to-[#2f3a47] inset-ring inset-ring-divider">
      <User className="size-[70%] translate-y-[18%] stroke-1 text-white/70" />
    </span>
  );
}

/** A recording with no picture: its two voices, in a recess as the
 * editor's picture is. */
export function SoundPreview() {
  return (
    <div className="flex h-28 flex-col gap-1 rounded-panel bg-recess p-inset">
      <span className="flex min-h-0 grow text-fg-tertiary">
        <Waveform bars={110} seed={1} />
      </span>
      <span className="flex min-h-0 grow text-fg-secondary">
        <Waveform bars={110} seed={4} />
      </span>
    </div>
  );
}
