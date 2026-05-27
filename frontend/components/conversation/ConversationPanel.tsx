'use client';

import { useAgentStore } from '@/hooks/useAgentState';
import { MessageList } from './MessageList';
import { InputArea } from './InputArea';
import { ConversationHistory } from './ConversationHistory';
import { Artifact } from '@/lib/types';

interface Props {
  onOpenArtifact: (artifacts: Artifact[], index: number) => void;
}

export function ConversationPanel({ onOpenArtifact }: Props) {
  const showList = useAgentStore((s) => s.showConversationList);
  const title = useAgentStore((s) => s.currentConversationTitle);
  const toggleList = useAgentStore((s) => s.toggleConversationList);
  const newConversation = useAgentStore((s) => s.newConversation);

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
          {title && !showList && (
            <span
              className="text-[9px] font-mono truncate"
              style={{ color: '#5a5a7a', maxWidth: '200px' }}
              title={title}
            >
              {title}
            </span>
          )}
        </div>

        <div className="flex items-center gap-1 flex-shrink-0">
          {/* History toggle */}
          <button
            onClick={toggleList}
            className="text-[9px] font-mono px-2 py-0.5 rounded transition-colors"
            style={{
              background: showList ? 'rgba(255,176,0,0.15)' : 'transparent',
              border: showList ? '1px solid rgba(255,176,0,0.4)' : '1px solid transparent',
              color: showList ? '#ffb000' : '#5a5a7a',
            }}
            title="Browse past conversations"
          >
            HISTORY
          </button>

          {/* New conversation */}
          {!showList && (
            <button
              onClick={newConversation}
              className="text-[9px] font-mono px-2 py-0.5 rounded transition-colors"
              style={{
                background: 'transparent',
                border: '1px solid transparent',
                color: '#5a5a7a',
              }}
              onMouseEnter={(e) => {
                (e.target as HTMLButtonElement).style.color = '#ffb000';
                (e.target as HTMLButtonElement).style.borderColor = 'rgba(255,176,0,0.3)';
              }}
              onMouseLeave={(e) => {
                (e.target as HTMLButtonElement).style.color = '#5a5a7a';
                (e.target as HTMLButtonElement).style.borderColor = 'transparent';
              }}
              title="Start a new conversation"
            >
              + NEW
            </button>
          )}
        </div>
      </div>

      {/* Body — either conversation history list or active chat */}
      {showList ? (
        <ConversationHistory />
      ) : (
        <>
          <MessageList onOpenArtifact={onOpenArtifact} />
          <InputArea />
        </>
      )}
    </div>
  );
}
