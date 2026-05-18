export interface LauncherInfo {
  launcher_version: string
}

export interface Connection {
  address: string
  heapSize: string
  icon: string
  id: string
  javaHome: string
  javaArgs: string
  name: string
  username: string
  password: string
  group: string
  notes: string
  donotcache: boolean
  lastConnected: number | null
  showConsole: boolean
  peerCertificate: string | null

  // the below properties are transient and are used only in the UI
  nodeId: string
  parentId: string
}

export interface UntrustedCert {
  der?: string
  subject?: string
  issuer?: string
  expires_on?: string,
  sha256sum: string,
}

export interface LaunchResponse {
  code: number
  msg?: string
  peer?: UntrustedCert
  expected_fingerprint?: string
}
