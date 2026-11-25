// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listedFormats } from "../../recording/formats/format-name";
import { useFrameFormats } from "../../recording/formats/frame-formats-context";
import { Minimap, type MinimapProps } from "../minimap/minimap";
import {
  type MinimapVersion,
  MinimapVersions,
} from "../minimap/minimap-versions";

import { type StripViewNow, useStripViewNow } from "./strip-view-now";

import type { ToolPanelFormats } from "../../tools/tool-panel-frame";

/** The versions as their lines show them, each named by its formats. */
const versionsOf = (snapshot: ToolPanelFormats): MinimapVersion[] => {
  const named = listedFormats(snapshot);
  return (snapshot.versions ?? []).flatMap((version) => {
    if (version.formats.length === 0) return [];
    const [first] = version.formats;
    const names = version.formats.map(
      (id) => named.find((each) => each.id === id)?.name ?? "",
    );
    const isInHand = snapshot.formatId
      ? version.formats.includes(snapshot.formatId)
      : false;
    return [
      {
        formatId: isInHand && snapshot.formatId ? snapshot.formatId : first,
        isInHand,
        isPlaced: version.isPlaced,
        key: first,
        marks: version.marks,
        names,
      },
    ];
  });
};

/** The strip's minimap, its window following the view as the strip pans,
 * drawn again on its own without the strip; under it, while the
 * recording's formats come out in more than one version, a line for each,
 * as wide as the minimap so its marks sit under what they mark. */
export function StripMinimap({
  now,
  ...props
}: Omit<Extract<MinimapProps, { isPicture?: false }>, "view"> & {
  now: StripViewNow;
}) {
  const view = useStripViewNow(now);
  const formats = useFrameFormats();
  const versions = formats ? versionsOf(formats.snapshot) : [];
  // Always the same tree, so the minimap stays put while the lines open
  // and close under it rather than being drawn anew.
  return (
    <div className="flex flex-col">
      {/* The view window's frame reaches 4 px below the minimap's track
          (`-inset-y-1`); room for it here keeps the gap under the strip as
          wide to the eye as the one under the transport. */}
      <div className="pb-1">
        <Minimap {...props} view={view} />
      </div>
      <MinimapVersions
        map={props.map}
        onSwitch={(id) => {
          formats?.onPatch({ id, kind: "switch" });
        }}
        taken={props.taken}
        versions={versions}
      />
    </div>
  );
}
