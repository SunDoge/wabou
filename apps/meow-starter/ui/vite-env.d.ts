/// <reference types="vite/client" />

declare module "*.svg?raw" {
  const source: string;
  export default source;
}
