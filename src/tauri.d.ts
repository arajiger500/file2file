declare module "@tauri-apps/api/dialog";
declare module "@tauri-apps/api/shell";
declare module "@tauri-apps/api/fs";

declare module "@tauri-apps/api/core" {
    export function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
}
