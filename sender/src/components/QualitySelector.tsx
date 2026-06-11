type Quality = 'Low' | 'Medium' | 'High';

interface Props {
  value: Quality;
  onChange: (q: Quality) => void;
  disabled: boolean;
}

const LABELS: Record<Quality, string> = {
  Low:    'Basse (720p 15fps)',
  Medium: 'Moyenne (1080p 24fps)',
  High:   'Haute (1080p 30fps)',
};

export function QualitySelector({ value, onChange, disabled }: Props) {
  return (
    <div style={{ margin: '16px 0' }}>
      <label style={{ display: 'block', marginBottom: 8, fontWeight: 600 }}>
        Qualité du stream
      </label>
      <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
        {(Object.keys(LABELS) as Quality[]).map((q) => (
          <button
            key={q}
            disabled={disabled}
            onClick={() => onChange(q)}
            style={{
              padding: '8px 16px',
              borderRadius: 8,
              border: '2px solid',
              borderColor: value === q ? '#f97316' : '#e2e8f0',
              background: value === q ? '#fff7ed' : '#fff',
              cursor: disabled ? 'not-allowed' : 'pointer',
              opacity: disabled ? 0.5 : 1,
              fontWeight: value === q ? 700 : 400,
            }}
          >
            {LABELS[q]}
          </button>
        ))}
      </div>
    </div>
  );
}
