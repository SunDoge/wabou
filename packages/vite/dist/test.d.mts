import { UserConfig } from "vite";
//#region src/test.d.ts
export interface WabouTestConfigOptions {
  /** Additional Vitest/Vite configuration merged over Wabou's defaults. */
  vite?: UserConfig;
}
type ComponentTestUserConfig = UserConfig & {
  test?: {
    environment?: string;
    include?: string[];
    setupFiles?: string[];
    server?: {
      deps?: {
        inline?: (string | RegExp)[];
      };
    };
  };
};
/** Configure Vitest to compile Wabou TSX through Solid's universal renderer. */
export declare function defineWabouTestConfig(options?: WabouTestConfigOptions): ComponentTestUserConfig;
//#endregion
export {};
//# sourceMappingURL=test.d.mts.map