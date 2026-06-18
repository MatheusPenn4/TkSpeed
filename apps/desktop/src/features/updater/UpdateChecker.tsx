import { useEffect, useRef, useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

// ─── Types ────────────────────────────────────────────────────────────────────

type Phase =
  | 'idle'
  | 'available'
  | 'expanded'
  | 'downloading'
  | 'installing'
  | 'error'

interface DownloadStats {
  downloaded: number
  total: number
  speed: number      // bytes/s
  eta: number        // seconds
}

// ─── Constants ────────────────────────────────────────────────────────────────

const CYAN   = '#00BFFF'
const TEAL   = '#00FFE7'
const BG     = 'rgba(4, 8, 20, 0.97)'
const BORDER = 'rgba(0, 191, 255, 0.18)'
const GLASS  = 'rgba(0, 191, 255, 0.06)'

// ─── Helpers ──────────────────────────────────────────────────────────────────

function fmtBytes(b: number) {
  if (b < 1024) return `${b} B`
  if (b < 1048576) return `${(b / 1024).toFixed(1)} KB`
  return `${(b / 1048576).toFixed(1)} MB`
}

function fmtEta(s: number) {
  if (!isFinite(s) || s > 3600) return '—'
  if (s < 60) return `${Math.ceil(s)}s`
  return `${Math.floor(s / 60)}m ${Math.ceil(s % 60)}s`
}

function parseChangelog(body: string | null | undefined): string[] {
  if (!body) return []
  return body
    .split('\n')
    .map(l => l.replace(/^[-*•]\s*/, '').trim())
    .filter(l => l.length > 0 && !l.startsWith('#'))
    .slice(0, 6)
}

// ─── Component ────────────────────────────────────────────────────────────────

export function UpdateChecker() {
  const [phase, setPhase]     = useState<Phase>('idle')
  const [update, setUpdate]   = useState<Update | null>(null)
  const [stats, setStats]     = useState<DownloadStats>({ downloaded: 0, total: 0, speed: 0, eta: Infinity })
  const [errMsg, setErrMsg]   = useState('')
  const [dismissed, setDismissed] = useState(false)

  const lastTime = useRef<number>(0)
  const lastBytes = useRef<number>(0)

  // Check for update on mount
  useEffect(() => {
    check()
      .then(u => { if (u?.available) { setUpdate(u); setPhase('available') } })
      .catch((e: unknown) => {
        const msg = e instanceof Error ? e.message : String(e)
        // Ignora "no update" — só expõe erros reais (ex: falha de assinatura)
        if (!msg.toLowerCase().includes('no update')) {
          setErrMsg(msg)
          setPhase('error')
        }
      })
  }, [])

  // Re-check every hour silently
  useEffect(() => {
    const id = setInterval(() => {
      check()
        .then(u => {
          if (u?.available) { setUpdate(u); setDismissed(false); setPhase('available') }
        })
        .catch(() => {})
    }, 3_600_000)
    return () => clearInterval(id)
  }, [])

  async function startDownload() {
    if (!update) return
    setPhase('downloading')
    setStats({ downloaded: 0, total: 0, speed: 0, eta: Infinity })
    lastTime.current  = performance.now()
    lastBytes.current = 0

    try {
      await update.downloadAndInstall(event => {
        if (event.event === 'Started') {
          setStats(s => ({ ...s, total: event.data.contentLength ?? 0 }))
        } else if (event.event === 'Progress') {
          const chunk = event.data.chunkLength
          const now   = performance.now()
          const dt    = (now - lastTime.current) / 1000

          if (dt > 0.25) {
            const bytesDelta = (lastBytes.current + chunk)
            const speed      = bytesDelta / dt
            setStats(s => {
              const downloaded = s.downloaded + chunk
              const remaining  = s.total > 0 ? s.total - downloaded : 0
              return {
                downloaded,
                total: s.total,
                speed,
                eta: speed > 0 ? remaining / speed : Infinity,
              }
            })
            lastTime.current  = now
            lastBytes.current = 0
          } else {
            lastBytes.current += chunk
            setStats(s => ({ ...s, downloaded: s.downloaded + chunk }))
          }
        } else if (event.event === 'Finished') {
          setPhase('installing')
        }
      })
      await relaunch()
    } catch (e: unknown) {
      setErrMsg(e instanceof Error ? e.message : 'Falha no download.')
      setPhase('error')
    }
  }

  if (dismissed || phase === 'idle') return null

  const changelog = parseChangelog(update?.body)
  const pct = stats.total > 0 ? Math.round((stats.downloaded / stats.total) * 100) : 0

  return (
    <AnimatePresence>
      <motion.div
        key="updater-root"
        initial={{ opacity: 0, y: 20, scale: 0.97 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: 16, scale: 0.97 }}
        transition={{ duration: 0.35, ease: [0.16, 1, 0.3, 1] }}
        style={{
          position: 'fixed',
          bottom: 24,
          right: 24,
          zIndex: 9999,
          width: 340,
          borderRadius: 20,
          background: BG,
          border: `1px solid ${BORDER}`,
          boxShadow: `0 0 0 1px rgba(255,255,255,0.03), 0 24px 60px rgba(0,0,0,0.7), 0 0 40px rgba(0,191,255,0.07)`,
          overflow: 'hidden',
          fontFamily: 'inherit',
          color: '#fff',
        }}
      >
        {/* Top glow line */}
        <div style={{
          position: 'absolute', top: 0, left: 40, right: 40, height: 1,
          background: `linear-gradient(90deg, transparent, ${CYAN}, transparent)`,
          opacity: 0.6,
        }} />

        {/* ── AVAILABLE ── */}
        {(phase === 'available' || phase === 'expanded') && update && (
          <div style={{ padding: '18px 20px 20px' }}>

            {/* Header */}
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 14 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                <div style={{
                  width: 32, height: 32, borderRadius: 10,
                  background: GLASS, border: `1px solid ${BORDER}`,
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  boxShadow: `0 0 16px rgba(0,191,255,0.15)`,
                }}>
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke={CYAN} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="23 4 23 10 17 10" />
                    <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
                  </svg>
                </div>
                <div>
                  <div style={{ fontSize: 12, fontWeight: 600, letterSpacing: '0.02em', lineHeight: 1.2 }}>
                    Atualização disponível
                  </div>
                  <div style={{ fontSize: 11, color: CYAN, fontFamily: 'monospace', letterSpacing: '0.05em', marginTop: 2 }}>
                    v{update.version}
                  </div>
                </div>
              </div>

              <button
                onClick={() => setDismissed(true)}
                style={{
                  width: 26, height: 26, borderRadius: 8,
                  background: 'rgba(255,255,255,0.04)',
                  border: '1px solid rgba(255,255,255,0.07)',
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  cursor: 'pointer', color: 'rgba(255,255,255,0.4)',
                  transition: 'background 0.15s',
                }}
                onMouseEnter={e => (e.currentTarget.style.background = 'rgba(255,255,255,0.09)')}
                onMouseLeave={e => (e.currentTarget.style.background = 'rgba(255,255,255,0.04)')}
              >
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round">
                  <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>

            {/* Changelog — expanded */}
            <AnimatePresence>
              {phase === 'expanded' && changelog.length > 0 && (
                <motion.div
                  initial={{ height: 0, opacity: 0 }}
                  animate={{ height: 'auto', opacity: 1 }}
                  exit={{ height: 0, opacity: 0 }}
                  transition={{ duration: 0.28, ease: [0.16, 1, 0.3, 1] }}
                  style={{ overflow: 'hidden' }}
                >
                  <div style={{
                    borderRadius: 12, background: 'rgba(0,191,255,0.04)',
                    border: '1px solid rgba(0,191,255,0.1)',
                    padding: '10px 12px', marginBottom: 14,
                  }}>
                    <div style={{ fontSize: 9, letterSpacing: '0.14em', color: 'rgba(0,191,255,0.6)', fontFamily: 'monospace', marginBottom: 8, textTransform: 'uppercase' }}>
                      Novidades
                    </div>
                    {changelog.map((item, i) => (
                      <div key={i} style={{ display: 'flex', gap: 8, marginBottom: 5, fontSize: 11, color: 'rgba(255,255,255,0.65)', lineHeight: 1.4 }}>
                        <span style={{ color: CYAN, flexShrink: 0, marginTop: 1 }}>·</span>
                        <span>{item}</span>
                      </div>
                    ))}
                  </div>
                </motion.div>
              )}
            </AnimatePresence>

            {/* Divider */}
            <div style={{ height: 1, background: 'rgba(255,255,255,0.05)', marginBottom: 14 }} />

            {/* Actions */}
            <div style={{ display: 'flex', gap: 8 }}>
              {changelog.length > 0 && (
                <button
                  onClick={() => setPhase(phase === 'expanded' ? 'available' : 'expanded')}
                  style={{
                    flex: 1, height: 36, borderRadius: 10,
                    background: 'rgba(255,255,255,0.04)',
                    border: '1px solid rgba(255,255,255,0.08)',
                    color: 'rgba(255,255,255,0.55)', fontSize: 11,
                    fontWeight: 500, cursor: 'pointer', letterSpacing: '0.03em',
                    transition: 'all 0.15s',
                  }}
                  onMouseEnter={e => (e.currentTarget.style.background = 'rgba(255,255,255,0.08)')}
                  onMouseLeave={e => (e.currentTarget.style.background = 'rgba(255,255,255,0.04)')}
                >
                  {phase === 'expanded' ? 'Ocultar' : 'Ver novidades'}
                </button>
              )}
              <button
                onClick={startDownload}
                style={{
                  flex: 2, height: 36, borderRadius: 10,
                  background: `linear-gradient(135deg, ${CYAN}, #0099DD)`,
                  border: '1px solid rgba(255,255,255,0.12)',
                  color: '#fff', fontSize: 12, fontWeight: 600,
                  cursor: 'pointer', letterSpacing: '0.04em',
                  boxShadow: `0 0 20px rgba(0,191,255,0.3)`,
                  transition: 'all 0.15s',
                  display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 6,
                }}
                onMouseEnter={e => { e.currentTarget.style.boxShadow = `0 0 32px rgba(0,191,255,0.55)`; e.currentTarget.style.transform = 'translateY(-1px)' }}
                onMouseLeave={e => { e.currentTarget.style.boxShadow = `0 0 20px rgba(0,191,255,0.3)`; e.currentTarget.style.transform = 'translateY(0)' }}
              >
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                  <polyline points="7 10 12 15 17 10" />
                  <line x1="12" y1="15" x2="12" y2="3" />
                </svg>
                Atualizar agora
              </button>
            </div>
          </div>
        )}

        {/* ── DOWNLOADING ── */}
        {phase === 'downloading' && (
          <div style={{ padding: '18px 20px 20px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 16 }}>
              <div style={{
                width: 32, height: 32, borderRadius: 10,
                background: GLASS, border: `1px solid ${BORDER}`,
                display: 'flex', alignItems: 'center', justifyContent: 'center',
              }}>
                <motion.div
                  animate={{ rotate: 360 }}
                  transition={{ duration: 1.2, repeat: Infinity, ease: 'linear' }}
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke={CYAN} strokeWidth="2.5" strokeLinecap="round">
                    <path d="M21 12a9 9 0 1 1-6.219-8.56" />
                  </svg>
                </motion.div>
              </div>
              <div>
                <div style={{ fontSize: 12, fontWeight: 600 }}>Baixando atualização</div>
                <div style={{ fontSize: 10, color: 'rgba(255,255,255,0.4)', fontFamily: 'monospace', marginTop: 2 }}>
                  {fmtBytes(stats.downloaded)}{stats.total > 0 ? ` / ${fmtBytes(stats.total)}` : ''}
                  {stats.speed > 0 && ` · ${fmtBytes(stats.speed)}/s`}
                  {isFinite(stats.eta) && ` · ${fmtEta(stats.eta)}`}
                </div>
              </div>
              <div style={{ marginLeft: 'auto', fontSize: 20, fontWeight: 700, color: CYAN, fontFamily: 'monospace', letterSpacing: '-0.02em' }}>
                {pct}%
              </div>
            </div>

            {/* Progress track */}
            <div style={{ height: 4, borderRadius: 4, background: 'rgba(255,255,255,0.06)', overflow: 'hidden' }}>
              <motion.div
                animate={{ width: `${pct}%` }}
                transition={{ duration: 0.3, ease: 'linear' }}
                style={{
                  height: '100%', borderRadius: 4,
                  background: `linear-gradient(90deg, ${CYAN}, ${TEAL})`,
                  boxShadow: `0 0 10px ${CYAN}`,
                }}
              />
            </div>

            {/* Segment indicators */}
            <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: 6 }}>
              {[0, 25, 50, 75, 100].map(mark => (
                <div key={mark} style={{ fontSize: 9, fontFamily: 'monospace', color: pct >= mark ? `rgba(0,191,255,0.5)` : 'rgba(255,255,255,0.15)' }}>
                  {mark}%
                </div>
              ))}
            </div>
          </div>
        )}

        {/* ── INSTALLING ── */}
        {phase === 'installing' && (
          <div style={{ padding: '20px', display: 'flex', alignItems: 'center', gap: 12 }}>
            <div style={{
              width: 32, height: 32, borderRadius: 10, flexShrink: 0,
              background: 'rgba(0,255,100,0.08)', border: '1px solid rgba(0,255,100,0.2)',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              boxShadow: '0 0 16px rgba(0,255,100,0.12)',
            }}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#00FF88" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </div>
            <div>
              <div style={{ fontSize: 12, fontWeight: 600 }}>Download concluído</div>
              <div style={{ fontSize: 10, color: 'rgba(255,255,255,0.4)', marginTop: 2 }}>
                Instalando e reiniciando...
              </div>
            </div>
          </div>
        )}

        {/* ── ERROR ── */}
        {phase === 'error' && (
          <div style={{ padding: '18px 20px 20px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 14 }}>
              <div style={{
                width: 32, height: 32, borderRadius: 10, flexShrink: 0,
                background: 'rgba(255,60,60,0.08)', border: '1px solid rgba(255,60,60,0.2)',
                display: 'flex', alignItems: 'center', justifyContent: 'center',
              }}>
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="#FF5555" strokeWidth="2.5" strokeLinecap="round">
                  <circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" />
                </svg>
              </div>
              <div>
                <div style={{ fontSize: 12, fontWeight: 600, color: '#FF7070' }}>Falha na atualização</div>
                <div style={{ fontSize: 10, color: 'rgba(255,255,255,0.4)', marginTop: 2, maxWidth: 220 }}>{errMsg}</div>
              </div>
            </div>
            <div style={{ display: 'flex', gap: 8 }}>
              <button
                onClick={() => setDismissed(true)}
                style={{
                  flex: 1, height: 34, borderRadius: 10,
                  background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)',
                  color: 'rgba(255,255,255,0.5)', fontSize: 11, cursor: 'pointer',
                }}
              >
                Fechar
              </button>
              <button
                onClick={startDownload}
                style={{
                  flex: 2, height: 34, borderRadius: 10,
                  background: 'rgba(255,80,80,0.15)', border: '1px solid rgba(255,80,80,0.25)',
                  color: '#FF7070', fontSize: 11, fontWeight: 600, cursor: 'pointer',
                }}
              >
                Tentar novamente
              </button>
            </div>
          </div>
        )}
      </motion.div>
    </AnimatePresence>
  )
}
