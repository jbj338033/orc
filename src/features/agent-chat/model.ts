import type { Message } from "@/entities/message/model";
import { createMessage } from "@/entities/message/model";
import { create } from "zustand";

interface ChatState {
  messages: Record<string, Message[]>;
  sendMessage: (surfaceId: string, content: string) => void;
}

export const useChatStore = create<ChatState>((set) => ({
  messages: {},
  sendMessage: (surfaceId, content) => {
    const userMsg = createMessage("user", content);
    set((state) => ({
      messages: {
        ...state.messages,
        [surfaceId]: [...(state.messages[surfaceId] ?? []), userMsg],
      },
    }));

    // placeholder echo
    setTimeout(() => {
      const agentMsg = createMessage("agent", `echo: ${content}`);
      set((state) => ({
        messages: {
          ...state.messages,
          [surfaceId]: [...(state.messages[surfaceId] ?? []), agentMsg],
        },
      }));
    }, 500);
  },
}));
