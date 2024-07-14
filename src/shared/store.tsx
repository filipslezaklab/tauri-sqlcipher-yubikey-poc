import { createJSONStorage, persist } from "zustand/middleware";
import { createWithEqualityFn } from "zustand/traditional";
import { AuthenticationMethod } from "./types";

const defaults: StoreValues = {
  authenticationMethod: AuthenticationMethod.UNSET,
};

export const useAppStore = createWithEqualityFn<Store>()(
  persist(
    (set) => ({
      ...defaults,
      setState: (vals) => {
        if (vals) {
          set(vals);
        }
      },
    }),
    {
      version: 1,
      name: "app-store",
      storage: createJSONStorage(() => localStorage),
    },
  ),
  Object.is,
);

type Store = StoreValues & StoreMethods;

type StoreValues = {
  authenticationMethod: AuthenticationMethod;
};

type StoreMethods = {
  setState: (values?: Partial<StoreValues>) => void;
};
