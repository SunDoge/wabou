import {
  CapabilityError,
  type CapabilityClientOptions,
  type NativeCapability,
} from "./native-capability";

export type JsonCapabilityMethodName<Capability> = Extract<
  {
    [Key in keyof Capability]: Capability[Key] extends (
      ...args: never[]
    ) => unknown
      ? Key
      : never;
  }[keyof Capability],
  string
>;

export type JsonCapabilityClient = <Response>(
  method: string,
  request?: unknown,
) => Promise<Response>;

/** Bind Wabou's versioned JSON capability transport to a typed app wrapper. */
export function bindJsonCapability<Capability extends NativeCapability>(
  capability: Capability | undefined,
  options: CapabilityClientOptions,
): JsonCapabilityClient {
  return async <Response>(method: string, request?: unknown) => {
    if (capability?.__wabouCapabilityVersion !== options.version)
      throw new CapabilityError(
        `The native ${options.name} capability version ${options.version} is unavailable`,
        "capability_unavailable",
      );
    const functionValue = (capability as object as Record<string, unknown>)[
      method
    ];
    if (typeof functionValue !== "function")
      throw new CapabilityError(
        `The native ${options.name}.${method} method is unavailable`,
        "method_unavailable",
      );
    const raw = await (request === undefined
      ? (functionValue as () => string | PromiseLike<string>).call(capability)
      : (functionValue as (value: string) => string | PromiseLike<string>).call(
          capability,
          JSON.stringify(request),
        ));
    if (typeof raw !== "string")
      throw new CapabilityError(
        `The native ${options.name}.${method} method returned a non-string response`,
        "invalid_response",
      );
    let envelope: unknown;
    try {
      envelope = JSON.parse(raw);
    } catch {
      throw new CapabilityError(
        `The native ${options.name}.${method} method returned invalid JSON`,
        "invalid_response",
      );
    }
    if (
      typeof envelope !== "object" ||
      envelope === null ||
      !("ok" in envelope)
    )
      throw new CapabilityError(
        `The native ${options.name}.${method} method returned an invalid response envelope`,
        "invalid_response",
      );
    if ((envelope as { ok?: unknown }).ok === true) {
      if (!("value" in envelope))
        throw new CapabilityError(
          `The native ${options.name}.${method} method returned a success envelope without a value`,
          "invalid_response",
        );
      return (envelope as { value: Response }).value;
    }
    const error = (
      envelope as {
        error?: { code?: unknown; message?: unknown };
      }
    ).error;
    const code = typeof error?.code === "string" ? error.code : undefined;
    const message =
      typeof error?.message === "string"
        ? error.message
        : `${options.name}.${method} failed`;
    throw new CapabilityError(message, code ?? "handlerFailure");
  };
}
