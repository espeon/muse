import { create } from "zustand";

export type ConfigEndpoints = {
  makiBaseURL: string;
  umiBaseURL: string;
  setEndpoints: (newEndpoints: ConfigEndpoints) => void;
};

export const useConfig = create<ConfigEndpoints>((set, get) => {
  const endpoints = get();

  return {
    ...endpoints,
    setEndpoints: (newEndpoints: ConfigEndpoints) => set(newEndpoints),
  };
});
