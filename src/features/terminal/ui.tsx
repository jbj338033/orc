import { useEffect, useRef } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";

declare global {
  interface Window {
    __TAURI__?: {
      core: {
        invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
      };
      event: {
        listen: (event: string, handler: (e: { payload: unknown }) => void) => Promise<() => void>;
      };
    };
  }
}

function getTauri() {
  return window.__TAURI__;
}

interface TerminalSurfaceProps {
  surfaceId: string;
}

export function TerminalSurface({ surfaceId }: TerminalSurfaceProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const spawnedRef = useRef(false);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const term = new Terminal({
      fontSize: 13,
      fontFamily: "'SF Mono', 'Menlo', 'Monaco', monospace",
      theme: {
        background: "#0c0c0c",
        foreground: "#e5e5e5",
        cursor: "#e5e5e5",
        selectionBackground: "#ffffff30",
      },
      cursorBlink: true,
      allowProposedApi: true,
    });

    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(el);
    fit.fit();

    termRef.current = term;
    fitRef.current = fit;

    const tauri = getTauri();
    if (!tauri) {
      return () => { term.dispose(); };
    }

    let unlistenData: (() => void) | undefined;
    let unlistenExit: (() => void) | undefined;

    const setup = async () => {
      if (spawnedRef.current) return;
      spawnedRef.current = true;

      unlistenData = await tauri.event.listen(
        `pty-data-${surfaceId}`,
        (e: { payload: unknown }) => {
          term.write(e.payload as string);
        },
      );

      unlistenExit = await tauri.event.listen(`pty-exit-${surfaceId}`, () => {
        term.write("\r\n[process exited]\r\n");
      });

      term.onData((data: string) => {
        tauri.core.invoke("pty_write", { id: surfaceId, data });
      });

      await tauri.core.invoke("pty_spawn", {
        id: surfaceId,
        rows: term.rows,
        cols: term.cols,
      });
    };

    setup();

    const resizeObserver = new ResizeObserver(() => {
      fit.fit();
      if (spawnedRef.current && tauri) {
        tauri.core.invoke("pty_resize", {
          id: surfaceId,
          rows: term.rows,
          cols: term.cols,
        });
      }
    });
    resizeObserver.observe(el);

    return () => {
      resizeObserver.disconnect();
      unlistenData?.();
      unlistenExit?.();
      term.dispose();
      if (tauri && spawnedRef.current) {
        tauri.core.invoke("pty_kill", { id: surfaceId });
      }
    };
  }, [surfaceId]);

  return (
    <div
      ref={containerRef}
      className="h-full w-full bg-[#0c0c0c] p-1"
    />
  );
}
