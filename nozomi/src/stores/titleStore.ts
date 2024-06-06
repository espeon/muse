import {create} from "zustand";

interface TitleStore {
    pageTitle: string;
    setPageTitle: (title: string) => void;
    pageTitleVisible: boolean;
    setPageTitleVisible: (visible: boolean) => void;
}

export const useTitleStore = create<TitleStore>((set) => ({
    pageTitle: "",
    setPageTitle: (title: string) => set({ pageTitle: title }),
    pageTitleVisible: true,
    setPageTitleVisible: (visible: boolean)=> set({ pageTitleVisible: visible }),
}));

