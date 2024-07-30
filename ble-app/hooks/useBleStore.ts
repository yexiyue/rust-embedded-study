import { Device, State } from "react-native-ble-plx";
import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
type BleState = {
  device: {
    [key: Device["id"]]: Device;
  };
};

type BleAction = {
  addDevice: (device: Device) => void;
  clearDevice: () => void;
};

export const useBleStore = create(
  immer<BleAction & BleState>((set) => ({
    device: {},
    addDevice: (device: Device) =>
      set((state) => {
        state.device[device.id] = device;
      }),
    clearDevice: () =>
      set((state) => {
        state.device = {};
      }),
  }))
);
