import { EventCallback } from '@tauri-apps/api/event';
export interface Program {
    name: string;
    path: string;
    icon: Array<number>;
}
interface WindowStatus {
    path: string;
    active: boolean;
    time: number;
}
export declare function getProgramList(): Promise<Program[]>;
export declare function suspend(): Promise<unknown>;
export declare function resume(): Promise<unknown>;
export declare function isActive(path: string): Promise<boolean>;
export declare function onStatusChanged(fn: EventCallback<WindowStatus>): Promise<import("@tauri-apps/api/event").UnlistenFn>;
export {};
