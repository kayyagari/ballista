import {invoke} from "@tauri-apps/api/core";

export interface BallistaInfo {
    ballista_version: string
}

export async function requestBallistaInfo() {
    console.log("requesting ballista info");
    const jsonArr: string = await invoke("get_ballista_info");
    return JSON.parse(jsonArr)
}