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
import { EYE_LABELS } from '@/lib/constants';
import { MODEL_CONFIG } from '@/lib/models';

export function EyesPanel() {
  const [collapsed, setCollapsed] = useState(false);
  const eyeState = useAgentStore((s) => s.eyeState);
  const activeModel = useAgentStore((s) => s.activeModel);
  const tools = useAgentStore((s) => s.tools);
  const startTime = useAgentStore((s) => s.startTime);
  const vaultKeys = useAgentStore((s) => s.vaultKeys);
  const mcpServers = useAgentStore((s) => s.mcpServers);
  const connected = useAgentStore((s) => s.connected);

  const modelCfg = MODEL_CONFIG[activeModel];

  return (
    <motion.div
      animate={{ width: collapsed ? 32 : 240 }}
      transition={{ duration: 0.2, ease: 'easeInOut' }}
      className="relative h-full border-r border-argus-border flex-shrink-0 overflow-hidden"
      style={{ background: '#0d0d16' }}
    >
      {/* Collapsed strip */}
      <AnimatePresence>
        {collapsed && (
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
        {!collapsed && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 flex flex-col"
          >
            {/* Panel header */}
            <div className="flex items-center justify-between px-3 py-2 border-b border-argus-borderBright flex-shrink-0" style={{ background: 'var(--surface-hi)' }}>
              <span className="text-[9px] font-mono tracking-widest uppercase text-argus-amber">
                THE EYES
              </span>
              <button
                onClick={() => setCollapsed(true)}
                className="text-argus-textDim hover:text-argus-amber transition-colors"
              >
                <ChevronLeft size={14} />
              </button>
            </div>

            {/* Scrollable sections */}
            <div className="flex-1 overflow-y-auto min-h-0">
              {/* Agent Status */}
              <CollapsibleSection title="Agent Status" defaultOpen={true}>
                <div className="space-y-2">
                  <div className="flex items-center gap-2">
                    <EyeStateIndicator state={eyeState} size="sm" showLabel={true} />
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-[9px] font-mono tracking-widest uppercase text-argus-textDim">MODEL</span>
                    <span className="text-[11px] font-mono text-argus-amber">{modelCfg.name}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-[9px] font-mono tracking-widest uppercase text-argus-textDim">TIER</span>
                    <span className="text-[11px] font-mono" style={{ color: modelCfg.color }}>
                      {modelCfg.icon} {modelCfg.tierLabel}
                    </span>
                  </div>
                </div>
              </CollapsibleSection>

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
                <VaultStatus
                  locked={!connected}
                  keys={vaultKeys}
                />
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
            </div>

            {/* Ferris — cell door, pinned below all sections */}
            <div
              className="flex-shrink-0 mx-2 mb-2 mt-3"
              aria-hidden="true"
              style={{
                border: '2px solid #c9a84c',
                borderRadius: '6px',
                background: '#06060e',
                overflow: 'hidden',
                boxShadow: '0 0 12px rgba(201,168,76,0.15), inset 0 0 20px rgba(0,0,0,0.6)',
              }}
            >
              {/* Cell header */}
              <div style={{
                borderBottom: '1px solid rgba(201,168,76,0.35)',
                padding: '4px 8px',
                background: 'rgba(201,168,76,0.07)',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
              }}>
                <span style={{ fontFamily: 'monospace', fontSize: '8px', letterSpacing: '0.18em', color: '#c9a84c', textTransform: 'uppercase' }}>Secure Hold</span>
                <span style={{ fontFamily: 'monospace', fontSize: '8px', letterSpacing: '0.1em', color: '#4a4a2a' }}>CELL-001</span>
              </div>

              {/* Cell interior */}
              <svg
                viewBox="0 0 220 148"
                xmlns="http://www.w3.org/2000/svg"
                style={{ width: '100%', display: 'block' }}
              >
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

                {/* Floor glow under Ferris */}
                <ellipse cx="110" cy="141" rx="52" ry="7" fill="url(#cellFloor)"/>

                {/* Ferris — real Rust orange, behind bars */}
                <g transform="translate(110, 100)">
                  {/* Gentle bob animation — peering through the bars */}
                  <animateTransform
                    attributeName="transform"
                    type="translate"
                    values="110,100; 110,97; 110,100; 110,102; 110,100"
                    dur="4s"
                    repeatCount="indefinite"
                    calcMode="spline"
                    keySplines="0.45 0 0.55 1; 0.45 0 0.55 1; 0.45 0 0.55 1; 0.45 0 0.55 1"
                  />

                  {/* Body */}
                  <ellipse cx="0" cy="0" rx="46" ry="33" fill="#ce4118"/>
                  {/* Belly — lighter center */}
                  <ellipse cx="0" cy="5" rx="28" ry="18" fill="#d9561e" opacity="0.55"/>

                  {/* Left eye — pupil shifted UP (looking at bars worriedly) */}
                  <circle cx="-17" cy="-17" r="11" fill="#f0ebe0"/>
                  <circle cx="-16" cy="-21" r="6" fill="#1a0a06"/>
                  <circle cx="-14" cy="-23" r="1.8" fill="white"/>
                  {/* Left brow — furrowed */}
                  <line x1="-24" y1="-26" x2="-10" y2="-29" stroke="#8a2b10" strokeWidth="2" strokeLinecap="round"/>

                  {/* Right eye — pupil shifted UP */}
                  <circle cx="17" cy="-17" r="11" fill="#f0ebe0"/>
                  <circle cx="18" cy="-21" r="6" fill="#1a0a06"/>
                  <circle cx="20" cy="-23" r="1.8" fill="white"/>
                  {/* Right brow — furrowed */}
                  <line x1="10" y1="-29" x2="24" y2="-26" stroke="#8a2b10" strokeWidth="2" strokeLinecap="round"/>

                  {/* Antennae — bent inward (anxious) */}
                  <path d="M-26,-26 Q-18,-36 -6,-32" fill="none" stroke="#8a2b10" strokeWidth="2.5" strokeLinecap="round"/>
                  <path d="M6,-32 Q18,-36 26,-26" fill="none" stroke="#8a2b10" strokeWidth="2.5" strokeLinecap="round"/>

                  {/* Left arm reaching UP, gripping horizontal rail */}
                  <path d="M-42,-4 Q-60,-20 -56,-40" fill="none" stroke="#ce4118" strokeWidth="10" strokeLinecap="round"/>
                  <ellipse cx="-56" cy="-40" rx="12" ry="8" fill="#ce4118" transform="rotate(-25,-56,-40)"/>
                  <ellipse cx="-63" cy="-50" rx="9" ry="6" fill="#bf3510" transform="rotate(-40,-63,-50)"/>

                  {/* Right arm reaching UP, gripping horizontal rail */}
                  <path d="M42,-4 Q60,-20 56,-40" fill="none" stroke="#ce4118" strokeWidth="10" strokeLinecap="round"/>
                  <ellipse cx="56" cy="-40" rx="12" ry="8" fill="#ce4118" transform="rotate(25,56,-40)"/>
                  <ellipse cx="63" cy="-50" rx="9" ry="6" fill="#bf3510" transform="rotate(40,63,-50)"/>

                  {/* Legs (partially visible at bottom) */}
                  <line x1="-36" y1="16" x2="-50" y2="30" stroke="#b83810" strokeWidth="4.5" strokeLinecap="round"/>
                  <line x1="-22" y1="21" x2="-30" y2="36" stroke="#b83810" strokeWidth="4.5" strokeLinecap="round"/>
                  <line x1="-8" y1="24" x2="-9" y2="39" stroke="#b83810" strokeWidth="4.5" strokeLinecap="round"/>
                  <line x1="36" y1="16" x2="50" y2="30" stroke="#b83810" strokeWidth="4.5" strokeLinecap="round"/>
                  <line x1="22" y1="21" x2="30" y2="36" stroke="#b83810" strokeWidth="4.5" strokeLinecap="round"/>
                  <line x1="8" y1="24" x2="9" y2="39" stroke="#b83810" strokeWidth="4.5" strokeLinecap="round"/>
                </g>

                {/* Bars — drawn on top of Ferris so he's truly behind them */}
                <g filter="url(#barGlowCell)">
                  <rect x="16"  y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                  <rect x="46"  y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                  <rect x="76"  y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                  <rect x="106" y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                  <rect x="136" y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                  <rect x="166" y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                  <rect x="196" y="0" width="5" height="148" fill="#c9a84c" opacity="0.72"/>
                </g>

                {/* Horizontal rails — top cross-brace + the grab rail Ferris grips */}
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
