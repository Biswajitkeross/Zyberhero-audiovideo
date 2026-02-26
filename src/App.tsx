import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import './App.css'

interface AlertLog {
  id: number
  message: string
  timestamp: string
  badWords: string[]
}

interface SimpleMonitoringStatus {
  is_monitoring: boolean
  detection_count: number
}

type Device = [string, string]; // [id, name]

function App() {
  const [logs, setLogs] = useState<AlertLog[]>([])
  const [badWords, setBadWords] = useState<string[]>([])
  const [newWord, setNewWord] = useState('')
  const [isDetectionEnabled, setIsDetectionEnabled] = useState(true)
  const [status, setStatus] = useState('Loading...')
  const [testText, setTestText] = useState('')

  // Add Simple Monitoring Status
  const [simpleStatus, setSimpleStatus] = useState<SimpleMonitoringStatus>({
    is_monitoring: false,
    detection_count: 0,
  })

  // Audio Device State
  const [devices, setDevices] = useState<Device[]>([])
  const [selectedDeviceId, setSelectedDeviceId] = useState<string>('')

  const logIdRef = useRef(0)

  // Move addLog and refreshDevices up for hoisting
  const addLog = (message: string, badWordsList: string[], details: string) => {
    const timestamp = new Date().toLocaleTimeString()
    logIdRef.current += 1
    const log: AlertLog = {
      id: logIdRef.current,
      message: `${message} - ${details}`,
      timestamp,
      badWords: badWordsList,
    }
    setLogs((prev) => [log, ...prev].slice(0, 50))
  }

  const refreshDevices = async () => {
    try {
      const devs = await invoke<Device[]>('get_output_devices')
      setDevices(devs)
      if (devs.length > 0 && !selectedDeviceId) {
        const defaultDev = devs.find(d => d[1].toLowerCase().includes("headphone")) || devs[0];
        setSelectedDeviceId(defaultDev[0])
      }
      addLog('Devices Refreshed', [], `${devs.length} devices found`)
    } catch (e) {
      console.error('Failed to get devices', e)
    }
  }

  // Initialize app
  useEffect(() => {
    const init = async () => {
      try {
        const appStatus = await invoke<string>('get_status')
        setStatus(appStatus)

        const words = await invoke<string[]>('get_all_bad_words')
        setBadWords(words)

        // Fetch Output Devices
        refreshDevices()

        // Check monitoring status on load
        try {
          const simple = await invoke<SimpleMonitoringStatus>('get_simple_monitoring_status')
          setSimpleStatus(simple)
        } catch (_e) {
          console.log("Simple monitoring not available yet")
        }
      } catch (error) {
        console.error('Failed to initialize:', error)
        setStatus('Error: Failed to initialize')
      }
    }

    init()

    // Listen for backend alerts (now includes count info)
    const unlisten = listen('bad-word-detected', (event) => {
      console.log('Backend Event:', event)
      const payload = event.payload as { word: string; count: number; max: number } | string
      if (typeof payload === 'object') {
        addLog(`🚨 STRIKE ${payload.count}/${payload.max}`, [payload.word], `Bad word detected!`)
        setSimpleStatus(prev => ({ ...prev, detection_count: payload.count }))
      } else {
        addLog('🔊 Alert Triggered', [payload], 'Bad Word Detected')
      }
    })

    // Listen for pause event
    const unlistenPause = listen('pause-audio-app', (event) => {
      console.log('Pause Event Received:', event.payload)
      addLog('⏸ PAUSE COMMAND', [], `System detected 3+ bad words. App to pause: ${event.payload}`)
    })

    // Listen for app closing event (3 strikes)
    const unlistenClose = listen('app-closing', (event) => {
      console.log('App Closing Event:', event.payload)
      addLog('🛑 APP CLOSING', [], `3 strikes reached - application will close`)
    })

    return () => {
      unlisten.then(f => f())
      unlistenPause.then(f => f())
      unlistenClose.then(f => f())
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Audio Engine (Frontend Beeps)
  const audioCtxRef = useRef<AudioContext | null>(null)

  const getAudioContext = () => {
    if (!audioCtxRef.current) {
      audioCtxRef.current = new (window.AudioContext || (window as any).webkitAudioContext)()
    }
    return audioCtxRef.current
  }

  const playTone = async (freq: number, duration: number) => {
    try {
      const ctx = getAudioContext()
      if (ctx.state === 'suspended') {
        await ctx.resume()
      }
      const osc = ctx.createOscillator()
      const gain = ctx.createGain()
      osc.type = 'sine'
      osc.frequency.setValueAtTime(freq, ctx.currentTime)
      gain.gain.setValueAtTime(1.0, ctx.currentTime)
      osc.connect(gain)
      gain.connect(ctx.destination)
      osc.start()
      osc.stop(ctx.currentTime + duration)
    } catch (e) {
      console.error("Audio Play Error:", e)
    }
  }

  const playSingleBeep = () => playTone(1000, 0.5)
  const playDoubleBeep = () => {
    playTone(1000, 0.3)
    setTimeout(() => playTone(1200, 0.3), 420)
  }
  const playAscending = () => {
    playTone(800, 0.2)
    setTimeout(() => playTone(1000, 0.2), 200)
    setTimeout(() => playTone(1200, 0.2), 400)
  }
  const playLongBeep = () => playTone(440, 5.0)

  // Play alert test
  const handlePlayAlert = async (type: 'single' | 'double' | 'ascending' | 'long') => {
    try {
      switch (type) {
        case 'single': playSingleBeep(); break;
        case 'double': playDoubleBeep(); break;
        case 'ascending': playAscending(); break;
      }
      addLog(`Tested Alert: ${type}`, [], 'Success')
    } catch (error) {
      addLog('Alert failed', [], String(error))
    }
  }

  // Add bad word
  const handleAddWord = async () => {
    if (!newWord.trim()) return
    try {
      await invoke<string>('add_bad_word', { word: newWord })
      const words = await invoke<string[]>('get_all_bad_words')
      setBadWords(words)
      setNewWord('')
      addLog(`Added bad word: ${newWord}`, [newWord], 'Success')
    } catch (error) {
      addLog('Failed to add word', [], String(error))
    }
  }

  // Remove bad word
  const handleRemoveWord = async (word: string) => {
    try {
      await invoke<string>('remove_bad_word', { word })
      const words = await invoke<string[]>('get_all_bad_words')
      setBadWords(words)
      addLog(`Removed bad word: ${word}`, [word], 'Success')
    } catch (error) {
      addLog('Failed to remove word', [], String(error))
    }
  }

  // Clear all bad words
  const handleClearAllWords = async () => {
    try {
      await invoke<string>('clear_bad_words')
      setBadWords([])
      addLog('Cleared all bad words', [], 'Success')
    } catch (error) {
      addLog('Failed to clear', [], String(error))
    }
  }

  // Toggle detection
  const handleToggleDetection = async () => {
    try {
      const newState = !isDetectionEnabled
      await invoke<string>('set_detection_enabled', { enabled: newState })
      setIsDetectionEnabled(newState)
      addLog(`Detection ${newState ? 'enabled' : 'disabled'}`, [], 'Success')
    } catch (error) {
      addLog('Failed to toggle detection', [], String(error))
    }
  }

  // Check text for bad words
  const handleCheckText = async () => {
    if (!testText.trim()) return
    try {
      const found = await invoke<string[]>('check_bad_words', { text: testText })
      addLog(`Checked text`, found, found.length > 0 ? 'Bad words found!' : 'Clean')
      if (found.length > 0) {
        await handlePlayAlert('single')
      }
    } catch (error) {
      addLog('Failed to check text', [], String(error))
    }
  }

  // Start system audio monitoring
  const handleStartMonitoring = async () => {
    try {
      const result = await invoke<string>('start_simple_monitoring')
      addLog('System Audio Monitoring Started', [], result)

      // Update state immediately
      setSimpleStatus(prev => ({ ...prev, is_monitoring: true }))
    } catch (error) {
      console.error('Failed to start monitoring:', error)
      addLog('Monitoring failed', [], String(error))
    }
  }

  // Stop system audio monitoring
  const handleStopMonitoring = async () => {
    try {
      const result = await invoke<string>('stop_simple_monitoring')
      addLog('System Audio Monitoring Stopped', [], result)
      setSimpleStatus({ is_monitoring: false, detection_count: 0 })
    } catch (error) {
      addLog('Failed to stop monitoring', [], String(error))
    }
  }

  // Poll for status updates
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const status = await invoke<SimpleMonitoringStatus>('get_simple_monitoring_status')
        setSimpleStatus(status)
      } catch (error) {
        // Silently ignore poll errors when app is starting
      }
    }, 1000)
    return () => clearInterval(interval)
  }, [])

  return (
    <div className="app-container">
      <header className="header">
        <h1>🎵 Audio Content Monitor</h1>
        <p className="status">Status: {status}</p>
      </header>

      <main className="main-content">

        {/* System Audio Monitoring Section */}
        <section className="section">
          <h2>📡 System Audio Monitoring</h2>
          <p className="section-description">
            1. Select your <b>Output Device</b> (where you hear sound) below.<br />
            2. Click Start Monitoring.<br />
            3. The system will automatically detect and alert you to bad words via real-time WASAPI loopback.
          </p>

          <div className="device-selection">
            <label>Output Device (To Ears):</label>
            <select
              value={selectedDeviceId}
              onChange={(e) => setSelectedDeviceId(e.target.value)}
              className="device-select"
              disabled={simpleStatus.is_monitoring}
            >
              <option value="">-- Default Speakers/Headphones --</option>
              {devices.map(([id, name]) => (
                <option key={id} value={id}>{name}</option>
              ))}
            </select>
            <button onClick={refreshDevices} className="btn btn-secondary btn-small" disabled={simpleStatus.is_monitoring}>
              🔄
            </button>
          </div>

          <div className="button-group" style={{ marginTop: '1rem' }}>
            {!simpleStatus.is_monitoring ? (
              <button
                onClick={handleStartMonitoring}
                className="btn btn-success btn-large"
              >
                ▶ Start Monitoring
              </button>
            ) : (
              <button onClick={handleStopMonitoring} className="btn btn-danger btn-large">
                ⏹ Stop Monitoring
              </button>
            )}
          </div>

          <div className="monitoring-status">
            <div className={`status-indicator ${simpleStatus.is_monitoring ? 'active' : 'inactive'}`}>
              {simpleStatus.is_monitoring ? '🔴 MONITORING ACTIVE' : '⚫ Monitoring Inactive'}
            </div>

            {/* 3-Strike Counter Display */}
            {simpleStatus.is_monitoring && (
              <div className="strike-counter" style={{ 
                marginTop: '1rem', 
                padding: '1rem', 
                background: simpleStatus.detection_count >= 3 ? '#ff4444' : simpleStatus.detection_count > 0 ? '#ff8800' : '#333',
                borderRadius: '8px',
                textAlign: 'center'
              }}>
                <div style={{ fontSize: '2rem', fontWeight: 'bold' }}>
                  {simpleStatus.detection_count >= 3 ? '🛑 CLOSING APP' : `⚠️ ${simpleStatus.detection_count}/3 STRIKES`}
                </div>
                <div style={{ fontSize: '0.9rem', marginTop: '0.5rem', opacity: 0.8 }}>
                  {simpleStatus.detection_count === 0 && 'No bad words detected yet'}
                  {simpleStatus.detection_count === 1 && '1 bad word detected - 2 more and app closes'}
                  {simpleStatus.detection_count === 2 && '2 bad words detected - 1 more and app closes!'}
                  {simpleStatus.detection_count >= 3 && 'Music paused. App closing...'}
                </div>
              </div>
            )}
          </div>
        </section>

        <section className="section">
          <h2>🚫 Bad Word Management</h2>

          <div className="word-input-group">
            <input
              type="text"
              value={newWord}
              onChange={(e) => setNewWord(e.target.value)}
              placeholder="Enter bad word to add..."
              className="input"
              onKeyPress={(e) => e.key === 'Enter' && handleAddWord()}
            />
            <button onClick={handleAddWord} className="btn btn-primary">
              Add Word
            </button>
            <button onClick={handleClearAllWords} className="btn btn-danger">
              Clear All
            </button>
            <button onClick={handleToggleDetection} className={`btn ${isDetectionEnabled ? 'btn-success' : 'btn-secondary'}`}>
              {isDetectionEnabled ? '✓ Detection ON' : '✗ Detection OFF'}
            </button>
          </div>

          <div className="words-list">
            <h3>Current Bad Words ({badWords.length})</h3>
            {badWords.length === 0 ? (
              <p className="empty-message">No bad words configured</p>
            ) : (
              <ul className="word-tags">
                {badWords.map((word) => (
                  <li key={word} className="word-tag">
                    {word}
                    <button
                      onClick={() => handleRemoveWord(word)}
                      className="remove-btn"
                      title="Remove this word"
                    >
                      ×
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </section>

        {/* Text Test Section */}
        <section className="section">
          <h2>🧪 Test Text for Bad Words</h2>
          <textarea
            value={testText}
            onChange={(e) => setTestText(e.target.value)}
            placeholder="Enter text to check for bad words..."
            className="textarea"
            rows={4}
          />
          <button onClick={handleCheckText} className="btn btn-primary">
            Check Text
          </button>
        </section>

        {/* Activity Log Section */}
        <section className="section">
          <h2>📋 Activity Log</h2>
          <div className="log-container">
            {logs.length === 0 ? (
              <p className="empty-message">No activity yet</p>
            ) : (
              <ul className="log-list">
                {logs.map((log) => (
                  <li key={log.id} className="log-entry">
                    <span className="log-time">{log.timestamp}</span>
                    <span className="log-message">{log.message}</span>
                    {log.badWords.length > 0 && (
                      <span className="log-words">
                        [{log.badWords.join(', ')}]
                      </span>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </div>
        </section>

        {/* Alert Test Section (Frontend) */}
        <section className="section">
          <h2>🔊 Alert Test (Frontend)</h2>
          <div className="button-group">
            <button onClick={() => handlePlayAlert('single')} className="btn btn-primary">
              Single Beep
            </button>
            <button onClick={() => handlePlayAlert('double')} className="btn btn-warning">
              Double Beep
            </button>
            <button onClick={() => handlePlayAlert('ascending')} className="btn btn-success">
              Ascending
            </button>
            <button onClick={playLongBeep} className="btn btn-danger" style={{ marginLeft: '10px' }}>
              🔊 Test 5s
            </button>
          </div>
        </section>

      </main>
    </div>
  )
}

export default App
