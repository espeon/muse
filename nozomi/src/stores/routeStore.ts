import { stat } from "fs";
import { StoreApi, UseBoundStore, create } from "zustand";

type RouteStore = {
  history: string[];
  future: string[];
  location: string | null;
  push: (location: string) => void;
  getNextLast: () => string | null;
  pop: () => void;
};

export const useRouteStore: UseBoundStore<StoreApi<RouteStore>> = create<RouteStore>((set, get) => ({
  history: [],
  future: [],
  location: null,
  push: (location) => {
    // if location is same as future
    if (get().future[0] === location) {
        console.log("Future same as location")
        // move first of future to location
        set((state) => ({
          future: state.future.slice(1),
        }));
      return;
    }
    set((state) => ({
      history: [...state.history, location],
      future: [],
    }));
  },
  getNextLast: () => {
    if (useRouteStore.getState().future.length > 0) {
      return useRouteStore.getState().future[0];
    }
    return useRouteStore.getState().history[useRouteStore.getState().history.length - 2];
  },
  pop: () => {
    set((state) => ({
      history: state.history.slice(0, -1),
      future: [state.history[state.history.length - 1], ...state.future],
    }));
  },
}))