// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { expect, fn, userEvent, waitFor, within } from "storybook/test";

import { t } from "../../../../i18n/i18n";
import { platformModifiers } from "../../../../lib/hotkey";
import { isWindows } from "../../../../lib/platform";
import { featureStage } from "../../../../storybook/feature-story-stage";
import { StoryGrid, StoryGridRow } from "../../../../storybook/story-grid";
import { conflictOf } from "../shortcut-conflicts";

import { HotkeyField } from "./hotkey-field";

import type { Meta, StoryObj } from "@storybook/react-vite";

const DEFAULT_SHORTCUT = "CommandOrControl+Shift+KeyR";

// Another shortcut already on ⌘⇧S (Ctrl+Shift+S on Windows), as Take a
// screenshot is by default, and the system's and apps' own combinations.
const storyConflicts = (value: string) =>
  conflictOf({
    isMac: !isWindows(),
    shortcuts: [
      {
        id: "screenshot",
        kind: "combo",
        label: "Take a screenshot",
        scope: "anywhere",
        value: "CommandOrControl+Shift+KeyS",
      },
    ],
    subject: { id: "story", scope: "anywhere" },
    value,
  });

const meta = {
  argTypes: {
    captureMode: {
      control: "inline-radio",
      options: ["shortcut", "single-control", "local-shortcut"],
      table: { defaultValue: { summary: "shortcut" } },
    },
    isClearable: {
      control: "boolean",
      table: { defaultValue: { summary: "true" } },
    },
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    value: { control: "text" },
  },
  args: {
    "aria-label": "Show recording controls",
    captureMode: "shortcut",
    conflictOf: storyConflicts,
    isClearable: true,
    isDisabled: false,
    onCaptureChange: fn(),
    onChange: fn(),
    value: DEFAULT_SHORTCUT,
  },
  component: HotkeyField,
  decorators: [featureStage({ width: 420 })],
  parameters: {
    controls: {
      include: ["captureMode", "isClearable", "isDisabled", "value"],
    },
    layout: "fullscreen",
  },
  // A recorded shortcut lands in the args, as Settings would save it.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <div className="flex justify-end">
        <HotkeyField
          {...args}
          onChange={(value) => {
            args.onChange(value);
            updateArgs({ value });
          }}
        />
      </div>
    );
  },
  title: "Features/Settings/Shortcuts/Hotkey Field",
} satisfies Meta<typeof HotkeyField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// Set, not set, and changed from its default, which offers the reset.
function StateGrid({ isDisabled }: { isDisabled?: boolean }) {
  const rows = [
    { label: "set", value: DEFAULT_SHORTCUT },
    { label: "not set", value: null },
    { label: "changed", value: "CommandOrControl+Alt+KeyS" },
  ];
  return (
    <StoryGrid>
      {rows.map(({ label, value }) => (
        <StoryGridRow key={label} label={label}>
          <HotkeyField
            aria-label="Show recording controls"
            defaultValue={DEFAULT_SHORTCUT}
            isDisabled={isDisabled}
            onChange={() => undefined}
            value={value}
          />
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: () => <StateGrid />,
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: () => <StateGrid isDisabled />,
};

// Press the field, then a key alone. The field waits for the app to set its
// own shortcuts aside before it reads keys.
const pressInField = async (canvasElement: HTMLElement, keys: string) => {
  const field = within(canvasElement).getByRole("button", { pressed: false });
  await userEvent.click(field);
  await waitFor(async () => {
    await expect(field).toHaveAttribute("aria-pressed", "true");
    await expect(field).not.toHaveTextContent(t("hotkey-field-preparing"));
  });
  await userEvent.keyboard(keys);
};

// S alone: its combinations, best first; the one Take a screenshot has is
// marked and cannot be chosen, and the first free one is in hand.
export const Suggestions: Story = {
  play: async ({ canvasElement }) => {
    await pressInField(canvasElement, "s");
    await expect(
      within(canvasElement).findByRole("radiogroup"),
    ).resolves.toBeVisible();
  },
};

// A whole chord the apps already use (Copy): not taken unseen, but offered
// marked beside the free ones, which come first.
export const ReservedChord: Story = {
  play: async ({ canvasElement }) => {
    await pressInField(
      canvasElement,
      isWindows() ? "{Control>}c{/Control}" : "{Meta>}c{/Meta}",
    );
  },
};

// A modifier added with its toggle: Option (Alt on Windows) on top of the
// combination in hand.
export const ModifierToggles: Story = {
  play: async ({ canvasElement }) => {
    await pressInField(canvasElement, "k");
    const toggles = within(
      within(canvasElement).getByRole("group"),
    ).getAllByRole("button");
    await userEvent.click(toggles[1]);
  },
};

// Shift alone would type a capital, so the field asks for another modifier.
export const ShiftOnly: Story = {
  play: async ({ canvasElement }) => {
    await pressInField(canvasElement, "k");
    const toggles = within(
      within(canvasElement).getByRole("group"),
    ).getAllByRole("button");
    const shift = platformModifiers().indexOf("Shift");
    for (const [index, toggle] of toggles.entries())
      if (
        (toggle.getAttribute("aria-pressed") === "true") !==
        (index === shift)
      )
        await userEvent.click(toggle);
  },
};

export const Windows: Story = {
  parameters: { platform: "windows" },
  play: Suggestions.play,
};
