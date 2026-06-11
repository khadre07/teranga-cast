import { StreamControl } from './components/StreamControl';

export default function App() {
  return (
    <div
      style={{
        minHeight: '100vh',
        background: 'linear-gradient(135deg, #fff7ed 0%, #fef3c7 100%)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      <StreamControl />
    </div>
  );
}
