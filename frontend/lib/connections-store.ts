import fs from 'fs'
import path from 'path'

export type SocialConnection = {
  id: string
  platform: 'slack' | 'notion' | 'google_drive' | 'gmail' | 'dropbox' | 'linkedin' | 'github' | 'bitbucket' | 'gitlab'
  username: string
  is_active: boolean
  connected_at: string
  last_sync: string | null
}

const dataFile = path.join(process.cwd(), 'frontend', 'data', 'connections.json')

export function readConnections(): SocialConnection[] {
  try {
    if (!fs.existsSync(dataFile)) return []
    const raw = fs.readFileSync(dataFile, 'utf8')
    const parsed = JSON.parse(raw)
    return Array.isArray(parsed) ? parsed as SocialConnection[] : []
  } catch {
    return []
  }
}

export function writeConnections(cons: SocialConnection[]): void {
  try {
    const dir = path.dirname(dataFile)
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true })
    fs.writeFileSync(dataFile, JSON.stringify(cons, null, 2), 'utf8')
  } catch {}
}

export function upsertConnection(platform: SocialConnection['platform'], username = 'user') {
  const list = readConnections()
  const existing = list.find(c => c.platform === platform)
  if (existing) {
    existing.is_active = true
    existing.username = username
    existing.last_sync = new Date().toISOString()
    writeConnections(list)
    return existing
  }
  const conn: SocialConnection = {
    id: `${platform}-${Date.now()}`,
    platform,
    username,
    is_active: true,
    connected_at: new Date().toISOString(),
    last_sync: null
  }
  writeConnections([...list, conn])
  return conn
}

export function removeConnection(id: string) {
  const list = readConnections().filter(c => c.id !== id)
  writeConnections(list)
}