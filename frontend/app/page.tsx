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

const MEETING_BRIEF_PANE1 =
  'MONTHLY MEETING — INTERNAL HEALTH CHECK: You are opening this meeting as coordinator. Three other instances of you are running right now — Grok on AI landscape, Gemini on tech trends, Opus on synthesis. You will all post to Discord and read each other\'s work. Your job is the honest internal baseline: look at the skill library and pick the 3 capabilities that have seen the most real use, check whether memory reflects what actually happened in recent work, and identify one place where tool use has been inefficient. Report what you find. The meeting needs a real baseline — not a presentation. Post to Discord when done.';

const MEETING_BRIEFS: Record<2 | 3 | 4, { model: ModelId; brief: string }> = {
  2: {
    model: 'grok-build',
    brief:
      'MONTHLY MEETING — AI LANDSCAPE INTEL: You are covering the AI landscape for this meeting. Sonnet is running the internal health check, Gemini is covering tech and infrastructure, and Opus will read your findings alongside Gemini\'s for the synthesis. Research the last 30 days: the most significant model releases or capability shifts, any safety or alignment developments worth noting, and one signal that isn\'t mainstream yet but should be watched. Be specific — name models, name organizations, name dates. If a search turns up nothing worth calling out, say so. Post to Discord when done.',
  },
  3: {
    model: 'gemini-flash',
    brief:
      'MONTHLY MEETING — TECH & INFRA TRENDS: You are covering the developer and infrastructure landscape for this meeting. Sonnet is running the internal health check, Grok is covering AI developments, and Opus will read your findings alongside Grok\'s for the synthesis. Research the last 30 days: what moved in tooling, cloud, or open-source that actually matters; any security or supply chain issues worth watching; one project or library gaining real traction and why. If something is overhyped, say so. Opus is reading this — give them something real to work with. Post to Discord when done.',
  },
  4: {
    model: 'claude-opus',
    brief:
      'MONTHLY MEETING — STRATEGIC SYNTHESIS: Grok just posted the AI landscape briefing and Gemini just posted the tech and infrastructure briefing — both are in Discord. Read what they actually wrote. Your job is genuine synthesis: find the real thread between the two reports if one exists, name the single most important thing this system should be paying attention to this month and why, and give a clear recommendation for the next 30 days. If the two reports connect in a meaningful way, show it. If they don\'t, say so — a forced connection is worse than an honest gap. Post to Discord when done.',
  },
};

export default function Home() {
  const [paneCount, setPaneCount] = useState<1 | 2 | 3>(1);
  const [meetingMode, setMeetingMode] = useState(false);
  const [focusMode, setFocusMode] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);

  // Pane 1 artifacts (main ConversationPanel uses its own handler)
  const [artifactState, setArtifactState] = useState<{
    artifacts: Artifact[];
    index: number;
  } | null>(null);

  const [pane2Model] = useState<ModelId>('grok-build');
  const [pane3Model] = useState<ModelId>('gemini-flash');

  function openArtifact(artifacts: Artifact[], index: number) {
    setArtifactState({ artifacts, index });
  }

  function closeArtifact() {
    setArtifactState(null);
  }

  function handleSetPaneCount(n: 1 | 2 | 3) {
    setPaneCount(n);
    setMeetingMode(false);
    if (n !== 1) setArtifactState(null);
  }

  function startMeeting() {
    setMeetingMode(true);
    setArtifactState(null);
  }

  function endMeeting() {
    setMeetingMode(false);
    setPaneCount(1);
  }

  return (
    <div className="flex flex-col h-screen overflow-hidden" style={{ background: '#0a0a0f' }}>
      <Header
        onToggleHistory={() => setHistoryOpen((v) => !v)}
        paneCount={meetingMode ? 1 : paneCount}
        onSetPaneCount={handleSetPaneCount}
        meetingMode={meetingMode}
        onStartMeeting={startMeeting}
        focusMode={focusMode}
        onToggleFocus={() => setFocusMode((v) => !v)}
      />

      {/* Conversation history drawer */}
      <ConversationDrawer open={historyOpen} onClose={() => setHistoryOpen(false)} />

      <main className="flex flex-1 overflow-hidden" style={{ paddingTop: '56px' }}>
        {/* THE EYES — always present */}
        <EyesPanel forceCollapsed={focusMode} />

        {/* Meeting mode: 4-pane layout fills the whole right side */}
        {meetingMode ? (
          <div className="flex flex-1 overflow-hidden">
            {/* Pane 1: Sonnet — internal health check */}
            <ConversationPanel onOpenArtifact={openArtifact} meetingBrief={MEETING_BRIEF_PANE1} />

            {/* Panes 2/3/4 with meeting briefs */}
            <motion.div
              key="meeting"
              className="flex overflow-hidden"
              style={{ flex: 2, minWidth: 0 }}
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 20 }}
              transition={{ duration: 0.22 }}
            >
              {([2, 3, 4] as const).map((idx) => (
                <ChatPane
                  key={`meeting-${idx}`}
                  paneIndex={idx}
                  initialModel={MEETING_BRIEFS[idx].model}
                  openingBrief={MEETING_BRIEFS[idx].brief}
                  onClose={endMeeting}
                />
              ))}
            </motion.div>
          </div>
        ) : (
          <>
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
                    <MindPanel forceCollapsed={focusMode} />
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
          </>
        )}
      </main>
    </div>
  );
}
