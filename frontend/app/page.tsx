'use client';

import { useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { Header } from '@/components/header/Header';
import { EyesPanel } from '@/components/eyes/EyesPanel';
import { ConversationPanel } from '@/components/conversation/ConversationPanel';
import { ConversationDrawer } from '@/components/conversation/ConversationDrawer';
import { MindPanel } from '@/components/mind/MindPanel';
import { ArtifactPanel } from '@/components/artifacts/ArtifactPanel';
import { ChatPane } from '@/components/panes/ChatPane';
import { Artifact, ModelId } from '@/lib/types';

export default function Home() {
  const [paneCount, setPaneCount] = useState<1 | 2 | 3>(1);
  const [historyOpen, setHistoryOpen] = useState(false);

  // Pane 1 artifacts (main ConversationPanel uses its own handler)
  const [artifactState, setArtifactState] = useState<{
    artifacts: Artifact[];
    index: number;
  } | null>(null);

  // Each extra pane can have a different starting model
  const [pane2Model] = useState<ModelId>('grok-fast');
  const [pane3Model] = useState<ModelId>('gemini-flash');

  function openArtifact(artifacts: Artifact[], index: number) {
    setArtifactState({ artifacts, index });
  }

  function closeArtifact() {
    setArtifactState(null);
  }

  function handleSetPaneCount(n: 1 | 2 | 3) {
    setPaneCount(n);
    // Close artifact panel when switching layouts
    if (n !== 1) setArtifactState(null);
  }

  return (
    <div className="flex flex-col h-screen overflow-hidden" style={{ background: '#0a0a0f' }}>
      <Header
        onToggleHistory={() => setHistoryOpen((v) => !v)}
        paneCount={paneCount}
        onSetPaneCount={handleSetPaneCount}
      />

      {/* Conversation history drawer */}
      <ConversationDrawer open={historyOpen} onClose={() => setHistoryOpen(false)} />

      <main className="flex flex-1 overflow-hidden" style={{ paddingTop: '56px' }}>
        {/* THE EYES — always present */}
        <EyesPanel />

        {/* Pane 1 — always present */}
        <ConversationPanel onOpenArtifact={openArtifact} />

        {/* Right side — depends on pane count */}
        <AnimatePresence mode="wait">
          {paneCount === 1 && (
            <motion.div
              key="single"
              className="flex flex-1 overflow-hidden"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.18 }}
            >
              {artifactState ? (
                <div className="flex-1 overflow-hidden" style={{ minWidth: 0 }}>
                  <ArtifactPanel
                    artifacts={artifactState.artifacts}
                    initialIndex={artifactState.index}
                    onClose={closeArtifact}
                  />
                </div>
              ) : (
                <MindPanel />
              )}
            </motion.div>
          )}

          {paneCount === 2 && (
            <motion.div
              key="dual"
              className="flex overflow-hidden"
              style={{ flex: 1, minWidth: 0 }}
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 20 }}
              transition={{ duration: 0.2 }}
            >
              <ChatPane
                paneIndex={2}
                initialModel={pane2Model}
                onClose={() => handleSetPaneCount(1)}
              />
            </motion.div>
          )}

          {paneCount === 3 && (
            <motion.div
              key="triple"
              className="flex overflow-hidden"
              style={{ flex: 1, minWidth: 0 }}
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 20 }}
              transition={{ duration: 0.2 }}
            >
              <ChatPane
                paneIndex={2}
                initialModel={pane2Model}
                onClose={() => handleSetPaneCount(2)}
              />
              <ChatPane
                paneIndex={3}
                initialModel={pane3Model}
                onClose={() => handleSetPaneCount(2)}
              />
            </motion.div>
          )}
        </AnimatePresence>
      </main>
    </div>
  );
}
