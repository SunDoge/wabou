import { TextInputProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/input-otp.d.ts
export declare function normalizeOtpValue(value: string, maxLength: number, allowed?: RegExp): string;
export interface InputOTPProps {
  value?: string;
  defaultValue?: string;
  maxLength: number;
  allowed?: RegExp;
  disabled?: boolean;
  readOnly?: boolean;
  "aria-label": string;
  class?: string;
  inputClass?: string;
  children?: JSX.Element;
  onValueChange?: (value: string) => void;
  onComplete?: (value: string) => void;
  ref?: TextInputProps["ref"];
}
export declare function InputOTP(props: InputOTPProps): JSX.Element;
export declare function InputOTPGroup(props: ViewProps): JSX.Element;
export interface InputOTPSlotProps extends Omit<ViewProps, "children"> {
  index: number;
}
export declare function InputOTPSlot(props: InputOTPSlotProps): JSX.Element;
export declare function InputOTPSeparator(props: ViewProps): JSX.Element;
//#endregion
//# sourceMappingURL=input-otp.d.mts.map