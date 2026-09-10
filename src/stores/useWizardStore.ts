import { create } from "zustand";
import type { CreateServerRequest } from "@/types";

const defaults: CreateServerRequest = {
  name: "",
  path: "",
  provider: "paper",
  minecraftVersion: "",
  build: null,
  loaderVersion: null,
  installerVersion: null,
  javaPath: null,
  memoryMinMb: 2048,
  memoryMaxMb: 4096,
  port: 25565,
  motd: "",
  eulaAccepted: false,
  startAfterInstall: false,
  generateWorld: true,
  optimizedFlags: true,
  difficulty: "normal",
  gamemode: "survival",
  maxPlayers: 20,
  onlineMode: true,
  pvp: true,
  viewDistance: 10,
  simulationDistance: 10,
  allowFlight: false,
};

type WizardState = {
  step: number;
  request: CreateServerRequest;
  setStep: (step: number) => void;
  patch: (partial: Partial<CreateServerRequest>) => void;
  reset: () => void;
};

export const useWizardStore = create<WizardState>((set) => ({
  step: 0,
  request: defaults,
  setStep: (step) => set({ step }),
  patch: (partial) => set((s) => ({ request: { ...s.request, ...partial } })),
  reset: () => set({ step: 0, request: defaults }),
}));
