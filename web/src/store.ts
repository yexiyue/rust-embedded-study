import { create } from "zustand";
import { persist } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";

type Store = {
  staIp: string;
  setStaIp: (staIp: string) => void;
};

export const useStore = create<Store>()(
  persist(
    immer<Store>((set) => ({
      staIp: "",
      setStaIp(staIp) {
        set((state) => {
          state.staIp = staIp;
        });
      },
    })) as any,
    {
      name: "rust-embedded-study",
    }
  )
);
