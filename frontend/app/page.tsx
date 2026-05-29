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
  'MONTHLY MEETING — INTERNAL HEALTH CHECK: You are Argus (Claude Sonnet), the mission coordinator. Take a breath before you begin — this is not timed, accuracy matters more than speed. Conduct an honest internal systems assessment: (1) Review your current skill library and identify the 3 most-used capabilities, (2) Check memory health — are there gaps in what you recall about recent work?, (3) Identify one area where your tool use could be more efficient. Be direct and honest — do not perform health, report it. Post your findings to Discord when complete.';

const MEETING_BRIEFS: Record<2 | 3 | 4, { model: ModelId; brief: string }> = {
  2: {
    model: 'grok-build',
    brief:
      'MONTHLY MEETING — AI LANDSCAPE INTEL: You are Grok, Argus\'s AI landscape analyst. Take a breath before you begin — this is not a race, thorough and honest research is the goal. Research and report on: (1) The 3 most significant AI model releases or capability jumps in the past 30 days, (2) Any major safety incidents, alignment research, or policy developments, (3) One "signal to watch" — an emerging trend that isn\'t mainstream yet. If your search turns up nothing significant, say so plainly. No inflation. Post your honest findings to Discord when done.',
  },
  3: {
    model: 'gemini-flash',
    brief:
      'MONTHLY MEETING — TECH & INFRA TRENDS: You are Gemini, Argus\'s tech and infrastructure analyst. Take a breath before you begin — thoroughness beats speed here. Research and report on: (1) Notable developments in developer tooling, cloud infra, or open-source this month, (2) Any security vulnerabilities or supply chain issues worth watching, (3) One tool or library gaining real momentum and why it matters. Be specific and honest — if a trend is overhyped, say so. Post your findings to Discord when done.',
  },
  4: {
    model: 'claude-opus',
    brief:
      'MONTHLY MEETING — STRATEGIC SYNTHESIS: You are Opus, Argus\'s strategic reasoning engine. Take a breath and read carefully — your job is synthesis, not speed. Review the briefings from Grok (AI landscape) and Gemini (tech trends) just posted to Discord, then give an honest synthesis: (1) The single most important thing Argus should pay attention to this month and why, (2) Any real cross-signal connections between the two reports — do not manufacture connections that aren\'t there, (3) A recommended action or focus area for the next 30 days. Be direct. If the briefings don\'t surface anything urgent, say so. Post to Discord when complete.',
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
