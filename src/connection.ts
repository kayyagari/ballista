import { invoke } from "@tauri-apps/api/tauri";
import {DataNode} from "antd/es/tree";

export const DEFAULT_GROUP_NAME: string = "Default";
export interface Connection {
    address: string,
    heapSize: string,
    icon: string,
    id: string,
    javaHome: string,
    name: string,
    username: string,
    password: string,
    verify: boolean,
    group: string,
    notes: string,

    // the below properties are transient and are used only in the UI
    nodeId: string,
    parentId: string,
}

export async function loadConnections() {
    console.log("loading connections");
    const jsonArr: string = await invoke("load_connections");
    let data = JSON.parse(jsonArr);
    //data.sort(connectionSorter);
    return data;
}

export function orderConnections(data: Connection[]) {
    let groupConnMap: any = {};
    let prevGroup = null;
    for(let i =0; i < data.length; i++) {
        let con = data[i];
        let conArr = groupConnMap[con.group];
        if(conArr === undefined || conArr === null) {
            conArr = new Array();
            groupConnMap[con.group] = conArr;
        }
        conArr.push(con);
        if(prevGroup != null && prevGroup !== con.group) {
            groupConnMap[prevGroup].sort(connectionSorter);
        }
        prevGroup = con.group;
    }
    if(prevGroup != null) {
        groupConnMap[prevGroup].sort(connectionSorter);
    }

    let groupNames = Object.keys(groupConnMap).filter((val) => val != DEFAULT_GROUP_NAME).sort();
    if(groupConnMap[DEFAULT_GROUP_NAME]) {
        groupNames.unshift(DEFAULT_GROUP_NAME);
    }

    return {groupNames, groupConnMap};
}
export function connectionSorter(c1: Connection, c2: Connection) {
    let n1 = c1.name.toLowerCase();
    let n2 = c2.name.toLowerCase();
    if(n1 > n2) {
        return 1;
    }
    else if(n1 < n2) {
        return -1;
    }

    return 0;
}

export function searchText(token: string, c: Connection) {
    token = token.toLowerCase();
    for(const [key, val] of Object.entries(c)) {
        if(key == 'id') {
            continue;
        }
        if((typeof val == 'string') && val.toLowerCase().indexOf(token) > -1) {
            return true;
        }
    }
    return false;
}