// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { StoryGrid, StoryGridRow } from "../../../../storybook/story-grid";
import { useRestingAction } from "../../../../storybook/use-resting-action";
import { timelineScale } from "../timeline-scale";
import { squeezedTime } from "../timeline-story-fixtures";

import { Spring, type SpringProps } from "./spring";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** How wide a second is drawn, as it plays, in px. */
const PX_PER_S = 10;
/** The recording either side of the lull, in seconds: room for its end
 * handle, and enough to see where it starts and stops. */
const MARGIN_S = 2;
/** A lull 12 seconds long as recorded. */
const SPAN = { end: MARGIN_S + 12, start: MARGIN_S };
/** How long the lull plays while it squeezes with the rest. */
const SQUEEZED_S = 4;

type StoryProps = Omit<SpringProps, "scale" | "width">;

/** One lull on a row of the strip's height, the strip as long as the lull
 * plays with a little either side, so it is drawn as the timeline would
 * draw it. Pressing chooses it, and its end and a double-click change how
 * it plays, as the editor would; the drags are kept here and logged to
 * Actions once they rest. */
function LiveSpring(props: StoryProps) {
  const [state, setState] = useState(props);
  const logPlay = useRestingAction(props.onPlayChange);
  const logTrim = useRestingAction(props.onTrim);
  const { span } = state;
  const recorded = span.end - span.start;
  const map = squeezedTime(span.end + MARGIN_S, [
    { ...span, plays: state.plays },
  ]);
  const width = map.duration * PX_PER_S;
  return (
    // Room above for the badge, which stands over the lull.
    <div className="relative mt-6 h-11" dir="ltr" style={{ width }}>
      <Spring
        {...state}
        onPlayChange={(play) => {
          logPlay(play);
          setState({
            ...state,
            play,
            plays:
              play.kind === "keep"
                ? recorded
                : play.kind === "retimed"
                  ? recorded / play.speed
                  : SQUEEZED_S,
          });
        }}
        onPress={() => {
          props.onPress();
          setState({ ...state, isSelected: true });
        }}
        onTrim={(end) => {
          logTrim(end);
          const length = Math.max(0.1, end - span.start);
          setState({
            ...state,
            play: { kind: "keep" },
            plays: length,
            span: { end: span.start + length, start: span.start },
          });
        }}
        scale={timelineScale(map)}
        width={width}
      />
    </div>
  );
}

/** A lull as Controls set it; they start it afresh. */
const Row = (props: StoryProps) => (
  <LiveSpring key={JSON.stringify(props)} {...props} />
);

const meta = {
  args: {
    isAltHeld: false,
    isDeadAir: false,
    isSelected: false,
    moving: [],
    onPlayChange: fn(),
    onPress: fn(),
    onTrim: fn(),
    play: { kind: "squeeze" },
    plays: SQUEEZED_S,
    span: SPAN,
  },
  component: Row,
  parameters: {
    controls: {
      include: ["isAltHeld", "isDeadAir", "isSelected", "play", "plays"],
    },
  },
  title: "Features/Editor/Timeline/Spring",
} satisfies Meta<typeof Row>;

export default meta;
type Story = StoryObj<typeof meta>;

// Each story's stage is as wide as what it draws, so it sits centred and
// never scrolls for room it does not use.
export const Default: Story = {
  decorators: [featureStage({ width: (SQUEEZED_S + MARGIN_S * 2) * PX_PER_S })],
};

// How it plays, as its badge says: squeezed faster, retimed slower than
// recorded, kept as recorded, and dead air not yet dealt with.
export const Plays: Story = {
  decorators: [featureStage({ width: 344 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      {(
        [
          { label: "Faster", play: { kind: "squeeze" }, plays: SQUEEZED_S },
          { label: "Slower", play: { kind: "retimed", speed: 0.5 }, plays: 24 },
          { label: "Kept", play: { kind: "keep" }, plays: 12 },
        ] as const
      ).map(({ label, ...play }) => (
        <StoryGridRow key={label} label={label}>
          <Row {...args} {...play} />
        </StoryGridRow>
      ))}
      <StoryGridRow label="Dead air">
        <Row {...args} isDeadAir play={{ kind: "squeeze" }} plays={12} />
      </StoryGridRow>
    </StoryGrid>
  ),
};

// Squeezed harder, the coils pack tighter, until the spring is shut; its
// badge then waits for the pointer.
export const Squeezed: Story = {
  decorators: [featureStage({ width: 200 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      {[12, 6, 3, 1.5, 0.6].map((plays) => (
        <StoryGridRow key={plays} label={`${String(12 / plays)}×`}>
          <Row {...args} plays={plays} />
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};

// Chosen, its end can be dragged to retime it; with ⌥ held, to trim it.
export const States: Story = {
  decorators: [featureStage({ width: 192 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      <StoryGridRow label="Rest">
        <Row {...args} />
      </StoryGridRow>
      <StoryGridRow label="Selected">
        <Row {...args} isSelected />
      </StoryGridRow>
      <StoryGridRow label="Selected, ⌥ held">
        <Row {...args} isAltHeld isSelected />
      </StoryGridRow>
      <StoryGridRow label="Selected, shut">
        <Row {...args} isSelected plays={0.6} />
      </StoryGridRow>
    </StoryGrid>
  ),
};

// A lull where the picture stays still, and one where it moves by itself
// part of the way, such as a page loading: a screen playing sits on the coil
// there, since that holds down how fast the whole lull plays.
export const Moving: Story = {
  decorators: [featureStage({ width: 304 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid columns={["Rest", "Selected"]}>
      {[
        { label: "Still", moving: [] },
        { label: "Page loading", moving: [{ end: 9, start: 5 }] },
      ].map(({ label, moving }) => (
        <StoryGridRow key={label} label={label}>
          {[false, true].map((isSelected) => (
            <Row
              key={String(isSelected)}
              {...args}
              isSelected={isSelected}
              moving={moving}
              plays={8}
            />
          ))}
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};
