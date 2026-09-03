export interface NativeCapability {
  readonly __wabouCapabilityVersion: number;
}

export interface CapabilityClientOptions {
  name: string;
  version: number;
}

export class CapabilityError extends Error {
  readonly code: string;

  constructor(message: string, code = "capability_unavailable") {
    super(message);
    this.name = "CapabilityError";
    this.code = code;
  }
}

/** Validate and expose one versioned native capability namespace. */
export function bindCapability<Capability extends NativeCapability>(
  capability: Capability | undefined,
  options: CapabilityClientOptions,
): Capability {
  if (capability?.__wabouCapabilityVersion !== options.version)
    throw new CapabilityError(
      `The native ${options.name} capability version ${options.version} is unavailable`,
    );
  return capability;
}
