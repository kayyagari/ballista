import React, {useEffect, useState} from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { open, confirm } from '@tauri-apps/api/dialog';
import "./App.css";
import './CertDialog'
import {
    Avatar,
    Button,
    Col,
    Divider,
    Input,
    Layout,
    List,
    Menu,
    MenuProps,
    Row,
    theme,
    Modal,
    Checkbox, Spin, notification, Tree
} from "antd";
import type { DataNode } from 'antd/es/tree';
import {ApiOutlined, CarryOutOutlined, EyeInvisibleOutlined, EyeTwoTone, SettingOutlined} from "@ant-design/icons";
import CertDialog, {UntrustedCert} from "./CertDialog";
import {NotificationPlacement} from "antd/es/notification/interface";
import {
    Connection,
    connectionSorter,
    DEFAULT_GROUP_NAME,
    loadConnections,
    orderConnections,
    searchConnection, searchText
} from './connection';
import Search from "antd/es/input/Search";

const { Content, Sider } = Layout;

type MenuItem = Required<MenuProps>['items'][number];

function getItem(
    label: React.ReactNode,
    key: React.Key,
    icon?: React.ReactNode,
    children?: MenuItem[],
): MenuItem {
    return {
        key,
        icon,
        children,
        label,
    } as MenuItem;
}

const items: MenuItem[] = [
    getItem('', 'settings', <SettingOutlined />, [
        getItem('Import', 'import')])
];

const Context = React.createContext({ name: 'Default' });

function App() {
    const {
        token: { colorBgContainer },
    } = theme.useToken();
    const [api, contextHolder] = notification.useNotification();
    const openNotification = (placement: NotificationPlacement, msg: string) => {
        api.info({
            message: `Error`,
            description: <Context.Consumer>{({ name }) => `${msg}`}</Context.Consumer>,
            placement
        });
    };

    const [data, setData] = useState<Connection[]>([]);
    const [treeData, setTreeData] = useState<DataNode[]>([]);
    const [expandedKeys, setExpandedKeys] = useState<React.Key[]>([]);

    const emptyConnection: Connection = {
        address: "",
        heapSize: "",
        icon: "",
        id: "",
        javaHome: "",
        name: "",
        username: "",
        password: "",
        verify: true,
        group: "Default",
        notes: ""};

    const [cc, setCc] = useState<Connection>({...emptyConnection});

    const [dirty, setDirty] = useState<boolean>(false);

    const [cert, setCert] = useState<UntrustedCert>({
        der: undefined,
        subject: undefined,
        issuer: undefined,
        expires_on: undefined
    });
    const [loading, setLoading] = useState(false);

    useEffect(() => {loadConnections().then(d => {
        setData(d);
        createTreeNodes(d);
        if(d.length > 0) {
            setCc(d[0])
        }
    })}, [])
    async function importConnections(e: any) {
        console.log(e);
        const selected = await open({
            multiple: false,
            filters: [{
                name: '',
                extensions: ['json']
            }]
        });
        if (selected !== null) {
            console.log(selected)
            await invoke("import", {file_path: selected});
            console.log("after invoking import")
            loadConnections().then(d => {
                setData(d);
                createTreeNodes(d);
                for(let i =0; i < d.length; i++) {
                    if(cc.id == d[i].id) {
                        setCc(d[i]);
                    }
                }
            })
        }
    }

    async function launch() {
        setLoading(true);
        try {
            let resp: string = await invoke("launch", { id: cc.id });
            let result: any = JSON.parse(resp);
            if(result.code == 1) {
                setCert(result.cert);
            }
            setLoading(false);
            if(result.code == -1) {
                openNotification('topRight', result.msg);
            }
        }
        catch (e) {
            setLoading(false);
        }
    }

    async function trustAndLaunch() {
        await invoke("trust_cert", { cert: cert.der });
        setCert({});
        launch();
    }
    function abortLaunch() {
        setCert({});
    }
    function createNew() {
        setCc({...emptyConnection})
        setDirty(false);
    }

    function createTreeNodes(d: Connection[]) {
        if(d.length == 0) {
            return;
        }
        let orderedConMap = orderConnections(d);
        let nodes: DataNode[] = [];
        for(let i = 0; i < orderedConMap.groupNames.length; i++) {
            let name = orderedConMap.groupNames[i];
            let groupedCons = orderedConMap.groupConnMap[name];
            let conNodes: DataNode[] = [];
            let parentKey = i.toString();
            for(let j = 0; j < groupedCons.length; j++) {
                let c = groupedCons[j];
                let node = {
                  title: c.name,
                  key: parentKey + "-" + j.toString(),
                  con: c,
                  icon: <CarryOutOutlined />
                };
                conNodes.push(node);
            }

            let groupNode = {
                title: name,
                key: parentKey,
                icon: <CarryOutOutlined />,
                children: conNodes
            };
            nodes.push(groupNode);
        }
        setTreeData(nodes);
        //setExpandedKeys(["0-0"]);
    }

    const handleMenuClick = ({ key, domEvent }: any) => {
        console.log(key);
        if (key == 'import') {
            importConnections(domEvent);
        }
    };

    function updateName(e: any) {
        setCc({
            ...cc,
            name: e.target.value
        })
        setDirty(true);
    }

    function updateUrl(e: any) {
        setCc({
            ...cc,
            address: e.target.value
        })
        setDirty(true);
    }
    function updateUsername(e: any) {
        setCc({
            ...cc,
            username : e.target.value
        })
        setDirty(true);
    }

    function updatePassword(e: any) {
        setCc({
            ...cc,
            password: e.target.value
        })
        setDirty(true);
    }
    function updateJavaHome(e: any) {
        setCc({
            ...cc,
            javaHome: e.target.value
        })
        setDirty(true);
    }

    function updateHeapSize(e: any) {
        setCc({
            ...cc,
            heapSize: e.target.value
        })
        setDirty(true);
    }

    function updateVerify(e: any) {
        setCc({
            ...cc,
            verify: e.target.checked
        })
        setDirty(true);
    }

    function updateNotes(e: any) {
        setCc({
            ...cc,
            notes: e.target.value
        })
        setDirty(true);
    }

    function updateGroup(e: any) {
        setCc({
            ...cc,
            group: e.target.value
        })
        setDirty(true);
    }
    async function deleteConnection() {
        const confirmed = await confirm('Do you want to delete connection ' + cc.name + '?', { title: '', type: 'warning' });
        if(confirmed) {
            const resp = await invoke("delete", {id: cc.id});
            if(resp == "success") {
                let i = -1;
                let pos = -1;
                let tmp = data.filter(c => {
                    i++;
                    if(c.id == cc.id) {
                        pos = i;
                    }
                    return c.id !== cc.id;
                });
                setData(tmp);
                if(pos != -1) {
                    if(pos == tmp.length) {
                        pos--;
                    }
                    setCc(tmp[pos])
                }
            }
        }
    }

    async function saveConnection() {
        if(cc.group.trim().length == 0) {
            cc.group = DEFAULT_GROUP_NAME;
        }

        let saveResult: string = await invoke("save", {ce: JSON.stringify(cc)});
        try {
            let savedCon = JSON.parse(saveResult);
            setCc({...savedCon});
            setDirty(false);
            let tmp = data.filter(c => c.id !== savedCon.id);;
            tmp.push(savedCon);
            setData(tmp);
            createTreeNodes(tmp);
        }
        catch(e) {
            //TODO handle it
        }
    }

    function selectConnection(index: number) {
        setCc(data[index]);
        console.log("selected connection " + index);
    }

    const getParentKey = (key: React.Key, tree: DataNode[]): React.Key => {
        let parentKey: React.Key;
        for (let i = 0; i < tree.length; i++) {
            const node = tree[i];
            if (node.children) {
                if (node.children.some((item) => item.key === key)) {
                    parentKey = node.key;
                } else {
                    let tmp = getParentKey(key, node.children);
                    if(tmp) {
                        parentKey = tmp;
                    }
                }
            }
        }
        return parentKey!;
    };

    const onTreeNodeSelect = (selectedKeys: React.Key[], info: any) => {
        if(info.node.con) {
            setCc(info.node.con);
        }
    };

    const searchConnections = (e: React.ChangeEvent<HTMLInputElement>) => {
        let {value} = e.target;
        value = value.trim();
        if(value.length == 0) {
            createTreeNodes(data);
            return;
        }
        if(value.length < 2) {
            return;
        }
        let filteredCons = data.filter((c) => searchText(value, c));
        createTreeNodes(filteredCons);
        setExpandedKeys(["0-0"])
    };

    return (
        <Context.Provider value={{name: ""}}>
        {contextHolder}
        <Spin spinning={loading}>
        <Layout className='layout' style={{ height: '97vh' }}>
            <Sider width={'30%'} style={{ height: '470' }} theme={"light"} >
                <div style={{overflow: 'auto', height: '90%'}}>
                    <Search style={{ marginBottom: 8 }} placeholder="Search" onChange={searchConnections} />
                    <Tree
                        showLine={true}
                        defaultExpandedKeys={expandedKeys}
                        onSelect={onTreeNodeSelect}
                        treeData={treeData}
                    />
                </div>
                <Row align={'middle'} style={{height: '5'}} gutter={[24, 3]}>
                    <Col>
                        <Menu theme="light" mode="horizontal" triggerSubMenuAction="click" items={items} onClick={handleMenuClick} />
                    </Col>
                    <Col span={10} />
                    <Col><Button onClick={createNew} >+</Button></Col>
                </Row>
            </Sider>
            <Layout className="site-layout">
                {/* <Header style={{ padding: 0, background: 'rgba(255, 255, 255, 0.2)' }} /> */}
                <Content style={{ marginLeft: '4px' }} >
                    <div style={{ padding: 1, textAlign: 'center', background: colorBgContainer, width: '100%' }}>
                        <Row align={'middle'} gutter={[24, 3]} style={{ marginBottom: 50, marginTop: 10 }}>
                            <Col span={4}>Name:</Col>
                            <Col span={16}>
                                <Input placeholder={"Connection's name e.g Acme Test Instance"} size={"middle"} bordered value={cc.name} onChange={updateName} autoFocus={true}/>
                            </Col>
                        </Row>
                        <Row align={"middle"} gutter={[24, 3]} style={{ marginBottom: 8 }}>
                            <Col span={4} style={{ textAlign: "center" }}>URL:</Col>
                            <Col span={16}>
                                <Input placeholder="MC URL e.g https://localhost:8443" size={"middle"} bordered value={cc.address} onChange={updateUrl} onPressEnter={launch} />
                            </Col>
                            <Col><Button type={"primary"} onClick={launch} disabled={cc.id == "" || cc.address == "" || dirty} >Open</Button></Col>
                        </Row>
                        <Row align={'middle'} gutter={[24, 3]} style={{ marginBottom: 8 }}>
                            <Col span={4}>Username:</Col>
                            <Col span={12}>
                                <Input placeholder={"Username e.g admin"} size={"middle"} bordered value={cc.username} onChange={updateUsername} />
                            </Col>
                        </Row>
                        <Row align={'middle'} gutter={[24, 3]} style={{ marginBottom: 8 }}>
                            <Col span={4}>Password:</Col>
                            <Col span={12}>
                                <Input.Password placeholder={"Password. Skip, if sensitive"} size={"middle"} value={cc.password}
                                    iconRender={(visible) => (visible ? <EyeTwoTone /> : <EyeInvisibleOutlined />)}
                                    onChange={updatePassword} />
                            </Col>
                        </Row>
                        <Row align={'middle'} gutter={[24, 3]}>
                            <Col span={4}>Java Home:</Col>
                            <Col span={12}>
                                <Input placeholder={"Path to Java Home Directory"} size={"middle"} bordered value={cc.javaHome} onChange={updateJavaHome} />
                            </Col>
                        </Row>
                        <Row align={'middle'} gutter={[24, 3]}>
                            <Col span={4}>Max Memory:</Col>
                            <Col span={12}>
                                <Input placeholder={"e.g. 512m or 2g "} size={"middle"} bordered value={cc.heapSize} onChange={updateHeapSize} />
                            </Col>
                            <Col>
                                <Checkbox checked={cc.verify} onChange={updateVerify}>Verify JAR files</Checkbox>
                            </Col>
                        </Row>
                        <Row align={'middle'} gutter={[24, 3]}>
                            <Col span={4}>Group:</Col>
                            <Col span={12}>
                                <Input placeholder={"Name of the connection's group"} size={"middle"} bordered value={cc.group} onChange={updateGroup} />
                            </Col>
                        </Row>
                        <Row>
                            <Col span={20} style={{ marginTop: 20, alignContent: "end" }}>
                                <Button type={"primary"} disabled={!dirty} onClick={saveConnection}>Save</Button>
                            </Col>
                        </Row>
                        <Row style={{ marginTop: 150 }}>
                            <Col style={{ alignContent: "end" }}><Button type={"primary"} danger onClick={deleteConnection} disabled={cc.id == ""}>Delete</Button></Col>
                        </Row>
                    </div>
                    <CertDialog trustAndLaunch={trustAndLaunch} abortLaunch={abortLaunch} cert={cert}/>
                </Content>
            </Layout>
        </Layout>
        </Spin>
        </Context.Provider>
    );
}

export default App;
