/// <reference types="vite/client" />

declare module "virtual:wabou-stylesheet";

declare module "*.svg?raw" {
  const source: string;
  export default source;
}
