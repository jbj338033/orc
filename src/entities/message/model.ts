export type MessageRole = "user" | "agent";

export interface Message {
  id: string;
  role: MessageRole;
  content: string;
  timestamp: number;
}

export function createMessage(role: MessageRole, content: string): Message {
  return { id: crypto.randomUUID(), role, content, timestamp: Date.now() };
}
