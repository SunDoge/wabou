import { UserConfig } from "vite";
//#region src/test.d.ts
interface WabouTestConfigOptions {
  /** Additional Vitest/Vite configuration merged over Wabou's defaults. */
  vite?: UserConfig;
}
type ComponentTestUserConfig = UserConfig & {
  test?: {
    environment?: string;
    setupFiles?: string[];
  };
};
/** Configure Vitest to compile Wabou TSX through Solid's universal renderer. */
declare function defineWabouTestConfig(options?: WabouTestConfigOptions): ComponentTestUserConfig;
//#endregion
export { WabouTestConfigOptions, defineWabouTestConfig };
//# sourceMappingURL=test.d.mts.map