'use client';

import { useAgentStore } from '@/hooks/useAgentState';
import { Mission, MissionStatus } from '@/lib/types';

const STATUS_CONFIG: Record<MissionStatus, { label: string; color: string; dot: string }> = {
  planning:      { label: 'Planning',       color: '#c9a84c', dot: '◐' },
  sentry_review: { label: 'Sentry Review',  color: '#a78bfa', dot: '⬡' },
  sentry_hold:   { label: 'Sentry Hold',    color: '#f87171', dot: '⬡' },
  executing:     { label: 'Executing',      color: '#4ade80', dot: '◉' },
  verifying:     { label: 'Verifying',      color: '#67e8f9', dot: '◎' },
  complete:      { label: 'Complete',       color: '#4ade80', dot: '✦' },
  failed:        { label: 'Failed',         color: '#f87171', dot: '✕' },
};

const SUBTASK_COLORS: Record<string, string> = {
  pending:  '#3a3a48',
  running:  '#4ade80',
  complete: '#22c55e',
  failed:   '#f87171',
  skipped:  '#4a4a5a',
};

function MissionCard({ mission }: { mission: Mission }) {
  const cfg = STATUS_CONFIG[mission.status] ?? STATUS_CONFIG.planning;
  const complete = mission.subtasks.filter(s => s.status === 'complete').length;
  const total    = mission.subtasks.length;
  const pct      = total > 0 ? Math.round((complete / total) * 100) : 0;
  const idShort  = mission.id.slice(0, 8);

  return (
    <div
      className="rounded-lg overflow-hidden mb-2"
      style={{
        background: 'rgba(255,255,255,0.02)',
        border: `1px solid ${mission.status === 'executing' ? cfg.color + '44' : 'rgba(255,255,255,0.06)'}`,
        boxShadow: mission.status === 'executing' ? `0 0 12px ${cfg.color}22` : 'none',
      }}
    >
      {/* Header */}
      <div
        className="px-2.5 py-2 flex items-center justify-between"
        style={{ borderBottom: '1px solid rgba(255,255,255,0.05)' }}
      >
        <div className="flex items-center gap-1.5 min-w-0">
          <span className="text-[10px]" style={{ color: cfg.color }}>{cfg.dot}</span>
          <span
            className="text-[8px] font-mono px-1 py-px rounded uppercase tracking-wider flex-shrink-0"
            style={{ background: cfg.color + '22', color: cfg.color }}
          >
            {cfg.label}
          </span>
          <span className="text-[7px] font-mono text-[#3a3a48] flex-shrink-0">#{idShort}</span>
        </div>
        <span className="text-[7px] font-mono text-[#3a3a48] flex-shrink-0">
          {mission.primary_executor.replace('x-ai/', '').replace('google/', '').replace('anthropic/', '')}
        </span>
      </div>

      {/* Objective */}
      <div className="px-2.5 py-2">
        <div
          className="text-[9px] font-mono leading-relaxed"
          style={{ color: mission.status === 'complete' ? '#6a6a7a' : '#b0b0c8' }}
        >
          {mission.objective}
        </div>

        {/* Progress bar */}
        {total > 0 && (
          <div className="mt-2">
            <div className="flex items-center justify-between mb-1">
              <span className="text-[7px] font-mono text-[#3a3a48]">
                {complete}/{total} subtasks
              </span>
              <span className="text-[7px] font-mono" style={{ color: cfg.color }}>{pct}%</span>
            </div>
            <div className="h-0.5 rounded-full" style={{ background: 'rgba(255,255,255,0.06)' }}>
              <div
                className="h-0.5 rounded-full transition-all duration-500"
                style={{ width: `${pct}%`, background: cfg.color }}
              />
            </div>
          </div>
        )}

        {/* Subtasks */}
        {mission.subtasks.length > 0 && (
          <div className="mt-2 space-y-0.5">
            {mission.subtasks.map((sub, i) => (
              <div key={sub.id} className="flex items-center gap-1.5">
                <span
                  className="text-[7px] flex-shrink-0"
                  style={{ color: SUBTASK_COLORS[sub.status] ?? '#3a3a48' }}
                >
                  {sub.status === 'running'  ? '▶' :
                   sub.status === 'complete' ? '✓' :
                   sub.status === 'failed'   ? '✕' :
                   sub.status === 'skipped'  ? '–' : '○'}
                </span>
                <span className="text-[7px] font-mono text-[#4a4a5a] truncate">
                  {sub.description}
                </span>
              </div>
            ))}
          </div>
        )}

        {/* Deliverables */}
        {mission.deliverables.length > 0 && (
          <div className="mt-2 space-y-0.5">
            <div className="text-[7px] font-mono text-[#2a2a38] uppercase tracking-wider mb-1">
              deliverables
            </div>
            {mission.deliverables.map((d, i) => (
              <div key={i} className="flex items-center gap-1.5">
                <span
                  className="text-[7px] flex-shrink-0"
                  style={{ color: d.passed === true ? '#22c55e' : d.passed === false ? '#f87171' : '#3a3a48' }}
                >
                  {d.passed === true ? '✅' : d.passed === false ? '❌' : '○'}
                </span>
                <span className="text-[7px] font-mono text-[#3a3a48] truncate">{d.description}</span>
              </div>
            ))}
          </div>
        )}

        {/* Commit hash */}
        {mission.commit_hash && (
          <div className="mt-2 flex items-center gap-1">
            <span className="text-[7px] font-mono text-[#2a2a38]">commit</span>
            <span className="text-[7px] font-mono" style={{ color: '#4ade80' }}>
              {mission.commit_hash}
            </span>
          </div>
        )}
      </div>
    </div>
  );
}

export function MissionPanel() {
  const missions = useAgentStore((s) => s.missions);
  const active   = missions.filter(m => m.status !== 'complete' && m.status !== 'failed');
  const done     = missions.filter(m => m.status === 'complete' || m.status === 'failed');

  if (missions.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-3 px-4">
        <div className="text-[28px] opacity-20">⬡</div>
        <div className="text-[9px] font-mono text-[#2a2a38] text-center leading-relaxed">
          No missions active.<br />
          Tell me to <span style={{ color: '#4ade80' }}>start_mission</span> with an objective<br />
          and typed deliverables.
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="flex-1 overflow-y-auto px-2 py-2 space-y-3">
        {active.length > 0 && (
          <div>
            <div className="text-[7px] font-mono text-[#2a2a38] uppercase tracking-[2px] mb-2 px-0.5">
              active ({active.length})
            </div>
            {active.map(m => <MissionCard key={m.id} mission={m} />)}
          </div>
        )}
        {done.length > 0 && (
          <div>
            <div className="text-[7px] font-mono text-[#2a2a38] uppercase tracking-[2px] mb-2 px-0.5">
              completed
            </div>
            {done.slice(0, 5).map(m => <MissionCard key={m.id} mission={m} />)}
          </div>
        )}
      </div>
    </div>
  );
}
