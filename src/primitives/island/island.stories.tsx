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

import { StoryNotch } from "../../storybook/story-notch";

import { Island } from "./island";
import {
  IslandButton,
  IslandListRow,
  IslandOptions,
  IslandPrimaryButton,
  IslandWarning,
} from "./island-controls";
import { IslandRow } from "./island-row";

import type { IslandAnchor } from "./island-geometry";
import type { Meta, StoryObj } from "@storybook/react-vite";

const NOTCH = { height: 32, width: 190 };
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
        <div className="flex flex-col">
          {["Clear this screen", "Clear every screen"].map((label) => (
            <IslandListRow
              key={label}
              onPress={() => {
                setPart(null);
              }}
            >
              <span className="flex text-fg-secondary">
                <Trash2 aria-hidden className="size-icon" />
              </span>
              {label}
            </IslandListRow>
          ))}
        </div>
      ),
    },
    colour: {
      anchor: "start",
      content: (
        <div className="flex gap-inline">
          {COLOURS.map((swatch) => (
            <IslandButton
              aria-label={swatch}
              aria-pressed={swatch === colour}
              isOpen={swatch === colour}
              key={swatch}
              onPress={() => {
                setColour(swatch);
              }}
            >
              <span
                className="size-icon rounded-full"
                style={{ backgroundColor: swatch }}
              />
            </IslandButton>
          ))}
        </div>
      ),
    },
    warning: {
      anchor: "center",
      content: (
        <IslandWarning
          action="Allow"
          message="Screenwide needs permission to draw over other apps."
        />
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
          notch={hasNotch ? NOTCH : null}
          row={
            <IslandRow
              end={
                <>
                  <IslandButton aria-label="Undo">
                    <Undo2 aria-hidden className="size-icon" />
                  </IslandButton>
                  <IslandOptions
                    isOpen={part === "clear"}
                    label="Clear options"
                    onOpen={() => {
                      toggle("clear");
                    }}
                  >
                    <IslandButton aria-label="Erase">
                      <Eraser aria-hidden className="size-icon" />
                    </IslandButton>
                  </IslandOptions>
                  <IslandPrimaryButton
                    aria-label="Done"
                    onPress={() => {
                      toggle("warning");
                    }}
                  >
                    <Check aria-hidden className="size-icon" />
                  </IslandPrimaryButton>
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
                        <IslandButton
                          aria-label={label}
                          aria-pressed={tool === label}
                          isOpen={tool === label}
                          onPress={() => {
                            setTool(label);
                          }}
                        >
                          <Icon aria-hidden className="size-icon" />
                        </IslandButton>
                      </IslandOptions>
                    ) : (
                      <IslandButton
                        aria-label={label}
                        aria-pressed={tool === label}
                        isOpen={tool === label}
                        key={label}
                        onPress={() => {
                          setTool(label);
                        }}
                      >
                        <Icon aria-hidden className="size-icon" />
                      </IslandButton>
                    ),
                  )}
                </>
              }
            />
          }
          section={open?.content}
        />
      </div>
      {hasNotch ? <StoryNotch notch={NOTCH} /> : null}
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
  title: "Primitives/Island",
} satisfies Meta<typeof IslandDemo>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Row: Story = {};

export const SectionAtStart: Story = { args: { initialPart: "colour" } };

export const SectionAtEnd: Story = { args: { initialPart: "clear" } };

export const Warning: Story = { args: { initialPart: "warning" } };
