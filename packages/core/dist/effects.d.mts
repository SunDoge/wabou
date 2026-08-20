//#region src/generated/effect-abi.d.ts
declare const EFFECT_ABI_VERSION: 5;
declare const effectOps: Readonly<{
  readonly clipboardRead: {
    readonly capability: 1;
    readonly method: 1;
  };
  readonly clipboardWrite: {
    readonly capability: 1;
    readonly method: 2;
  };
  readonly windowCreate: {
    readonly capability: 2;
    readonly method: 1;
  };
  readonly windowClose: {
    readonly capability: 2;
    readonly method: 2;
  };
  readonly windowSetMaximized: {
    readonly capability: 2;
    readonly method: 3;
  };
  readonly windowSetTitle: {
    readonly capability: 2;
    readonly method: 4;
  };
  readonly windowMinimize: {
    readonly capability: 2;
    readonly method: 5;
  };
  readonly windowStartDragging: {
    readonly capability: 2;
    readonly method: 6;
  };
  readonly contextMenuShow: {
    readonly capability: 3;
    readonly method: 1;
  };
  readonly appDirsResolve: {
    readonly capability: 4;
    readonly method: 1;
  };
  readonly dialogOpen: {
    readonly capability: 5;
    readonly method: 1;
  };
  readonly dialogSave: {
    readonly capability: 5;
    readonly method: 2;
  };
  readonly dialogPickDirectory: {
    readonly capability: 5;
    readonly method: 3;
  };
  readonly dialogMessage: {
    readonly capability: 5;
    readonly method: 4;
  };
  readonly notificationShow: {
    readonly capability: 6;
    readonly method: 1;
  };
  readonly applicationExit: {
    readonly capability: 7;
    readonly method: 1;
  };
  readonly applicationRelaunch: {
    readonly capability: 7;
    readonly method: 2;
  };
}>;
//#endregion
//#region src/glue/effects.d.ts
interface EffectOp {
  readonly capability: number;
  readonly method: number;
}
//#endregion
export { EFFECT_ABI_VERSION, type EffectOp, effectOps };
//# sourceMappingURL=effects.d.mts.map