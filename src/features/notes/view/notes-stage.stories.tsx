// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../storybook/feature-story-stage";
import { ProjectBrowser } from "../../project-browser/project-browser";
import { recentProjects } from "../../project-browser/project-browser-fixtures";
import { projectBrowserMeta } from "../../project-browser/project-browser-story-meta";
import { parseNotes } from "../document/notes-document";

import { NotesStage } from "./notes-stage";
import { NotesView } from "./notes-view";

import type { Meta, StoryObj } from "@storybook/react-vite";

// Opening a notes card grows its page out of the card, and Projects draws
// it back in; any other card opens nothing here.

const meta = {
  args: { notes: null, page: null, projects: null },
  component: NotesStage,
  decorators: [featureStage({ window: "projects" })],
  parameters: { controls: { disable: true }, layout: "fullscreen" },
  render: function Render() {
    const [notes, setNotes] = useState<string | null>(null);
    const opened = recentProjects.find(({ file }) => file === notes);
    return (
      <NotesStage
        notes={notes}
        page={
          opened ? (
            <NotesView
              document={parseNotes(opened.notesOpening ?? "")}
              isTaking={false}
              onBack={() => {
                setNotes(null);
              }}
              onChange={fn()}
              onClose={fn()}
              onCopyPath={fn()}
              onHearVoice={fn()}
              onMinimize={fn()}
              onRedo={fn()}
              onRename={fn()}
              onUndo={fn()}
              playingVoice={null}
              title={opened.title}
              voices={[]}
            />
          ) : null
        }
        projects={
          <ProjectBrowser
            {...projectBrowserMeta.args}
            onOpen={(file) => {
              const project = recentProjects.find((one) => one.file === file);
              if (project?.kind === "notes") setNotes(file);
            }}
          />
        }
      />
    );
  },
  title: "Features/Notes/View/Opening",
} satisfies Meta<typeof NotesStage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const FromItsCard: Story = {};
