import { Component, type ReactNode } from 'react';

interface Props { children: ReactNode }
interface State { error: Error | null }

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  render() {
    if (this.state.error) {
      return (
        <div style={{
          height: '100vh',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          background: '#0d1117',
          color: '#f85149',
          fontFamily: 'monospace',
          padding: '2rem',
          gap: '1rem',
        }}>
          <h2 style={{ color: '#f85149' }}>Studio crashed</h2>
          <pre style={{ background: '#161b22', padding: '1rem', borderRadius: '6px', maxWidth: '80vw', overflow: 'auto', fontSize: '12px', color: '#c9d1d9' }}>
            {this.state.error.message}
            {'\n'}
            {this.state.error.stack}
          </pre>
          <button
            style={{ background: '#58a6ff', color: '#000', border: 'none', padding: '8px 20px', borderRadius: '6px', cursor: 'pointer', fontWeight: 600 }}
            onClick={() => this.setState({ error: null })}
          >
            Retry
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
