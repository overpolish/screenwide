// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RadioButton as AriaRadioButton,
  RadioField as AriaRadioField,
  RadioFieldProps as AriaRadioFieldProps,
} from "react-aria-components";

import { iconButtonVariants } from "../../../components/base/button/icon-button-variants";
import { cn } from "../../../lib/styling";

type IconRadioProps = AriaRadioFieldProps & {
  icon: React.ReactNode;
  buttonClassName?: string;
  iconClassName?: string;
};

export const IconRadio = ({
  buttonClassName,
  icon,
  iconClassName,
  ...props
}: IconRadioProps) => {
  return (
    <AriaRadioField {...props}>
      <AriaRadioButton
        className={({ isDisabled }) =>
          iconButtonVariants({
            className: cn(
              "data-[selected]:bg-neutral data-[selected]:data-[hovered]:bg-neutral-hover data-[selected]:data-[pressed]:bg-neutral-pressed",
              buttonClassName,
            ),
            isDisabled,
            isToggle: true,
          })
        }
      >
        <span
          className={cn(
            "flex shrink-0 items-center justify-center",
            iconClassName,
          )}
        >
          {icon}
        </span>
      </AriaRadioButton>
    </AriaRadioField>
  );
};
