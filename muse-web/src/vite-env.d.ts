/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_MAKI_URL?: string;
  readonly VITE_UMI_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
