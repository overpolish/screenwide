// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { StoryGrid, StoryGridRow } from "../../../../storybook/story-grid";
import { useRestingAction } from "../../../../storybook/use-resting-action";
import { ANNOTATION_TOOLS } from "../../tool-panels/tool-registry";
import { recordedTime, timelineScale } from "../timeline-scale";

import {
  ANNOTATION_BAR_PX,
  AnnotationBar,
  type AnnotationBarProps,
} from "./annotation-bar";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** How wide a second is drawn, in px. */
const PX_PER_S = 10;
/** Room, in px, each side of a strip for the hit area and handles that
 * reach past a short bar's ends. */
const MARGIN_PX = 24;

/** A strip `seconds` long, with room each side, the bar placed in it. */
function Strip({
  seconds,
  ...props
}: Omit<AnnotationBarProps, "scale" | "width"> & { seconds: number }) {
  const width = seconds * PX_PER_S;
  return (
    <div className="px-6" dir="ltr">
      <div className="relative" style={{ height: ANNOTATION_BAR_PX, width }}>
        <AnnotationBar
          {...props}
          scale={timelineScale(recordedTime(seconds))}
          width={width}
        />
      </div>
    </div>
  );
}

/** One bar alone, filling a strip as long as it is. */
function Fitted({
  seconds,
  ...props
}: Omit<AnnotationBarProps, "scale" | "span" | "width"> & {
  seconds: number;
}) {
  return (
    <Strip {...props} seconds={seconds} span={{ end: seconds, start: 0 }} />
  );
}

/** How long the Default story's strip is, in seconds. */
const STRIP_S = 40;

/** A bar in a strip it can be moved along, pressed to choose it and its
 * ends dragged once chosen, as the editor would; Controls start it
 * afresh. */
function LiveBar(props: Omit<AnnotationBarProps, "scale" | "width">) {
  const [span, setSpan] = useState(props.span);
  const [isSelected, setIsSelected] = useState(props.isSelected);
  const logRetime = useRestingAction(props.onRetime);
  return (
    <Strip
      {...props}
      isSelected={isSelected}
      onRetime={(start, end) => {
        logRetime(start, end);
        setSpan({ end, start });
      }}
      onSelect={() => {
        props.onSelect();
        setIsSelected(true);
      }}
      seconds={STRIP_S}
      span={span}
    />
  );
}

const meta = {
  args: {
    isSelected: false,
    kind: "arrow",
    onRetime: fn(),
    onSelect: fn(),
    scale: timelineScale(recordedTime(STRIP_S)),
    span: { end: 27, start: 13 },
    width: STRIP_S * PX_PER_S,
  },
  component: AnnotationBar,
  parameters: { controls: { include: ["isSelected", "kind", "span"] } },
  title: "Features/Editor/Timeline/Annotation Bar",
} satisfies Meta<typeof AnnotationBar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  decorators: [featureStage({ width: STRIP_S * PX_PER_S + MARGIN_PX * 2 })],
  render: (args) => (
    <LiveBar key={JSON.stringify([args.isSelected, args.span])} {...args} />
  ),
};

// However short, a bar keeps its badge, and a chosen one its handles clear
// of it; its hit area reaches past its ends.
export const States: Story = {
  decorators: [featureStage({ width: 640 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid columns={["Rest", "Selected"]}>
      {[
        { label: "Tiny", seconds: 0.2 },
        { label: "Short", seconds: 3 },
        { label: "Long", seconds: 24 },
      ].map(({ label, seconds }) => (
        <StoryGridRow key={label} label={label}>
          {[false, true].map((isSelected) => (
            <Fitted
              key={String(isSelected)}
              {...args}
              isSelected={isSelected}
              seconds={seconds}
            />
          ))}
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};

export const Tools: Story = {
  decorators: [featureStage({ width: 456 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid columns={["Rest", "Selected"]}>
      {ANNOTATION_TOOLS.map(({ id }) => (
        <StoryGridRow key={id} label={id}>
          {[false, true].map((isSelected) => (
            <Fitted
              key={String(isSelected)}
              {...args}
              isSelected={isSelected}
              kind={id}
              seconds={14}
            />
          ))}
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};
