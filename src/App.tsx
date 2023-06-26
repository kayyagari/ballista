import React, {useEffect, useState} from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from '@tauri-apps/api/dialog';
import "./App.css";
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
    notification,
    theme,
    Upload
} from "antd";
import { ApiOutlined, EyeInvisibleOutlined, EyeTwoTone, SettingOutlined } from "@ant-design/icons";
import type { NotificationPlacement } from 'antd/es/notification/interface';

const { Content, Sider } = Layout;

interface Connection {
    address: string,
    heapSize: string,
    icon: string,
    id: string,
    javaHome: string,
    name: string,
    username: string,
    password: string,

    // setAddress: (address: string) => void,
    // setHeapSize: (heapSize: string) => void,
    // setIcon: (icon: string) => void,
    // setId: (id: string) => void,
    // setJavaHome: (javaHome: string) => void,
    // setName: (name: string) => void,
    // setUsername: (username: string) => void,
    // setPassword: (password: string) => void,
    // setCurrentConnection: (cc: Connection) => void
}

/*const useResourceSearchStore = create<Connection>((set) => ({
    address: "",
    heapSize: "",
    icon: "",
    id: "",
    javaHome: "",
    name: "",
    username: "",
    password: "",
    setAddress: (a) => set(() => ({ address: a })),
    setHeapSize: (hs) => set(() => ({ heapSize: hs })),
    setIcon: (ic) => set(() => ({ icon: ic })),
    setId: (id) => set(() => ({ id: id })),
    setJavaHome: (jh) => set(() => ({ javaHome: jh })),
    setName: (n) => set(() => ({ name: n })),
    setUsername: (un) => set(() => ({ username: un })),
    setPassword: (p) => set(() => ({ password: p })),
    setCurrentConnection: (cc: Connection) => set(() => ({
        address: cc.address,
        heapSize: cc.heapSize,
        icon: cc.icon,
        id: cc.id,
        javaHome: cc.javaHome,
        name: cc.name,
        username: cc.username,
        password: cc.password
    }))
}));*/

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
    // const [api, contextHolder] = notification.useNotification();
    // const openNotification = (placement: NotificationPlacement, msg: string) => {
    //     api.info({
    //         message: `Notification ${placement}`,
    //         description: <Context.Consumer>{({ name }) => `${msg}`}</Context.Consumer>,
    //         placement,
    //     });
    // };

    //const store = useResourceSearchStore();

    const [data, setData] = useState<Connection[]>([]);

    const [cc, setCc] = useState<Connection>({
        address: "",
        heapSize: "",
        icon: "",
        id: "",
        javaHome: "",
        name: "",
        username: "",
        password: ""});

    useEffect(() => {loadConnections().then(d => {
        setData(d);
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
                for(let i =0; i < d.length; i++) {
                    if(cc.id == d[i].id) {
                        setCc(d[i]);
                    }
                }
            })
        }
    }

    async function loadConnections() {
        console.log("loading connections");
        const jsonArr: string = await invoke("load_connections");
        let data = JSON.parse(jsonArr);
        data.sort((c1: Connection, c2: Connection) => {
            let n1 = c1.name.toLowerCase();
            let n2 = c2.name.toLowerCase();
            if(n1 > n2) {
                return 1;
            }
            else if(n1 < n2) {
                return -1;
            }

            return 0;
        });
        return data;
    }

    async function launch() {
        await invoke("launch", { url: cc.address, java_home: cc.javaHome, username: cc.username == null ? '' : cc.username, password: cc.password == null ? '' : cc.password });
        //openNotification('topLeft', msg);
    }
    function createNew() {

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
    }

    function updateUrl(e: any) {
        setCc({
            ...cc,
            address: e.target.value
        })
    }
    function updateUsername(e: any) {
        setCc({
            ...cc,
            username : e.target.value
        })
    }

    function updatePassword(e: any) {
        setCc({
            ...cc,
            password: e.target.value
        })
    }
    function updateJavaHome(e: any) {
        setCc({
            ...cc,
            javaHome: e.target.value
        })
    }

    function deleteConnection() {
    }

    function saveConnection() {

    }

    function selectConnection(index: number) {
        setCc(data[index]);
        console.log("selected connection " + index);
    }

    return (
        <Layout className='layout' style={{ height: '90vh' }}>
            <Sider width={'30%'} style={{ height: '400' }} theme={"light"} >
                <div style={{overflow: 'auto', height: '79%'}}>
                    <List header={<span style={{ margin: '34%', fontSize: 15, color: 'brown'}}>Connections</span>}
                          size="small"
                          dataSource={data}
                          renderItem={(item, index) => (
                              <List.Item key={item.id} onClick={() => selectConnection(index)}>
                                  <List.Item.Meta
                                      avatar={<Avatar icon={<ApiOutlined />} />}
                                      title={item.name}
                                      description={item.name}
                                  />
                              </List.Item>
                          )}
                    />
                </div>
                <Divider />
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
                                <Input placeholder={"Connection's name e.g Acme Test Instance"} size={"middle"} bordered value={cc.name} onChange={updateName} />
                            </Col>
                        </Row>
                        <Row align={"middle"} gutter={[24, 3]} style={{ marginBottom: 8 }}>
                            <Col span={4} style={{ textAlign: "center" }}>URL:</Col>
                            <Col span={16}>
                                <Input placeholder="MC URL e.g https://localhost:8443" size={"middle"} bordered value={cc.address} onChange={updateUrl} onPressEnter={launch} />
                            </Col>
                            <Col><Button type={"primary"} onClick={launch} disabled={cc.address == ""} >Open</Button></Col>
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
                        {/*<Row>*/}
                        {/*    <Col span={20} style={{ marginTop: 20, alignContent: "end" }}><Button type={"primary"} onClick={saveConnection}>Save</Button></Col>*/}
                        {/*</Row>*/}
                        {/*<Row style={{ marginTop: 230 }}>*/}
                        {/*    <Col style={{ alignContent: "end" }}><Button type={"primary"} danger onClick={deleteConnection}>Delete</Button></Col>*/}
                        {/*</Row>*/}
                    </div>
                </Content>
            </Layout>
        </Layout>
    );
}

export default App;