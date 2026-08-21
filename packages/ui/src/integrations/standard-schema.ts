import {
  FORM_ERROR,
  type FormDraftErrors,
} from "../primitives/interactions/form-draft";

export interface StandardSchemaIssue {
  readonly message: string;
  readonly path?: ReadonlyArray<
    | PropertyKey
    | {
        readonly key: unknown;
      }
  >;
}

export type StandardSchemaResult<Output> =
  | { readonly value: Output; readonly issues?: undefined }
  | { readonly issues: readonly StandardSchemaIssue[] };

/** Structural subset of Standard Schema V1 used by synchronous form drafts. */
export interface StandardSchema<Input, Output = Input> {
  readonly "~standard": {
    readonly version: 1;
    readonly vendor: string;
    readonly validate: (
      value: unknown,
    ) => StandardSchemaResult<Output> | Promise<StandardSchemaResult<Output>>;
    readonly types?: {
      readonly input: Input;
      readonly output: Output;
    };
  };
}

function isPromiseLike(
  value: StandardSchemaResult<unknown> | Promise<StandardSchemaResult<unknown>>,
): value is Promise<StandardSchemaResult<unknown>> {
  return typeof (value as { then?: unknown }).then === "function";
}

function issueKey<T extends Record<PropertyKey, unknown>>(
  value: Readonly<T>,
  issue: StandardSchemaIssue,
): keyof T | typeof FORM_ERROR {
  const segment = issue.path?.[0];
  const candidate =
    typeof segment === "object" && segment !== null && "key" in segment
      ? segment.key
      : segment;
  return (typeof candidate === "string" ||
    typeof candidate === "number" ||
    typeof candidate === "symbol") &&
    Reflect.has(value, candidate)
    ? (candidate as keyof T)
    : FORM_ERROR;
}

/**
 * Adapt any synchronous Standard Schema V1 implementation (including Valibot,
 * Zod, and ArkType) to `createFormDraft` without coupling Wabou to its API.
 */
export function createStandardSchemaValidator<
  T extends Record<PropertyKey, unknown>,
>(
  schema: StandardSchema<T, unknown>,
): (value: Readonly<T>) => FormDraftErrors<T> {
  return (value) => {
    const result = schema["~standard"].validate(value);
    if (isPromiseLike(result)) {
      throw new TypeError(
        "createFormDraft requires synchronous validation; validate asynchronous schemas before submission",
      );
    }
    if (!result.issues) return {};
    const errors: FormDraftErrors<T> = {};
    for (const issue of result.issues) {
      const key = issueKey(value, issue);
      errors[key] ??= issue.message;
    }
    return errors;
  };
}
