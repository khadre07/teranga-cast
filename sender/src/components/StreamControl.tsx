import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { QualitySelector } from './QualitySelector';

type Quality = 'Low' | 'Medium' | 'High';

interface StreamInfo {
  url: string;
  local_ip: string;
  port: number;
}

export function StreamControl() {
  const [running, setRunning] = useState(false);
  const [quality, setQuality] = useState<Quality>('Medium');
  const [info, setInfo] = useState<StreamInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleStart() {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<StreamInfo>('start_stream', { quality });
      setInfo(result);
      setRunning(true);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleStop() {
    setLoading(true);
    try {
      await invoke('stop_stream');
      setRunning(false);
      setInfo(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div style={{ fontFamily: 'sans-serif', maxWidth: 520, margin: '0 auto', padding: 32 }}>
      <h1 style={{ fontSize: '1.8rem', fontWeight: 800, color: '#1e293b', marginBottom: 4 }}>
        TerangaCast
      </h1>
      <p style={{ color: '#64748b', marginBottom: 24 }}>
        Diffusez votre écran sur la TV via Wi-Fi — sans câble, sans internet.
      </p>

      <QualitySelector value={quality} onChange={setQuality} disabled={running || loading} />

      <button
        onClick={running ? handleStop : handleStart}
        disabled={loading}
        style={{
          width: '100%',
          padding: '14px',
          borderRadius: 12,
          border: 'none',
          background: running ? '#ef4444' : '#f97316',
          color: '#fff',
          fontSize: '1.1rem',
          fontWeight: 700,
          cursor: loading ? 'wait' : 'pointer',
          marginTop: 8,
        }}
      >
        {loading ? '…' : running ? 'Arrêter le stream' : 'Démarrer le stream'}
      </button>

      {info && (
        <div
          style={{
            marginTop: 24,
            padding: 20,
            borderRadius: 12,
            background: '#f0fdf4',
            border: '1px solid #86efac',
          }}
        >
          <p style={{ fontWeight: 600, color: '#166534', marginBottom: 8 }}>Stream actif</p>
          <p style={{ color: '#15803d', marginBottom: 4 }}>Sur votre TV, ouvrez :</p>
          <code
            style={{
              display: 'block',
              padding: '10px 14px',
              borderRadius: 8,
              background: '#dcfce7',
              color: '#166534',
              fontSize: '1.1rem',
              fontWeight: 700,
              letterSpacing: 1,
            }}
          >
            {info.url}
          </code>
        </div>
      )}

      {error && (
        <div
          style={{
            marginTop: 16,
            padding: 14,
            borderRadius: 10,
            background: '#fef2f2',
            border: '1px solid #fca5a5',
            color: '#dc2626',
            fontSize: '0.9rem',
          }}
        >
          {error}
        </div>
      )}
    </div>
  );
}
