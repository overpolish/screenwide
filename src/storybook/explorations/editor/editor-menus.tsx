// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Mic, Monitor, Play, Video, Volume2 } from "lucide-react";

import { IconButton } from "../../../primitives/button/icon-button";
import { ListBox } from "../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../primitives/listbox-item/listbox-item";
import { MenuSection, PanelWindow } from "../shared/panel-window";
import { Waveform } from "../shared/picture";

import { clock, MOMENTS } from "./editor-data";

const TRACKS = [
  { icon: Monitor, id: "screen", name: "Screen" },
  { icon: Video, id: "camera", name: "Camera" },
  { icon: Volume2, id: "system", name: "System audio" },
  { icon: Mic, id: "microphone", name: "Microphone" },
];

// Which tracks the export carries: a native menu of checked items, as a
// menu holds on-off choices, rather than switches in the lanes.
export function TracksMenu() {
  return (
    <PanelWindow className="w-56">
      <ListBox
        aria-label="Tracks"
        className="w-full"
        defaultSelectedKeys={["screen", "system", "microphone"]}
        selectionMode="multiple"
      >
        <MenuSection title="In the export">
          {TRACKS.map(({ icon: Icon, id, name }) => (
            <ListBoxItem id={id} key={id} textValue={name}>
              <span className="flex items-center gap-inline">
                <Icon className="size-icon-sm shrink-0 text-fg-secondary" />
                {name}
              </span>
            </ListBoxItem>
          ))}
        </MenuSection>
      </ListBox>
    </PanelWindow>
  );
}

const NOTE = MOMENTS.find(({ note }) => note);

// A moment's voice note, opened from its pin on the ruler: the moment's own
// colour names its kind, the play button is the one thing to do here.
export function MomentNote() {
  if (!NOTE?.note) return null;
  return (
    <PanelWindow className="w-64 gap-inline p-inset">
      <span className="flex items-center gap-inline">
        <span
          className="size-2 rounded-full"
          style={{ background: NOTE.color }}
        />
        <span className="text-heading">{NOTE.name}</span>
        <span className="text-caption text-fg-tertiary tabular-nums">
          {clock(NOTE.at)}
        </span>
      </span>
      <span className="flex items-center gap-inline">
        <IconButton
          aria-label="Play note"
          className="rounded-full"
          color="primary"
        >
          <Play className="ms-0.5 fill-current" />
        </IconButton>
        <span className="flex h-5 grow text-fg-tertiary">
          <Waveform bars={48} seed={7} />
        </span>
        <span className="text-caption text-fg-tertiary tabular-nums">
          {NOTE.note.duration}
        </span>
      </span>
      <p className="text-body text-fg-secondary">{NOTE.note.transcript}</p>
    </PanelWindow>
  );
}
