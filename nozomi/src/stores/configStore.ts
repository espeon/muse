import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

export type ConfigEndpoints = {
  makiExternalBaseURL: string;
  umiBaseURL: string;
  setEndpoints: (newEndpoints: ConfigEndpoints) => void;
};

export const useConfig = create<ConfigEndpoints>()(
  persist(
    (set, get) => {
      const endpoints = get();

      return {
        ...endpoints,
        setEndpoints: (newEndpoints: ConfigEndpoints) => set(newEndpoints),
      };
    },
    {
      name: "store-maki-url-config-abzk2", // name of item in the storage (must be unique)
      storage: createJSONStorage(() => sessionStorage), // (optional) by default the 'localStorage' is used
    },
  ),
);
