'use client';

import { useAgentStore } from '@/hooks/useAgentState';
import { MessageList } from './MessageList';
import { InputArea } from './InputArea';
import { Artifact } from '@/lib/types';
import { Plus } from 'lucide-react';

interface Props {
  onOpenArtifact: (artifacts: Artifact[], index: number) => void;
}

export function ConversationPanel({ onOpenArtifact }: Props) {
  const title = useAgentStore((s) => s.currentConversationTitle);
  const newConversation = useAgentStore((s) => s.newConversation);
  const isStreaming = useAgentStore((s) => s.isStreaming);

  return (
    <div
      className="flex-1 flex flex-col h-full min-w-[400px] scanlines"
      style={{ borderRight: '1px solid #32325a' }}
    >
      {/* Header */}
      <div
        className="flex-shrink-0 px-4 py-2 border-b border-argus-borderBright flex items-center justify-between"
        style={{ background: 'var(--surface-hi)' }}
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

      <MessageList onOpenArtifact={onOpenArtifact} />
      <InputArea />
    </div>
  );
}
