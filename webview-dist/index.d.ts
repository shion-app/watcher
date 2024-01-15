import { EventCallback } from '@tauri-apps/api/event';
interface Program {
    name: string;
    path: string;
    icon: Array<number>;
}
export declare function getProgramList(): Promise<Program[]>;
export declare function onWindowActivate(fn: EventCallback<string>): Promise<import("@tauri-apps/api/event").UnlistenFn>;
export {};
