// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { sampleAppIcon } from "../glide-sample-icon";

import { GlideIsland } from "./glide-island";
import { gliding } from "./glide-island-preview";
import { type GlideIslandScript, IDLE, useScript } from "./glide-island-script";

import type { GlideIslandState } from "./glide-island-types";
import type { GlideRegion } from "../detection/glide-regions";

// The island's stories: the island centred on the canvas, where the gesture
// began, holding one state, playing a gesture through, or showing several
// destinations side by side.

export type GlideIslandStageProps = {
  /** Whether the glided app's icon resolves. */
  hasIcon: boolean;
  /** Destinations to show side by side, in place of `state`. */
  regions?: GlideRegion[];
  /** A gesture to play through on a loop, in place of `state`. */
  script?: GlideIslandScript;
  state?: GlideIslandState;
};

export function GlideIslandStage({
  hasIcon,
  regions,
  script,
  state = IDLE,
}: GlideIslandStageProps) {
  const played = useScript(script);
  const showsIcon = hasIcon && (script === undefined || played.icon);
  const iconSrc = showsIcon ? sampleAppIcon : null;
  return (
    <div className="flex size-full items-center justify-center gap-separate">
      {regions ? (
        regions.map((region) => (
          <GlideIsland
            iconSrc={iconSrc}
            key={JSON.stringify(region)}
            pulse={0}
            {...gliding(region)}
          />
        ))
      ) : (
        <GlideIsland
          iconSrc={iconSrc}
          pulse={played.pulse}
          {...(script ? played.state : state)}
        />
      )}
    </div>
  );
}
