// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { Switch } from "../../../../primitives/switch/switch";
import { useIsFindingFace } from "../../recording/scenes/use-face-tracking-progress";

/**
 * Whether the camera follows the face: held in the middle of the camera's
 * crop at the zoom the crop gives, outside every scene and in each scene that
 * does not say otherwise. While the face is first found, a footnote says so.
 */
export function CameraFollowFaceRow({
  follows,
  isLocked,
  onChange,
}: {
  follows: boolean;
  isLocked: boolean;
  onChange: (follow: boolean) => void;
}) {
  const isFindingFace = useIsFindingFace();
  return (
    <>
      <ControlRow title={t("editor-panels-follow-face")}>
        {(controlProps) => (
          <Switch
            {...controlProps}
            isDisabled={isLocked}
            isSelected={follows}
            onChange={onChange}
          />
        )}
      </ControlRow>
      {follows && isFindingFace ? (
        <p className="text-caption text-fg-secondary">
          {t("editor-panels-follow-face-finding")}
        </p>
      ) : null}
    </>
  );
}
