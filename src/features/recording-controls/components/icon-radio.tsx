// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RadioButton as AriaRadioButton,
  RadioField as AriaRadioField,
  RadioFieldProps as AriaRadioFieldProps,
} from "react-aria-components";
import { VariantProps } from "tailwind-variants";

import { elementFocusVisible, focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

const radioVariants = tv({
  slots: {
    base: [
      "group p-control relative grid h-full grow grid-rows-[minmax(0,1fr)_auto] justify-items-center rounded-xl text-muted transition-colors select-none",
      "data-[hovered]:bg-neutral",
      "data-[pressed]:bg-neutral-hover",
      "data-[selected]:bg-neutral data-[selected]:text-content-fg",
      "data-[selected]:data-[hovered]:bg-neutral-hover",
      "data-[selected]:data-[pressed]:bg-neutral-pressed",
      focusStyles,
      elementFocusVisible,
    ],
    icon: "flex h-full min-h-0 items-center justify-center [&_svg]:size-icon-prominent",
    subtext: "text-xs font-semibold",
  },
});

type IconRadioProps = AriaRadioFieldProps &
  VariantProps<typeof radioVariants> & {
    icon: React.ReactNode;
    subtext: string;
  };

export const IconRadio = ({ icon, subtext, ...props }: IconRadioProps) => {
  const { base, icon: _icon, subtext: _subtext } = radioVariants();

  return (
    <AriaRadioField {...props} className="flex grow self-stretch">
      <AriaRadioButton className={base()}>
        <div className={_icon()}>{icon}</div>
        <div className={_subtext()}>{subtext}</div>
      </AriaRadioButton>
    </AriaRadioField>
  );
};
