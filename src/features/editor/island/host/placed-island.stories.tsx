// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { beatIsland } from "../beat/beat-island";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";
import { modelWorkIsland } from "../model-work/model-work-island";

import { type PictureInset } from "./editor-island-kind";
import { useIslandBounds } from "./island-bounds";
import { PlacedIsland } from "./placed-island";

import type { ModelWork } from "../../../../bindings/ModelWork";
import type { Meta, StoryObj } from "@storybook/react-vite";

/** The picture in the window, as the editor lays it out: the tool column at
 * its start and the strip under it. */
const INSET: PictureInset = { bottom: 180, left: 64, right: 0, top: 0 };

type ClashProps = EditorIslandStoryProps & {
  /** Where the beat chosen on the strip sits, in percent of the picture's
   * width from its start. */
  itemAt: number;
  onCancel: (work: ModelWork) => void;
  onCut: () => void;
};

/** The models' work in the picture's bottom start corner and a beat's
 * island over its item, placed as the island window places them. */
function Clash({ isShown = true, itemAt, onCancel, onCut }: ClashProps) {
  const work = useIslandBounds();
  const at =
    INSET.left +
    (itemAt / 100) * (window.innerWidth - INSET.left - INSET.right);
  return (
    <div className="dark relative size-full text-fg" dir="ltr">
      {/* Where the beat sits along the picture's foot. */}
      <div
        aria-hidden
        className="absolute h-4 w-px bg-primary"
        style={{ bottom: INSET.bottom - 16, insetInlineStart: at }}
      />
      <PlacedIsland
        at={at}
        clearOf={work.bounds}
        inset={INSET}
        onParts={() => undefined}
        shown={
          isShown
            ? beatIsland({
                beat: { kind: "click", label: "New Project" },
                id: "beat",
                onCut,
                onPlayAsRecorded: () => undefined,
                speed: 1,
              })
            : null
        }
      />
      <PlacedIsland
        corner="start"
        edge="bottom"
        inset={INSET}
        onParts={work.track}
        shown={
          isShown
            ? modelWorkIsland({
                jobs: [
                  { fraction: 0.42, secondsLeft: 150, work: "restoreVoice" },
                  { fraction: 0.8, secondsLeft: 20, work: "followFace" },
                ],
                onCancel,
              })
            : null
        }
      />
    </div>
  );
}

const meta = {
  ...editorIslandStoryMeta,
  argTypes: {
    ...editorIslandStoryMeta.argTypes,
    itemAt: { control: { max: 100, min: 0, step: 1, type: "range" } },
  },
  args: { isShown: true, itemAt: 10, onCancel: fn(), onCut: fn() },
  component: Clash,
  decorators: [featureStage({ window: "editor-recording" })],
  parameters: {
    ...editorIslandStoryMeta.parameters,
    controls: { include: ["isShown", "itemAt"] },
  },
  title: "Features/Editor/Island/Clash",
} satisfies Meta<typeof Clash>;

export default meta;
type Story = StoryObj<typeof meta>;

export const NearTheStart: Story = {};
