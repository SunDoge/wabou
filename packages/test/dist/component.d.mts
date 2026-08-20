import { JSX } from "solid-js";
//#region src/component.d.ts
interface ComponentLocator {
  readonly tag: string;
  readonly role: string;
  readonly name: string;
  readonly text: string;
  attribute(name: string): string | null;
  click(): void;
  press(key: string): void;
}
interface ComponentScreen {
  getByRole(role: string, options?: {
    name?: string;
    index?: number;
  }): ComponentLocator;
  queryByRole(role: string, options?: {
    name?: string;
    index?: number;
  }): ComponentLocator | null;
  dispose(): void;
}
/** Dispose the active component tree. Vitest users get this automatically. */
declare function cleanupComponents(): void;
/**
 * Mount a component into Wabou's real Solid renderer while recording its
 * authored host tree. This is intentionally a fast component-contract test:
 * native layout, hit testing, and final semantic projection remain the job of
 * `wabou test` behavior scenarios.
 */
declare function renderComponent(render: () => JSX.Element): ComponentScreen;
//#endregion
export { ComponentLocator, ComponentScreen, cleanupComponents, renderComponent };
//# sourceMappingURL=component.d.mts.map