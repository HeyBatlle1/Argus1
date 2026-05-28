'use client';

import { History } from 'lucide-react';
import { ArgusEye } from './ArgusEye';
import { ConnectionStatus } from './ConnectionStatus';
import { ModelSelector } from './ModelSelector';
import { SentryDropdown } from './SentryDropdown';
import { useAgentStore } from '@/hooks/useAgentState';

interface Props {
  onToggleHistory: () => void;
  paneCount: 1 | 2 | 3;
  onSetPaneCount: (n: 1 | 2 | 3) => void;
}

export function Header({ onToggleHistory, paneCount, onSetPaneCount }: Props) {
  const title = useAgentStore((s) => s.currentConversationTitle);
  const conversations = useAgentStore((s) => s.conversations);

  return (
    <header
      className="fixed top-0 left-0 right-0 z-30 h-14 flex items-center justify-between px-4 border-b border-argus-border"
      style={{ background: '#0d0d14' }}
    >
      {/* Left: Logo + History */}
      <div className="flex items-center gap-3">
        <ArgusEye />
        <div className="flex flex-col">
          <span className="font-mono text-sm font-bold tracking-[0.2em] uppercase text-argus-amber leading-none">
            ARGUS
          </span>
          <span className="font-mono text-[9px] tracking-widest uppercase text-argus-textDim leading-none mt-0.5">
            The Hundred-Eyed Agent
          </span>
        </div>

        {/* Divider */}
        <div className="h-6 w-px mx-1" style={{ background: '#1e1e32' }} />

        {/* History button */}
        <button
          onClick={onToggleHistory}
          className="flex items-center gap-1.5 px-2.5 py-1.5 rounded transition-all text-[9px] font-mono tracking-wider uppercase"
          style={{
            border: '1px solid #1e1e32',
            color: '#5a5a7a',
            background: 'transparent',
          }}
          onMouseEnter={(e) => {
            (e.currentTarget as HTMLButtonElement).style.color = '#ffb000';
            (e.currentTarget as HTMLButtonElement).style.borderColor = 'rgba(255,176,0,0.3)';
            (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,176,0,0.06)';
          }}
          onMouseLeave={(e) => {
            (e.currentTarget as HTMLButtonElement).style.color = '#5a5a7a';
            (e.currentTarget as HTMLButtonElement).style.borderColor = '#1e1e32';
            (e.currentTarget as HTMLButtonElement).style.background = 'transparent';
          }}
          title={`${conversations.length} conversation${conversations.length !== 1 ? 's' : ''}`}
        >
          <History size={11} />
          HISTORY
          {conversations.length > 0 && (
            <span
              className="text-[8px] font-mono px-1 rounded"
              style={{ background: 'rgba(255,176,0,0.15)', color: '#ffb000' }}
            >
              {conversations.length}
            </span>
          )}
        </button>
      </div>

      {/* Center: current conversation title */}
      {title && (
        <div
          className="absolute left-1/2 -translate-x-1/2 max-w-xs truncate text-center"
        >
          <span className="text-[10px] font-mono" style={{ color: '#3a3a5a' }}>
            {title}
          </span>
        </div>
      )}

      {/* Right: Pane toggles + Connection + Sentry + Model */}
      <div className="flex items-center gap-2">
        {/* Split-pane controls */}
        <div
          className="flex items-center rounded overflow-hidden"
          style={{ border: '1px solid #1e1e32' }}
          title="Split panes"
        >
          {([1, 2, 3] as const).map((n) => (
            <button
              key={n}
              onClick={() => onSetPaneCount(n)}
              className="flex items-center justify-center transition-colors"
              style={{
                width: 26,
                height: 24,
                background: paneCount === n ? 'rgba(201,168,76,0.15)' : 'transparent',
                borderRight: n < 3 ? '1px solid #1e1e32' : undefined,
              }}
              title={`${n} pane${n > 1 ? 's' : ''}`}
            >
              <PaneIcon count={n} active={paneCount === n} />
            </button>
          ))}
        </div>

        <div className="h-4 w-px" style={{ background: '#1e1e32' }} />
        <ConnectionStatus />
        <SentryDropdown />
        <ModelSelector />
      </div>
    </header>
  );
}

function PaneIcon({ count, active }: { count: number; active: boolean }) {
  const color = active ? '#c9a84c' : '#3a3a5a';
  if (count === 1) return (
    <svg width="14" height="12" viewBox="0 0 14 12" fill="none">
      <rect x="1" y="1" width="12" height="10" rx="1" stroke={color} strokeWidth="1.2"/>
    </svg>
  );
  if (count === 2) return (
    <svg width="14" height="12" viewBox="0 0 14 12" fill="none">
      <rect x="1" y="1" width="5.5" height="10" rx="1" stroke={color} strokeWidth="1.2"/>
      <rect x="7.5" y="1" width="5.5" height="10" rx="1" stroke={color} strokeWidth="1.2"/>
    </svg>
  );
  return (
    <svg width="14" height="12" viewBox="0 0 14 12" fill="none">
      <rect x="1"   y="1" width="3.5" height="10" rx="0.8" stroke={color} strokeWidth="1.2"/>
      <rect x="5.2" y="1" width="3.5" height="10" rx="0.8" stroke={color} strokeWidth="1.2"/>
      <rect x="9.4" y="1" width="3.5" height="10" rx="0.8" stroke={color} strokeWidth="1.2"/>
    </svg>
  );
}
