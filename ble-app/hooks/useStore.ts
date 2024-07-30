import AsyncStorage from "@react-native-async-storage/async-storage";
import { DeviceId } from "react-native-ble-plx";
import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";

type State = {
  current: DeviceId;
  services: {
    [key: DeviceId]: ServiceConfig;
  };
};

export type ServiceConfig = {
  name?: string | null;
  color?: string;
  setLightService: ServiceIds;
  closeLightService: ServiceIds;
};

export type ServiceIds = {
  serviceUuid: string;
  characteristicUuid: string;
};
type Actions = {
  setColor: (color: string) => void;
  setService: (deviceId: DeviceId, serviceConfig: ServiceConfig) => void;
  removeService: (deviceId: DeviceId) => void;
  setCurrent: (deviceId: DeviceId) => void;
};

export const useStore = create(
  persist(
    immer<State & Actions>((set) => ({
      current: "",
      services: {},
      setService(deviceId, serviceConfig) {
        set((state) => {
          state.services[deviceId] = serviceConfig;
          if (!state.current) {
            state.current = deviceId;
          }
        });
      },
      setColor: (color: string) => {
        set((state) => {
          state.services[state.current].color = color;
        });
      },
      setCurrent: (deviceId: DeviceId) => {
        set((state) => {
          state.current = deviceId;
        });
      },
      removeService(deviceId) {
        set((state) => {
          delete state.services[deviceId];
          state.current = Object.keys(state.services)[0] || "";
        });
      },
    })),
    {
      name: "theme",
      storage: createJSONStorage(() => AsyncStorage),
    }
  )
);
