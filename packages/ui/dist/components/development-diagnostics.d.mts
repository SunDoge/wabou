//#region src/components/development-diagnostics.d.ts
export interface DevServerDiagnostic {
  message: string;
  stack?: string;
  id?: string;
  frame?: string;
  plugin?: string;
  loc?: {
    file?: string;
    line: number;
    column: number;
  };
}
//#endregion
//# sourceMappingURL=development-diagnostics.d.mts.map