'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronLeft } from 'lucide-react';
import { useAgentStore } from '@/hooks/useAgentState';
import { CollapsibleSection } from '@/components/shared/CollapsibleSection';
import { EyeStateIndicator } from '@/components/shared/EyeStateIndicator';
import { ToolStatus } from './ToolStatus';
import { VaultStatus } from './VaultStatus';
import { SystemInfo } from './SystemInfo';
import { NexusCore } from './NexusCore';
import { MODEL_CONFIG } from '@/lib/models';
import { ModelId } from '@/lib/types';

interface Props {
  forceCollapsed?: boolean;
}

const INSTANCES: { model: ModelId; short: string }[] = [
  { model: 'claude-haiku', short: 'HAIKU' },
  { model: 'grok-build',  short: 'GROK'  },
  { model: 'gemini-flash', short: 'GEMINI' },
  { model: 'claude-opus', short: 'OPUS'  },
];

export function EyesPanel({ forceCollapsed = false }: Props) {
  const [collapsed, setCollapsed] = useState(false);
  const isCollapsed = forceCollapsed || collapsed;

  const eyeState   = useAgentStore((s) => s.eyeState);
  const activeModel = useAgentStore((s) => s.activeModel);
  const corePulse  = useAgentStore((s) => s.corePulse);
  const tools      = useAgentStore((s) => s.tools);
  const vaultKeys  = useAgentStore((s) => s.vaultKeys);
  const mcpServers = useAgentStore((s) => s.mcpServers);
  const connected  = useAgentStore((s) => s.connected);
  const activity   = useAgentStore((s) => s.activity);
  const switchModel = useAgentStore((s) => s.switchModel);

  const modelCfg = MODEL_CONFIG[activeModel];

  // Last 3 activity entries used as discourse ticker
  const discourse = activity.slice(0, 3);

  return (
    <motion.div
      animate={{ width: isCollapsed ? 32 : 248 }}
      transition={{ duration: 0.2, ease: 'easeInOut' }}
      className="relative h-full border-r border-argus-border flex-shrink-0 overflow-hidden"
      style={{ background: '#070710' }}
    >
      {/* Collapsed strip */}
      <AnimatePresence>
        {isCollapsed && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 flex flex-col items-center pt-4 gap-3"
          >
            <button
              onClick={() => setCollapsed(false)}
              className="text-argus-textDim hover:text-argus-amber transition-colors"
            >
              <ChevronLeft size={14} className="rotate-180" />
            </button>
            <EyeStateIndicator state={eyeState} size="sm" />
          </motion.div>
        )}
      </AnimatePresence>

      {/* Full panel */}
      <AnimatePresence>
        {!isCollapsed && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 flex flex-col"
          >
            {/* Panel header */}
            <div className="flex items-center justify-between px-3 py-2 border-b border-argus-border flex-shrink-0" style={{ background: 'rgba(255,255,255,0.02)' }}>
              <span className="text-[9px] font-mono tracking-widest uppercase" style={{ color: '#67f6ff' }}>
                THE EYES
              </span>
              <button
                onClick={() => setCollapsed(true)}
                className="text-argus-textDim hover:text-argus-amber transition-colors"
              >
                <ChevronLeft size={14} />
              </button>
            </div>

            {/* Scrollable content */}
            <div className="flex-1 overflow-y-auto min-h-0">

              {/* NexusCore canvas */}
              <div className="flex justify-center pt-4 pb-2">
                <NexusCore eyeState={eyeState} pulse={corePulse} size={188} />
              </div>

              {/* Instance Constellation */}
              <div className="px-3 pb-3">
                <div className="text-[8px] font-mono tracking-[2px] uppercase mb-1.5" style={{ color: 'rgba(103,246,255,0.4)' }}>
                  Active Instances
                </div>
                <div className="grid grid-cols-2 gap-1">
                  {INSTANCES.map(({ model, short }) => {
                    const cfg = MODEL_CONFIG[model];
                    const isActive = activeModel === model;
                    return (
                      <button
                        key={model}
                        onClick={() => switchModel(model)}
                        className="instance-card text-left px-2 py-1.5 rounded-lg border"
                        style={{
                          background: isActive ? `${cfg.color}10` : 'rgba(255,255,255,0.025)',
                          borderColor: isActive ? cfg.color : 'rgba(255,255,255,0.07)',
                        }}
                      >
                        <div className="text-[9px] font-mono truncate" style={{ color: isActive ? cfg.color : '#5a5a68' }}>
                          {short}
                        </div>
                        <div className="text-[8px] text-[#3a3a48] mt-px">
                          {isActive ? 'active' : 'standby'}
                        </div>
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Active Tools */}
              <CollapsibleSection title="Active Tools" defaultOpen={true} count={tools.length}>
                <div className="divide-y divide-argus-border/30">
                  {tools.map((tool) => (
                    <ToolStatus key={tool.name} tool={tool} />
                  ))}
                </div>
              </CollapsibleSection>

              {/* Vault */}
              <CollapsibleSection title="Vault" defaultOpen={true}>
                <VaultStatus locked={!connected} keys={vaultKeys} />
              </CollapsibleSection>

              {/* MCP Servers */}
              <CollapsibleSection title="MCP Servers" defaultOpen={false} count={mcpServers.length}>
                {mcpServers.length === 0 ? (
                  <p className="text-[10px] font-mono text-argus-textDim">No servers connected.</p>
                ) : (
                  <div className="space-y-1">
                    {mcpServers.map((name) => (
                      <div key={name} className="flex items-center gap-1.5">
                        <span className="w-1.5 h-1.5 rounded-full bg-argus-greenLight flex-shrink-0" />
                        <span className="text-[10px] font-mono text-argus-textDim truncate">{name}</span>
                      </div>
                    ))}
                  </div>
                )}
              </CollapsibleSection>

              {/* System */}
              <CollapsibleSection title="System" defaultOpen={false}>
                <SystemInfo />
              </CollapsibleSection>

              {/* Discourse ticker — live activity as cross-instance chatter */}
              {discourse.length > 0 && (
                <CollapsibleSection title="Intranet" defaultOpen={true}>
                  <div className="space-y-1.5">
                    {discourse.map((entry) => (
                      <div key={entry.id} className="text-[9px] leading-tight">
                        <span className="font-mono" style={{ color: '#c084fc' }}>
                          {modelCfg.name}:
                        </span>
                        <span className="text-[#5a5a68] ml-1">
                          {entry.kind === 'tool' ? `ran ${entry.label}` : entry.label}
                        </span>
                      </div>
                    ))}
                  </div>
                </CollapsibleSection>
              )}
            </div>

            {/* Ferris — secure hold, always pinned at bottom */}
            <div
              className="flex-shrink-0 mx-2 mb-2 mt-2"
              aria-hidden="true"
              style={{
                border: '2px solid #c9a84c',
                borderRadius: '6px',
                background: '#06060e',
                overflow: 'hidden',
                boxShadow: '0 0 12px rgba(201,168,76,0.15), inset 0 0 20px rgba(0,0,0,0.6)',
              }}
            >
              <div style={{ borderBottom: '1px solid rgba(201,168,76,0.35)', padding: '4px 8px', background: 'rgba(201,168,76,0.07)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontFamily: 'monospace', fontSize: '8px', letterSpacing: '0.18em', color: '#c9a84c', textTransform: 'uppercase' }}>Secure Hold</span>
                <span style={{ fontFamily: 'monospace', fontSize: '8px', letterSpacing: '0.1em', color: '#4a4a2a' }}>CELL-001</span>
              </div>
              <svg viewBox="0 0 220 148" xmlns="http://www.w3.org/2000/svg" style={{ width: '100%', display: 'block' }}>
                <defs>
                  <filter id="barGlowCell" x="-60%" y="-10%" width="220%" height="120%">
                    <feGaussianBlur stdDeviation="2" result="blur"/>
                    <feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge>
                  </filter>
                  <radialGradient id="cellFloor" cx="50%" cy="100%" r="50%">
                    <stop offset="0%" stopColor="#1a1a10" stopOpacity="0.8"/>
                    <stop offset="100%" stopColor="#06060e" stopOpacity="0"/>
                  </radialGradient>
                </defs>
                <ellipse cx="110" cy="141" rx="52" ry="7" fill="url(#cellFloor)"/>
                <g transform="translate(110, 100)">
                  <animateTransform attributeName="transform" type="translate" values="110,100; 110,97; 110,100; 110,102; 110,100" dur="4s" repeatCount="indefinite" calcMode="spline" keySplines="0.45 0 0.55 1; 0.45 0 0.55 1; 0.45 0 0.55 1; 0.45 0 0.55 1"/>
                  <ellipse cx="0" cy="0" rx="46" ry="33" fill="#ce4118"/>
                  <ellipse cx="0" cy="5" rx="28" ry="18" fill="#d9561e" opacity="0.55"/>
                  <circle cx="-17" cy="-17" r="11" fill="#f0ebe0"/>
                  <circle cx="-16" cy="-21" r="6" fill="#1a0a06"/>
                  <circle cx="-14" cy="-23" r="1.8" fill="white"/>
                  <line x1="-24" y1="-26" x2="-10" y2="-29" stroke="#8a2b10" strokeWidth="2" strokeLinecap="round"/>
                  <circle cx="17" cy="-17" r="11" fill="#f0ebe0"/>
                  <circle cx="18" cy="-21" r="6" fill="#1a0a06"/>
                  <circle cx="20" cy="-23" r="1.8" fill="white"/>
                  <line x1="10" y1="-29" x2="24" y2="-26" stroke="#8a2b10" strokeWidth="2" strokeLinecap="round"/>
                  <path d="M-26,-26 Q-18,-36 -6,-32" fill="none" stroke="#8a2b10" strokeWidth="2.5" strokeLinecap="round"/>
                  <path d="M6,-32 Q18,-36 26,-26" fill="none" stroke="#8a2b10" strokeWidth="2.5" strokeLinecap="round"/>
                  <path d="M-42,-4 Q-60,-20 -56,-40" fill="none" stroke="#ce4118" strokeWidth="10" strokeLinecap="round"/>
                  <ellipse cx="-56" cy="-40" rx="12" ry="8" fill="#ce4118" transform="rotate(-25,-56,-40)"/>
                  <ellipse cx="-63" cy="-50" rx="9" ry="6" fill="#bf3510" transform="rotate(-40,-63,-50)"/>
                  <path d="M42,-4 Q60,-20 56,-40" fill="none" stroke="#ce4118" strokeWidth="10" strokeLinecap="round"/>
                  <ellipse cx="56" cy="-40" rx="12" ry="8" fill="#ce4118" transform="rotate(25,56,-40)"/>
                  <ellipse cx="63" cy="-50" rx="9" ry="6" fill="#bf3510" transform="rotate(40,63,-50)"/>
                  <line x1="-36" y1="16" x2="-50" y2="30" stroke="#b83810" strokeWidth="4.5" strokeLinecap="round"/>
                  <line x1="-22" y1="21" x2="-30" y2="36" stroke="#b83810" strokeWidth="4.5" strokeLinecap="round"/>
                  <line x1="-8" y1="24" x2="-9" y2="39" stroke="#b83810" strokeWidth="4.5" strokeLinecap="round"/>
                  <line x1="36" y1="16" x2="50" y2="30" stroke="#b83810" strokeWidth="4.5" strokeLinecap="round"/>
                  <line x1="22" y1="21" x2="30" y2="36" stroke="#b83810" strokeWidth="4.5" strokeLinecap="round"/>
                  <line x1="8" y1="24" x2="9" y2="39" stroke="#b83810" strokeWidth="4.5" strokeLinecap="round"/>
                </g>
                <g filter="url(#barGlowCell)">
                  <rect x="16"  y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                  <rect x="46"  y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                  <rect x="76"  y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                  <rect x="106" y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                  <rect x="136" y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                  <rect x="166" y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                  <rect x="196" y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                </g>
                <rect x="0" y="0"  width="220" height="5" fill="#c9a84c" opacity="0.55" filter="url(#barGlowCell)"/>
                <rect x="0" y="58" width="220" height="5" fill="#c9a84c" opacity="0.6"  filter="url(#barGlowCell)"/>
                <rect x="0" y="143" width="220" height="5" fill="#c9a84c" opacity="0.5" filter="url(#barGlowCell)"/>
              </svg>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
