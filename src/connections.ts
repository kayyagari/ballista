import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
interface Connection {
  connectionName: string;
  connectionURL: string;
  username: string;
  password: string;
  group: string;
  path: string;
  note: string;
}

interface ConnectionsState {
  groups: string[];
  connections: Connection[];
  addConnection: (connection: Connection) => void;
  removeConnection: (connection: Connection) => void;
  clearConnections: () => void;
  addGroup: (group: string) => void;
  removeGroup: (group: string) => void;
  clearGroups: () => void;
}

export const useConnectionStore = create<ConnectionsState>()(
  persist(
    (set) => ({
      connections: [],
      groups: ["Default"],
      addConnection: (connection) =>
        set((state) => ({
          connections: [...state.connections, connection],
        })),
      removeConnection: (connection) =>
        set((state) => ({
          connections: state.connections.filter((c) => c !== connection),
        })),
      clearConnections: () => set({ connections: [] }),
      addGroup: (group) =>
        set((state) => ({
          groups: [...state.groups, group],
        })),
      removeGroup: (group) =>
        set((state) => ({
          groups: state.groups.filter((g) => g !== group),
        })),
      clearGroups: () => set({ groups: [] }),
    }),
    {
      name: "connections-storage",
      storage: createJSONStorage(() => localStorage),
    }
  )
);
