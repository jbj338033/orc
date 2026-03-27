import { useRef, useEffect, useState } from "react";
import { useChatStore } from "./model";

const EMPTY: never[] = [];

interface AgentSurfaceProps {
  surfaceId: string;
}

export function AgentSurface({ surfaceId }: AgentSurfaceProps) {
  const messages = useChatStore((s) => s.messages[surfaceId]) ?? EMPTY;
  const sendMessage = useChatStore((s) => s.sendMessage);
  const [input, setInput] = useState("");
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages.length]);

  const handleSend = () => {
    const trimmed = input.trim();
    if (!trimmed) return;
    sendMessage(surfaceId, trimmed);
    setInput("");
  };

  return (
    <div className="flex h-full flex-col">
      {/* messages */}
      <div className="flex-1 overflow-y-auto px-4 py-4">
        {messages.length === 0 && (
          <div className="flex h-full items-center justify-center">
            <p className="text-[13px] text-neutral-600">Start a conversation</p>
          </div>
        )}
        <div className="mx-auto flex max-w-2xl flex-col gap-3">
          {messages.map((msg) => (
            <div
              key={msg.id}
              className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}
            >
              <div
                className={`max-w-[80%] rounded-2xl px-4 py-2.5 text-[13px] leading-relaxed ${
                  msg.role === "user"
                    ? "bg-blue-600 text-white"
                    : "bg-white/[0.06] text-neutral-200"
                }`}
              >
                {msg.content}
              </div>
            </div>
          ))}
          <div ref={bottomRef} />
        </div>
      </div>

      {/* input */}
      <div className="px-4 pb-4">
        <div className="mx-auto flex max-w-2xl items-end gap-2.5 rounded-xl bg-white/[0.07] p-1.5 ring-1 ring-white/[0.1] focus-within:ring-white/[0.2]">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                handleSend();
              }
            }}
            placeholder="Message..."
            rows={1}
            className="flex-1 resize-none bg-transparent px-2.5 py-2 text-[13px] text-white placeholder-neutral-600 outline-none"
          />
          <button
            onClick={handleSend}
            className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-lg transition-all duration-150 ${
              input.trim()
                ? "bg-blue-600 text-white hover:bg-blue-500"
                : "text-neutral-600"
            }`}
          >
            <svg className="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
              <path strokeLinecap="round" strokeLinejoin="round" d="m5 12 7-7 7 7M12 5v14" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}
