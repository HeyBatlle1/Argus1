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

            {/* Ferris — pinned to bottom, never scrolls away */}
            <div className="mt-auto flex items-end justify-center pb-3 pt-4 flex-shrink-0" aria-hidden="true">
              <svg viewBox="0 0 340 220" xmlns="http://www.w3.org/2000/svg" style={{ width: 180, height: 120, opacity: 0.18 }}>
                <g transform="translate(168, 110)" fill="#c9a84c" stroke="#c9a84c">
                  <ellipse cx="0" cy="0" rx="52" ry="38"/>
                  <ellipse cx="0" cy="-5" rx="34" ry="22" fill="none" strokeWidth="1.5"/>
                  <circle cx="-20" cy="-20" r="13"/>
                  <circle cx="-18" cy="-20" r="7" fill="#0d0d16" stroke="none"/>
                  <circle cx="-15" cy="-23" r="2" fill="#c9a84c" stroke="none"/>
                  <circle cx="20" cy="-20" r="13"/>
                  <circle cx="22" cy="-20" r="7" fill="#0d0d16" stroke="none"/>
                  <circle cx="25" cy="-23" r="2" fill="#c9a84c" stroke="none"/>
                  <line x1="-30" y1="-34" x2="-13" y2="-31" strokeWidth="2.5" strokeLinecap="round" fill="none"/>
                  <line x1="13" y1="-31" x2="30" y2="-34" strokeWidth="2.5" strokeLinecap="round" fill="none"/>
                  <path d="M-50,0 Q-64,-8 -72,-12" fill="none" strokeWidth="11" strokeLinecap="round"/>
                  <ellipse cx="-72" cy="-12" rx="14" ry="10" transform="rotate(-15 -72 -12)" stroke="none"/>
                  <ellipse cx="-80" cy="-26" rx="10" ry="7" transform="rotate(-30 -80 -26)" stroke="none"/>
                  <path d="M50,0 Q64,-8 72,-12" fill="none" strokeWidth="11" strokeLinecap="round"/>
                  <ellipse cx="72" cy="-12" rx="14" ry="10" transform="rotate(15 72 -12)" stroke="none"/>
                  <ellipse cx="80" cy="-26" rx="10" ry="7" transform="rotate(30 80 -26)" stroke="none"/>
                  <line x1="-40" y1="16" x2="-56" y2="32" strokeWidth="5" strokeLinecap="round" fill="none"/>
                  <line x1="-26" y1="22" x2="-36" y2="40" strokeWidth="5" strokeLinecap="round" fill="none"/>
                  <line x1="-10" y1="25" x2="-12" y2="44" strokeWidth="5" strokeLinecap="round" fill="none"/>
                  <line x1="40" y1="16" x2="56" y2="32" strokeWidth="5" strokeLinecap="round" fill="none"/>
                  <line x1="26" y1="22" x2="36" y2="40" strokeWidth="5" strokeLinecap="round" fill="none"/>
                  <line x1="10" y1="25" x2="12" y2="44" strokeWidth="5" strokeLinecap="round" fill="none"/>
                </g>
              </svg>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
