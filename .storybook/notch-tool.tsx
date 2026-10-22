// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Storybook bundles manager code with the classic JSX runtime, which needs
// `React` in scope.
import * as React from "react";
import { Button } from "storybook/internal/components";
import { addons, types, useGlobals, useParameter } from "storybook/manager-api";

// The MacBook notch, as a toolbar toggle like the theme's: one press flips
// it, and its label says which it is. It shows only on stories that lay out
// around the notch, which set `parameters.notch`; no other story has a use
// for it.

const ADDON_ID = "screenwide/notch";
const TOOL_ID = `${ADDON_ID}/tool`;

const TITLES = { off: "Show the notch", on: "Hide the notch" } as const;
const LABELS = { off: "no notch", on: "notch" } as const;

// A screen's top edge with the notch cut into it, stroked to sit alongside
// Storybook's own icons.
function NotchIcon({ isOn }: { isOn: boolean }) {
  return (
    <svg
      aria-hidden
      fill="none"
      height={14}
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
      width={14}
    >
      <rect height="16" rx="2" width="20" x="2" y="4" />
      {isOn ? (
        <path d="M9 4v2a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1V4" fill="currentColor" />
      ) : null}
    </svg>
  );
}

function NotchTool() {
  const isUsed = useParameter<boolean>("notch", false);
  const [globals, updateGlobals] = useGlobals();
  if (!isUsed) return null;
  const isOn = globals.notch !== "off";
  return (
    <Button
      ariaLabel={isOn ? TITLES.on : TITLES.off}
      key={TOOL_ID}
      onClick={() => {
        updateGlobals({ notch: isOn ? "off" : "on" });
      }}
      padding="small"
      tooltip={isOn ? TITLES.on : TITLES.off}
      variant="ghost"
    >
      <NotchIcon isOn={isOn} />
      {isOn ? LABELS.on : LABELS.off}
    </Button>
  );
}

addons.register(ADDON_ID, () => {
  addons.add(TOOL_ID, {
    match: ({ viewMode }) => viewMode === "story" || viewMode === "docs",
    render: () => <NotchTool />,
    title: TITLES.on,
    type: types.TOOL,
  });
});
