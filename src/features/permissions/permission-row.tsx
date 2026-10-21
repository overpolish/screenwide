// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import { ReactNode, useId } from "react";

import { ControlRow } from "../../composites/control-row/control-row";
import { t } from "../../i18n/i18n";
import { Badge } from "../../primitives/badge/badge";
import { Button } from "../../primitives/button/button";

import { PermissionIcon } from "./permission-icon";
import { PermissionKind, PermissionStatus } from "./types";

type PermissionRowProps = {
  icon: ReactNode;
  onGrant: (permission: PermissionKind, status: PermissionStatus) => void;
  permission: PermissionKind;
  status: PermissionStatus;
  title: string;
  description?: string;
  isOptional?: boolean;
};

// Granted, the row ends in a check; otherwise in the one step that grants
// it: asking, where the system still can, or its settings, where only the
// person can switch it on.
export function PermissionRow({
  description,
  icon,
  isOptional,
  onGrant,
  permission,
  status,
  title,
}: PermissionRowProps) {
  // Named by its own words and then the row, as "Grant Camera": the row's
  // title alone would hide what the button does.
  const buttonId = useId();
  return (
    <ControlRow
      description={description}
      leading={<PermissionIcon>{icon}</PermissionIcon>}
      title={title}
      titleAccessory={
        isOptional ? <Badge>{t("permissions-optional")}</Badge> : undefined
      }
    >
      {(controlProps) =>
        status.granted ? (
          <span
            aria-label={t("permissions-granted")}
            className="flex h-control-height items-center"
            role="img"
          >
            <Check aria-hidden="true" className="size-icon text-success" />
          </span>
        ) : (
          <Button
            aria-describedby={controlProps["aria-describedby"]}
            aria-labelledby={`${buttonId} ${controlProps["aria-labelledby"]}`}
            id={buttonId}
            onPress={() => {
              onGrant(permission, status);
            }}
          >
            {status.canRequest
              ? t("permissions-grant")
              : t("permissions-open-settings")}
          </Button>
        )
      }
    </ControlRow>
  );
}
