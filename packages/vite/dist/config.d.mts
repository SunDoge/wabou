//#region src/config.d.ts
/** Frontend-only fallback-host configuration.
 *
 * Normal Wabou applications register Rust capabilities and widgets directly
 * in their statically linked executable.
 */
export interface WabouProjectConfig {
  window?: {
    title?: string;
    width?: number;
    height?: number;
    decorations?: boolean;
  };
  ui?: {
    entry?: string;
  };
}
/** Type-check ordinary data or an asynchronous environment-dependent factory. */
export declare function defineConfig<T extends WabouProjectConfig | (() => WabouProjectConfig | Promise<WabouProjectConfig>)>(config: T): T;
//#endregion
//# sourceMappingURL=config.d.mts.map