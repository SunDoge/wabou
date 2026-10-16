import { FormDraftErrors } from "../primitives/interactions/form-draft.mjs";
//#region src/integrations/standard-schema.d.ts
export interface StandardSchemaIssue {
  readonly message: string;
  readonly path?: ReadonlyArray<PropertyKey | {
    readonly key: unknown;
  }>;
}
export type StandardSchemaResult<Output> = {
  readonly value: Output;
  readonly issues?: undefined;
} | {
  readonly issues: readonly StandardSchemaIssue[];
};
/** Structural subset of Standard Schema V1 used by synchronous form drafts. */
export interface StandardSchema<Input, Output = Input> {
  readonly "~standard": {
    readonly version: 1;
    readonly vendor: string;
    readonly validate: (value: unknown) => StandardSchemaResult<Output> | Promise<StandardSchemaResult<Output>>;
    readonly types?: {
      readonly input: Input;
      readonly output: Output;
    };
  };
}
/**
 * Adapt any synchronous Standard Schema V1 implementation (including Valibot,
 * Zod, and ArkType) to `createFormDraft` without coupling Wabou to its API.
 */
export declare function createStandardSchemaValidator<T extends Record<PropertyKey, unknown>>(schema: StandardSchema<T, unknown>): (value: Readonly<T>) => FormDraftErrors<T>;
//#endregion
//# sourceMappingURL=standard-schema.d.mts.map