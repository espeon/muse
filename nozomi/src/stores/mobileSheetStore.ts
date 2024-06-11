import { create } from 'zustand'

type MobileSheetStore = {
  isOpen: boolean
  toggle: () => void
}

export const useMobileSheetStore = create<MobileSheetStore>((set) => ({
  isOpen: false,
  toggle: () => set((state) => ({ isOpen: !state.isOpen })),
}))

