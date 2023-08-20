import React, {useEffect, useRef, useState} from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { open, confirm } from '@tauri-apps/api/dialog';
import { appWindow } from "@tauri-apps/api/window";
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
    Checkbox, Spin, notification, Tree, Select, Space, InputRef, ConfigProvider
} from "antd";
import type { DataNode } from 'antd/es/tree';
import {
    ApiOutlined,
    CarryOutOutlined,
    EyeInvisibleOutlined,
    EyeTwoTone,
    PlusOutlined,
    SettingOutlined
} from "@ant-design/icons";
import CertDialog, {UntrustedCert} from "./CertDialog";
import {NotificationPlacement} from "antd/es/notification/interface";
import {
    Connection,
    connectionSorter,
    DEFAULT_GROUP_NAME,
    loadConnections,
    orderConnections,
    searchText
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

    const { defaultAlgorithm, darkAlgorithm } = theme;
    const [isDarkMode, setIsDarkMode] = useState(false);
    appWindow.theme().then(t => setIsDarkMode(t == 'dark'));
    appWindow.onThemeChanged(({ payload: theme }) => {
        setIsDarkMode(theme == 'dark');
    });

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
    const [autoExpandParent, setAutoExpandParent] = useState(false);
    const [searchVal, setSearchVal] = useState("");
    const [selectedTreeNodeKey, setSelectedTreeNodeKey] = useState<React.Key[]>([]);

    const [newGroupName, setNewGroupName] = useState("");
    const [groupNames, setGroupNames] = useState([DEFAULT_GROUP_NAME]);
    const groupInputRef = useRef<InputRef>(null); // for the group name selection

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
        notes: "",
        nodeId: "",
        parentId: ""
        };

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
            // also gather the group names for once
            let titles = new Array();
            for(let i = 0; i < d.length; i++) {
                let g = d[i].group;
                if(titles.indexOf(g) == -1) {
                    titles.push(g);
                }
            }
            titles.sort();
            setGroupNames(titles);
        }
    })}, [])
    async function importConnections(e: any) {
        const selected = await open({
            multiple: false,
            filters: [{
                name: '',
                extensions: ['json']
            }]
        });
        if (selected !== null) {
            await invoke("import", {file_path: selected});
            loadConnections().then(d => {
                setData(d);
                createTreeNodes(d);
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
        setSelectedTreeNodeKey([]);
        setDirty(false);
    }

    function createTreeNodes(d: Connection[]) {
        let tobeExpanded: string[] = [];
        if(d.length == 0) {
            setTreeData([]);
            return tobeExpanded;
        }
        let orderedConMap = orderConnections(d);
        let nodes: DataNode[] = [];
        for(let i = 0; i < orderedConMap.groupNames.length; i++) {
            let name = orderedConMap.groupNames[i];
            let groupedCons = orderedConMap.groupConnMap[name];
            let conNodes: DataNode[] = [];
            let parentId = i.toString();
            tobeExpanded.push(parentId + "-0");
            for(let j = 0; j < groupedCons.length; j++) {
                let c = groupedCons[j];
                c.parentId = parentId;
                let nodeId = parentId + "-" + j.toString();
                c.nodeId = nodeId;
                let node = {
                  title: c.name,
                  key: nodeId,
                  con: c,
                  icon: <CarryOutOutlined />
                };
                conNodes.push(node);
            }

            let groupNode = {
                title: name,
                key: parentId,
                icon: <CarryOutOutlined />,
                children: conNodes
            };
            nodes.push(groupNode);
        }
        setTreeData(nodes);
        if(nodes.length > 0) {
            let firstParent = nodes[0];
            if(firstParent.children && firstParent.children.length > 0) {
                let child = firstParent.children[0];
                let selectedKeys = [child.key];
                setSelectedTreeNodeKey(selectedKeys);
                setExpandedKeys(selectedKeys);
                setAutoExpandParent(true);
                onTreeNodeSelect(selectedKeys, {node: child});
            }
        }
        return tobeExpanded;
    }

    const handleMenuClick = ({ key, domEvent }: any) => {
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

    function updateGroup(name: string) {
        setCc({
            ...cc,
            group: name
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
                // it is easier to search again rather than updating the tree
                // this is clearly inefficient and needs to be fixed
                searchConnections(searchVal, tmp);
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
            setDirty(false);
            let tmp = data.filter(c => c.id !== savedCon.id);
            tmp.push(savedCon);
            setData(tmp);
            createTreeNodes(tmp);
            let selectedKey = [savedCon.nodeId];
            setSelectedTreeNodeKey(selectedKey);
            setExpandedKeys(selectedKey);
            setAutoExpandParent(true);
            setCc({...savedCon});
        }
        catch(e) {
            //TODO handle it
        }
    }

    const onTreeNodeSelect = (selectedKeys: React.Key[], info: any) => {
        setSelectedTreeNodeKey(selectedKeys);
        if(info.node.con) {
            setCc(info.node.con);
        }
    };

    const setValAndSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
        let {value} = e.target;
        value = value.trim();
        setSearchVal(value);
        searchConnections(value, data);
    }

    const searchConnections = (value: string, connections: Connection[]) => {
        console.log("search value " + searchVal);
        if(value.length == 0) {
            createTreeNodes(connections);
            return;
        }
        if(value.length < 2) {
            return;
        }
        let filteredCons = connections.filter((c) => searchText(value, c));
        let tobeExpanded: string[] = createTreeNodes(filteredCons);
        if(tobeExpanded.length > 0) {
            setExpandedKeys(tobeExpanded as React.Key[]);
            setAutoExpandParent(true);
        }
        else {
            setExpandedKeys([]);
            setAutoExpandParent(false);
        }
    };

    const updateNewGroupName = (event: React.ChangeEvent<HTMLInputElement>) => {
        setNewGroupName(event.target.value);
    };
    const addNewGroup = (e: React.MouseEvent<HTMLButtonElement | HTMLAnchorElement>) => {
        e.preventDefault();
        let tmp = newGroupName.trim().toLowerCase();
        if(tmp.length == 0) {
            return;
        }
        let exists = false;
        for(let i = 0; i < groupNames.length; i++) {
            if(groupNames[i].toLowerCase() == tmp) {
                exists = true;
            }
        }
        if(!exists) {
            let tmp = [...groupNames, newGroupName]
            tmp.sort();
            setGroupNames(tmp);
        }
        setNewGroupName("");
        setTimeout(() => {
            groupInputRef.current?.focus();
        }, 0);
    };

    const onExpand = (newExpandedKeys: React.Key[]) => {
        setExpandedKeys(newExpandedKeys);
        setAutoExpandParent(false);
    };

    return (
        <Context.Provider value={{name: ""}}>
          <ConfigProvider theme={{algorithm: isDarkMode? darkAlgorithm : defaultAlgorithm}}>
            {contextHolder}
            <Spin spinning={loading}>
            <Layout className='layout' style={{ height: '97vh' }}>
                <Sider width={'30%'} style={{ height: '470' }} theme={isDarkMode ? "dark" : "light"}>
                    <div style={{overflow: 'auto', height: '90%'}}>
                        <Search style={{ marginBottom: 8 }} placeholder="Search" value={searchVal} onChange={setValAndSearch} />
                        <Tree
                            showLine={true}
                            onExpand={onExpand}
                            expandedKeys={expandedKeys}
                            autoExpandParent={autoExpandParent}
                            onSelect={onTreeNodeSelect}
                            selectedKeys={selectedTreeNodeKey}
                            treeData={treeData}
                        />
                    </div>
                    <Row align={'middle'} style={{height: '5'}} gutter={[24, 3]}>
                        <Col>
                            <Menu theme={isDarkMode ? "dark" : "light"} mode="horizontal" triggerSubMenuAction="click" items={items} onClick={handleMenuClick} />
                        </Col>
                        <Col span={10} />
                        <Col><Button onClick={createNew} >+</Button></Col>
                    </Row>
                </Sider>
                <Layout className="site-layout">
                    {/* <Header style={{ padding: 0, background: 'rgba(255, 255, 255, 0.2)' }} /> */}
                    <Content style={{ marginLeft: '4px' }} >
                        <div style={{ padding: 1, textAlign: 'center', width: '100%' }}>
                            <Row align={'middle'} gutter={[24, 3]} style={{ marginBottom: 50, marginTop: 10 }}>
                                <Col span={4}><span>Name:</span></Col>
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
                                    <Select
                                        style={{ width: 300 }}
                                        placeholder="Name of the connection's group"
                                        value={cc.group}
                                        onChange={updateGroup}
                                        dropdownRender={(menu) => (
                                            <>
                                                {menu}
                                                <Divider style={{ margin: '8px 0' }} />
                                                <Space style={{ padding: '0 8px 4px' }}>
                                                    <Input
                                                        placeholder="New group name"
                                                        ref={groupInputRef}
                                                        value={newGroupName}
                                                        onChange={updateNewGroupName}
                                                    />
                                                    <Button type="text" icon={<PlusOutlined />} onClick={addNewGroup}/>
                                                </Space>
                                            </>
                                        )}
                                        options={groupNames.map((name) => ({ label: name, value: name }))}
                                    />
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
        </ConfigProvider>
        </Context.Provider>
    );
}

export default App;
