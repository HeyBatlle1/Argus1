'use client';

import { useEffect, useRef } from 'react';
import { useAgentStore } from '@/hooks/useAgentState';
import { MessageList } from './MessageList';
import { InputArea } from './InputArea';
import { Artifact } from '@/lib/types';
import { Plus } from 'lucide-react';

interface Props {
  onOpenArtifact: (artifacts: Artifact[], index: number) => void;
  meetingBrief?: string;
}

// Ferris watermark — amber on dark, visible but quiet
const FERRIS_SVG = `
<svg viewBox="0 0 340 220" xmlns="http://www.w3.org/2000/svg">
  <g transform="translate(168, 110)" fill="#c9a84c" stroke="#c9a84c">
    <!-- Body -->
    <ellipse cx="0" cy="0" rx="52" ry="38"/>
    <ellipse cx="0" cy="-5" rx="34" ry="22" fill="none" stroke-width="1.5"/>
    <!-- Eyes -->
    <circle cx="-20" cy="-20" r="13"/>
    <circle cx="-18" cy="-20" r="7" fill="#0a0a0f" stroke="none"/>
    <circle cx="-15" cy="-23" r="2" fill="#c9a84c" stroke="none"/>
    <circle cx="20" cy="-20" r="13"/>
    <circle cx="22" cy="-20" r="7" fill="#0a0a0f" stroke="none"/>
    <circle cx="25" cy="-23" r="2" fill="#c9a84c" stroke="none"/>
    <!-- Antennae -->
    <line x1="-30" y1="-34" x2="-13" y2="-31" stroke-width="2.5" stroke-linecap="round" fill="none"/>
    <line x1="13" y1="-31" x2="30" y2="-34" stroke-width="2.5" stroke-linecap="round" fill="none"/>
    <!-- Left claw -->
    <path d="M-50,0 Q-64,-8 -72,-12" fill="none" stroke-width="11" stroke-linecap="round"/>
    <ellipse cx="-72" cy="-12" rx="14" ry="10" transform="rotate(-15 -72 -12)" stroke="none"/>
    <ellipse cx="-80" cy="-26" rx="10" ry="7" transform="rotate(-30 -80 -26)" stroke="none"/>
    <!-- Right claw -->
    <path d="M50,0 Q64,-8 72,-12" fill="none" stroke-width="11" stroke-linecap="round"/>
    <ellipse cx="72" cy="-12" rx="14" ry="10" transform="rotate(15 72 -12)" stroke="none"/>
    <ellipse cx="80" cy="-26" rx="10" ry="7" transform="rotate(30 80 -26)" stroke="none"/>
    <!-- Legs -->
    <line x1="-40" y1="16" x2="-56" y2="32" stroke-width="5" stroke-linecap="round" fill="none"/>
    <line x1="-26" y1="22" x2="-36" y2="40" stroke-width="5" stroke-linecap="round" fill="none"/>
    <line x1="-10" y1="25" x2="-12" y2="44" stroke-width="5" stroke-linecap="round" fill="none"/>
    <line x1="40" y1="16" x2="56" y2="32" stroke-width="5" stroke-linecap="round" fill="none"/>
    <line x1="26" y1="22" x2="36" y2="40" stroke-width="5" stroke-linecap="round" fill="none"/>
    <line x1="10" y1="25" x2="12" y2="44" stroke-width="5" stroke-linecap="round" fill="none"/>
  </g>
</svg>
`;

export function ConversationPanel({ onOpenArtifact, meetingBrief }: Props) {
  const title = useAgentStore((s) => s.currentConversationTitle);
  const newConversation = useAgentStore((s) => s.newConversation);
  const isStreaming = useAgentStore((s) => s.isStreaming);
  const connected = useAgentStore((s) => s.connected);
  const sendMessage = useAgentStore((s) => s.sendMessage);
  const messages = useAgentStore((s) => s.messages);

  const briefSentRef = useRef(false);
  useEffect(() => {
    if (connected && meetingBrief && !briefSentRef.current && messages.length === 0) {
      briefSentRef.current = true;
      setTimeout(() => sendMessage(meetingBrief), 1200);
    }
  }, [connected, meetingBrief, messages.length, sendMessage]);

  return (
    <div
      className="flex-1 flex flex-col h-full min-w-[400px] scanlines relative"
      style={{ borderRight: '1px solid #32325a' }}
    >
      {/* Ferris watermark — always behind everything */}
      <div
        className="absolute inset-0 flex items-center justify-center pointer-events-none"
        style={{ zIndex: 0 }}
        aria-hidden="true"
      >
        <div
          style={{ width: 380, height: 260, opacity: 0.07 }}
          dangerouslySetInnerHTML={{ __html: FERRIS_SVG }}
        />
      </div>

      {/* Header */}
      <div
        className="flex-shrink-0 px-4 py-2 border-b border-argus-borderBright flex items-center justify-between relative"
        style={{ background: 'var(--surface-hi)', zIndex: 1 }}
      >
        <div className="flex items-center gap-3 min-w-0">
          <span className="text-[9px] font-mono tracking-widest uppercase text-argus-amber flex-shrink-0">
            CONVERSATION
          </span>
          {title && (
            <span
              className="text-[9px] font-mono truncate"
              style={{ color: '#3a3a5a', maxWidth: '260px' }}
              title={title}
            >
              {title}
            </span>
          )}
        </div>

        <button
          onClick={newConversation}
          disabled={isStreaming}
          className="flex items-center gap-1 text-[9px] font-mono px-2 py-1 rounded transition-colors flex-shrink-0"
          style={{
            border: '1px solid transparent',
            color: '#5a5a7a',
            background: 'transparent',
            opacity: isStreaming ? 0.4 : 1,
            cursor: isStreaming ? 'not-allowed' : 'pointer',
          }}
          onMouseEnter={(e) => {
            if (!isStreaming) {
              (e.currentTarget as HTMLButtonElement).style.color = '#ffb000';
              (e.currentTarget as HTMLButtonElement).style.borderColor = 'rgba(255,176,0,0.3)';
              (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,176,0,0.06)';
            }
          }}
          onMouseLeave={(e) => {
            (e.currentTarget as HTMLButtonElement).style.color = '#5a5a7a';
            (e.currentTarget as HTMLButtonElement).style.borderColor = 'transparent';
            (e.currentTarget as HTMLButtonElement).style.background = 'transparent';
          }}
          title="New conversation"
        >
          <Plus size={10} />
          NEW
        </button>
      </div>

      {/* Chat content — sits above the watermark */}
      <div className="flex-1 flex flex-col overflow-hidden relative" style={{ zIndex: 1 }}>
        <MessageList onOpenArtifact={onOpenArtifact} />
        <InputArea />
      </div>
    </div>
  );
}
