// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Check,
  Eraser,
  Highlighter,
  Pen,
  Square,
  Trash2,
  Undo2,
} from "lucide-react";
import { type ReactNode, useState } from "react";

import { PREVIEW_NOTCH } from "../../storybook/preview-notch";
import { StoryNotch } from "../../storybook/story-notch";
import { Alert } from "../alert/alert";
import { Button } from "../button/button";
import { IconButton, IconToggleButton } from "../button/icon-button";
import { ListBox } from "../listbox/listbox";
import { ListBoxItem } from "../listbox-item/listbox-item";
import { PillGroup } from "../pill-group/pill-group";

import { Island } from "./island";
import { IslandRow } from "./island-row";
import { IslandOptions } from "./options/island-options";

import type { IslandAnchor } from "./island-geometry";
import type { Meta, StoryObj } from "@storybook/react-vite";

const COLOURS = ["#f5f5f4", "#ef4444", "#f59e0b", "#22c55e", "#3b82f6"];
const TOOLS = [
  { icon: Pen, label: "Pen" },
  { icon: Highlighter, label: "Highlighter" },
  { icon: Square, label: "Box" },
];

type Part = "clear" | "colour" | "warning";

function IslandDemo({
  hasNotch = false,
  initialPart = null,
  isShown = true,
}: {
  hasNotch?: boolean;
  initialPart?: Part | null;
  isShown?: boolean;
}) {
  const [part, setPart] = useState<Part | null>(initialPart);
  const [tool, setTool] = useState("Pen");
  const [colour, setColour] = useState(COLOURS[1]);
  const toggle = (next: Part) => {
    setPart((current) => (current === next ? null : next));
  };
  const sections: Record<Part, { anchor: IslandAnchor; content: ReactNode }> = {
    clear: {
      anchor: "end",
      content: (
        <ListBox
          aria-label="Clear"
          onAction={() => {
            setPart(null);
          }}
        >
          {["Clear this screen", "Clear every screen"].map((label) => (
            <ListBoxItem id={label} key={label} showsSelection={false}>
              <span className="flex items-center gap-inline">
                <span className="flex text-fg-secondary">
                  <Trash2 aria-hidden className="size-icon" />
                </span>
                {label}
              </span>
            </ListBoxItem>
          ))}
        </ListBox>
      ),
    },
    colour: {
      anchor: "start",
      content: (
        <PillGroup
          aria-label="Colour"
          items={COLOURS.map((swatch) => ({
            ariaLabel: swatch,
            icon: (
              <span
                className="size-icon rounded-full"
                style={{ backgroundColor: swatch }}
              />
            ),
            id: swatch,
          }))}
          onSelectionChange={setColour}
          selected={colour}
        />
      ),
    },
    warning: {
      anchor: "center",
      content: (
        <Alert action={<Button>Allow</Button>} color="error">
          Screenwide needs permission to draw over other apps.
        </Alert>
      ),
    },
  };
  const open = part ? sections[part] : null;
  return (
    <div className="relative h-72 w-[640px]">
      <div className="absolute inset-x-0" style={{ top: hasNotch ? 0 : 6 }}>
        <Island
          anchor={open?.anchor}
          aria-label="Drawing tools"
          isShown={isShown}
          notch={hasNotch ? PREVIEW_NOTCH : null}
          row={
            <IslandRow
              end={
                <>
                  <IconButton aria-label="Undo">
                    <Undo2 aria-hidden className="size-icon" />
                  </IconButton>
                  <IslandOptions
                    isOpen={part === "clear"}
                    label="Clear options"
                    onOpen={() => {
                      toggle("clear");
                    }}
                  >
                    <IconButton aria-label="Erase">
                      <Eraser aria-hidden className="size-icon" />
                    </IconButton>
                  </IslandOptions>
                  <Button
                    aria-label="Done"
                    className="w-12 px-0"
                    color="primary"
                    onPress={() => {
                      toggle("warning");
                    }}
                  >
                    <Check aria-hidden className="size-icon" />
                  </Button>
                </>
              }
              start={
                <>
                  {TOOLS.map(({ icon: Icon, label }) =>
                    label === "Pen" ? (
                      <IslandOptions
                        isOpen={part === "colour"}
                        key={label}
                        label="Colour"
                        onOpen={() => {
                          toggle("colour");
                        }}
                      >
                        <IconToggleButton
                          aria-label={label}
                          isSelected={tool === label}
                          onChange={() => {
                            setTool(label);
                          }}
                        >
                          <Icon aria-hidden className="size-icon" />
                        </IconToggleButton>
                      </IslandOptions>
                    ) : (
                      <IconToggleButton
                        aria-label={label}
                        isSelected={tool === label}
                        key={label}
                        onChange={() => {
                          setTool(label);
                        }}
                      >
                        <Icon aria-hidden className="size-icon" />
                      </IconToggleButton>
                    ),
                  )}
                </>
              }
            />
          }
          section={open?.content}
        />
      </div>
      {hasNotch ? <StoryNotch notch={PREVIEW_NOTCH} /> : null}
    </div>
  );
}

const meta = {
  argTypes: {
    isShown: { table: { defaultValue: { summary: "true" } } },
  },
  args: { isShown: true },
  component: IslandDemo,
  decorators: [
    (Story, { args, globals }) =>
      Story({ args: { ...args, hasNotch: globals.notch !== "off" } }),
  ],
  parameters: {
    controls: { include: ["isShown"] },
    layout: "centered",
    notch: true,
  },
  title: "Primitives/Island/Shape",
} satisfies Meta<typeof IslandDemo>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Row: Story = {};

export const SectionAtStart: Story = { args: { initialPart: "colour" } };

export const SectionAtEnd: Story = { args: { initialPart: "clear" } };

export const Warning: Story = { args: { initialPart: "warning" } };
