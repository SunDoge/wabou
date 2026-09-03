import * as v from "valibot";
import { evaluateCell, type CellValues } from "./formula";

export const temperatureSchema = v.pipe(
  v.string(),
  v.trim(),
  v.nonEmpty("Enter a temperature."),
  v.toNumber("Enter a valid number."),
  v.finite("Enter a finite number."),
);

export interface PersonDraft {
  first: string;
  last: string;
}

const personName = (label: string) =>
  v.pipe(
    v.string(),
    v.trim(),
    v.minLength(1, `${label} is required.`),
    v.maxLength(80, `${label} must be 80 characters or fewer.`),
  );

export const personSchema = v.object({
  first: personName("First name"),
  last: personName("Surname"),
});

export function cellInputSchema(cells: CellValues, address: string) {
  return v.pipe(
    v.string(),
    v.maxLength(512, "Cell content must be 512 characters or fewer."),
    v.check((value) => {
      if (!value.startsWith("=")) return true;
      const result = evaluateCell({ ...cells, [address]: value }, address);
      return result !== "#ERR!" && result !== "#CYCLE!";
    }, "Enter a valid, non-circular formula."),
  );
}

export function firstValidationError(
  result: v.SafeParseResult<v.GenericSchema>,
): string | undefined {
  return result.success ? undefined : result.issues[0]?.message;
}
