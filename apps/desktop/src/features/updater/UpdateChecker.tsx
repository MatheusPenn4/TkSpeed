import { useEffect, useState } from 'react'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

export function UpdateChecker() {
  const [update, setUpdate] = useState<Update | null>(null)
  const [downloading, setDownloading] = useState(false)
  const [progress, setProgress] = useState(0)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    check()
      .then(u => { if (u?.available) setUpdate(u) })
      .catch(() => {})
  }, [])

  async function install() {
    if (!update) return
    setDownloading(true)
    setError(null)
    try {
      let downloaded = 0
      let total = 0
      await update.downloadAndInstall(event => {
        if (event.event === 'Started') {
          total = event.data.contentLength ?? 0
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength
          if (total > 0) setProgress(Math.round((downloaded / total) * 100))
        }
      })
      await relaunch()
    } catch (e) {
      setError('Falha ao instalar atualização.')
      setDownloading(false)
    }
  }

  if (!update) return null

  return (
    <div
      style={{
        position: 'fixed',
        bottom: 24,
        right: 24,
        zIndex: 9999,
        background: 'rgba(0, 10, 30, 0.95)',
        border: '1px solid rgba(0, 200, 255, 0.3)',
        borderRadius: 12,
        padding: '14px 18px',
        minWidth: 280,
        backdropFilter: 'blur(16px)',
        boxShadow: '0 0 30px rgba(0,200,255,0.1)',
        color: '#fff',
        fontFamily: 'inherit',
      }}
    >
      <div style={{ fontSize: 11, letterSpacing: 2, color: 'rgba(0,200,255,0.7)', marginBottom: 6, textTransform: 'uppercase' }}>
        Atualização disponível
      </div>
      <div style={{ fontSize: 13, marginBottom: 12 }}>
        Versão <strong style={{ color: '#00ffe7' }}>{update.version}</strong> disponível
      </div>
      {error && (
        <div style={{ fontSize: 11, color: '#ff6b6b', marginBottom: 8 }}>{error}</div>
      )}
      {downloading ? (
        <div>
          <div style={{ fontSize: 11, color: 'rgba(255,255,255,0.5)', marginBottom: 6 }}>
            Baixando... {progress}%
          </div>
          <div style={{ height: 2, background: 'rgba(0,200,255,0.1)', borderRadius: 2 }}>
            <div
              style={{
                height: '100%',
                width: `${progress}%`,
                background: 'linear-gradient(90deg, #00c8ff, #00ffe7)',
                borderRadius: 2,
                transition: 'width 0.2s',
              }}
            />
          </div>
        </div>
      ) : (
        <div style={{ display: 'flex', gap: 8 }}>
          <button
            onClick={install}
            style={{
              flex: 1,
              padding: '8px 0',
              background: 'rgba(0,200,255,0.12)',
              border: '1px solid rgba(0,200,255,0.35)',
              borderRadius: 8,
              color: '#00c8ff',
              cursor: 'pointer',
              fontSize: 12,
              letterSpacing: 1,
              fontFamily: 'inherit',
            }}
          >
            Atualizar agora
          </button>
          <button
            onClick={() => setUpdate(null)}
            style={{
              padding: '8px 12px',
              background: 'transparent',
              border: '1px solid rgba(255,255,255,0.1)',
              borderRadius: 8,
              color: 'rgba(255,255,255,0.4)',
              cursor: 'pointer',
              fontSize: 12,
              fontFamily: 'inherit',
            }}
          >
            Depois
          </button>
        </div>
      )}
    </div>
  )
}
