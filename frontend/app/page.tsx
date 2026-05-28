'use client';

import { useState } from 'react';
import { AnimatePresence } from 'framer-motion';
import { Header } from '@/components/header/Header';
import { EyesPanel } from '@/components/eyes/EyesPanel';
import { ConversationPanel } from '@/components/conversation/ConversationPanel';
import { ConversationDrawer } from '@/components/conversation/ConversationDrawer';
import { MindPanel } from '@/components/mind/MindPanel';
import { ArtifactPanel } from '@/components/artifacts/ArtifactPanel';
import { Artifact } from '@/lib/types';

export default function Home() {
  const [artifactState, setArtifactState] = useState<{
    artifacts: Artifact[];
    index: number;
  } | null>(null);
  const [historyOpen, setHistoryOpen] = useState(false);

  function openArtifact(artifacts: Artifact[], index: number) {
    setArtifactState({ artifacts, index });
  }

  function closeArtifact() {
    setArtifactState(null);
  }

  return (
    <div className="flex flex-col h-screen overflow-hidden" style={{ background: '#0a0a0f' }}>
      <Header onToggleHistory={() => setHistoryOpen((v) => !v)} />

      {/* Conversation history drawer — overlays everything */}
      <ConversationDrawer open={historyOpen} onClose={() => setHistoryOpen(false)} />

      <main className="flex flex-1 overflow-hidden" style={{ paddingTop: '56px' }}>
        <EyesPanel />
        <ConversationPanel onOpenArtifact={openArtifact} />

        <AnimatePresence mode="wait">
          {artifactState ? (
            <div key="artifact" className="flex-1 overflow-hidden" style={{ minWidth: 0 }}>
              <ArtifactPanel
                artifacts={artifactState.artifacts}
                initialIndex={artifactState.index}
                onClose={closeArtifact}
              />
            </div>
          ) : (
            <MindPanel key="mind" />
          )}
        </AnimatePresence>
      </main>
    </div>
  );
}
