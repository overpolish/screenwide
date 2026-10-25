// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AudioLines,
  Flag,
  Keyboard,
  PencilRuler,
  Settings,
  Video,
} from "lucide-react";
import { type ReactNode, useState } from "react";

import { SidebarNav } from "../../../composites/sidebar-nav/sidebar-nav";
import { WindowHeader } from "../../../composites/window-header/window-header";
import { WindowShell } from "../../../composites/window-shell/window-shell";
import { TranscriptionSettingsPanel } from "../../../features/settings/sections/transcription-settings";
import { Alert } from "../../../primitives/alert/alert";

import { GeneralPane } from "./panes/general-pane";
import { MomentsPane } from "./panes/moments-pane";
import { Pane } from "./panes/pane";
import { RecordingPane } from "./panes/recording-pane";
import { ToolsPane } from "./panes/tools-pane";
import { useExploredSettings } from "./settings-state";
import { ShortcutsPane } from "./shortcuts/shortcuts-pane";

// Settings, reworked so each setting shows what it does. The window keeps
// the production shell (title bar, sidebar, boxed groups of rows) and adds a
// picture above the rows of each pane where a picture explains more than
// words: the live island for Recording and Moments, the keyboard for
// Shortcuts, and each tool at work in Tools. Recording leaves General, which
// keeps the settings about the app itself.

export type ExploredSection =
  "general" | "models" | "moments" | "recording" | "shortcuts" | "tools";

const SECTIONS: { icon: ReactNode; id: ExploredSection; label: string }[] = [
  { icon: <Settings />, id: "general", label: "General" },
  { icon: <Video />, id: "recording", label: "Recording" },
  { icon: <PencilRuler />, id: "tools", label: "Tools" },
  { icon: <Flag />, id: "moments", label: "Moments" },
  { icon: <AudioLines />, id: "models", label: "Models" },
  { icon: <Keyboard />, id: "shortcuts", label: "Shortcuts" },
];

export function ExploredSettingsWindow({
  initialSection = "general",
}: {
  initialSection?: ExploredSection;
}) {
  const settings = useExploredSettings();
  const [section, setSection] = useState<ExploredSection>(initialSection);
  const label =
    SECTIONS.find((candidate) => candidate.id === section)?.label ?? "";

  const pane = {
    general: <GeneralPane settings={settings} />,
    models: (
      <Pane label="Models">
        <TranscriptionSettingsPanel controls={settings.transcription} />
      </Pane>
    ),
    moments: <MomentsPane settings={settings} />,
    recording: <RecordingPane settings={settings} />,
    shortcuts: <ShortcutsPane settings={settings} />,
    tools: <ToolsPane settings={settings} />,
  }[section];

  return (
    <WindowShell header={<WindowHeader title={label} />}>
      <div className="flex min-h-0 grow gap-group ps-inset">
        <div className="flex shrink-0 flex-col gap-separate pb-inset">
          <SidebarNav
            aria-label="Settings sections"
            isExpandable={false}
            isExpanded
            items={SECTIONS}
            onSelectionChange={(id) => {
              setSection(id as ExploredSection);
            }}
            selected={section}
          />
        </div>
        <div className="flex min-h-0 min-w-0 grow flex-col gap-separate">
          {/* Each pane is its own tree, so a pane's picture starts over
              when it is opened again. */}
          <div className="flex min-h-0 grow flex-col" key={section}>
            {pane}
          </div>
          {settings.error ? (
            <Alert className="me-inset mb-inset" color="error" role="alert">
              {settings.error}
            </Alert>
          ) : null}
        </div>
      </div>
    </WindowShell>
  );
}
