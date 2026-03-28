import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Message } from "@/entities/message/model";
import { createMessage } from "@/entities/message/model";
import { create } from "zustand";

interface ChatState {
  messages: Record<string, Message[]>;
  streaming: Record<string, boolean>;
  listeners: Record<string, UnlistenFn[]>;
  sendMessage: (surfaceId: string, content: string) => void;
  initSession: (surfaceId: string, profileId: string) => Promise<void>;
  destroySession: (surfaceId: string) => Promise<void>;
}

export const useChatStore = create<ChatState>((set, get) => ({
  messages: {},
  streaming: {},
  listeners: {},

  initSession: async (surfaceId, profileId) => {
    try {
      await invoke("agent_spawn", { id: surfaceId, profileId });
    } catch {
      // session may already exist
    }

    const unlisteners: UnlistenFn[] = [];

    // streaming text
    unlisteners.push(
      await listen<string>(`agent-text-${surfaceId}`, (event) => {
        set((state) => {
          const msgs = state.messages[surfaceId] ?? [];
          const last = msgs[msgs.length - 1];

          if (last && last.role === "agent" && state.streaming[surfaceId]) {
            const updated = [...msgs.slice(0, -1), { ...last, content: last.content + event.payload }];
            return { messages: { ...state.messages, [surfaceId]: updated } };
          }

          const agentMsg = createMessage("agent", event.payload);
          return {
            messages: { ...state.messages, [surfaceId]: [...msgs, agentMsg] },
            streaming: { ...state.streaming, [surfaceId]: true },
          };
        });
      })
    );

    // done
    unlisteners.push(
      await listen(`agent-done-${surfaceId}`, () => {
        set((state) => ({
          streaming: { ...state.streaming, [surfaceId]: false },
        }));
      })
    );

    // error
    unlisteners.push(
      await listen<string>(`agent-error-${surfaceId}`, (event) => {
        const errorMsg = createMessage("agent", `[error] ${event.payload}`);
        set((state) => ({
          messages: {
            ...state.messages,
            [surfaceId]: [...(state.messages[surfaceId] ?? []), errorMsg],
          },
          streaming: { ...state.streaming, [surfaceId]: false },
        }));
      })
    );

    // tool call
    unlisteners.push(
      await listen<{ id: string; name: string }>(`agent-tool-call-${surfaceId}`, (event) => {
        const toolMsg = createMessage("agent", `[tool: ${event.payload.name}]`);
        set((state) => ({
          messages: {
            ...state.messages,
            [surfaceId]: [...(state.messages[surfaceId] ?? []), toolMsg],
          },
        }));
      })
    );

    set((state) => ({
      listeners: { ...state.listeners, [surfaceId]: unlisteners },
    }));
  },

  destroySession: async (surfaceId) => {
    const { listeners } = get();
    const unlisteners = listeners[surfaceId];
    if (unlisteners) {
      for (const unlisten of unlisteners) unlisten();
    }

    try {
      await invoke("agent_kill", { id: surfaceId });
    } catch {
      // ignore
    }

    set((state) => {
      const { [surfaceId]: _msgs, ...restMsgs } = state.messages;
      const { [surfaceId]: _listeners, ...restListeners } = state.listeners;
      const { [surfaceId]: _streaming, ...restStreaming } = state.streaming;
      return { messages: restMsgs, listeners: restListeners, streaming: restStreaming };
    });
  },

  sendMessage: (surfaceId, content) => {
    const userMsg = createMessage("user", content);
    set((state) => ({
      messages: {
        ...state.messages,
        [surfaceId]: [...(state.messages[surfaceId] ?? []), userMsg],
      },
    }));

    invoke("agent_send", { id: surfaceId, content }).catch((err) => {
      const errorMsg = createMessage("agent", `[error] ${err}`);
      set((state) => ({
        messages: {
          ...state.messages,
          [surfaceId]: [...(state.messages[surfaceId] ?? []), errorMsg],
        },
      }));
    });
  },
}));
