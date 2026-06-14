'use client';

import dynamic from 'next/dynamic';
import { useAgentStore } from '@/hooks/useAgentState';

const ForceGraph2D = dynamic(() => import('react-force-graph-2d'), { ssr: false });

export function SemanticField() {
  const memories = useAgentStore((s) => s.memories);
  const skills = useAgentStore((s) => s.skills);
  const sendMessage = useAgentStore((s) => s.sendMessage);

  const graphData = {
    nodes: [
      ...memories.map((m, i) => ({
        id: 'mem' + i,
        name: m.content.length > 44 ? m.content.slice(0, 44) + '…' : m.content,
        group: 1,
        val: (m.importance || 5) * 1.6,
      })),
      ...skills.map((s, i) => ({
        id: 'sk' + i,
        name: s.name,
        group: 2,
        val: ((s.useCount || 1)) * 2.5,
      })),
    ],
    links: memories.slice(0, -1).map((_, i) => ({
      source: 'mem' + i,
      target: 'mem' + (i + 1),
    })),
  };

  if (graphData.nodes.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center px-4">
        <div className="text-2xl mb-2 opacity-20">✧</div>
        <div className="font-mono text-[10px] text-[#3a3a48] tracking-[1px]">
          THE FIELD IS EMPTY
        </div>
        <div className="text-[9px] text-[#3a3a48] mt-1">
          memories + skills appear here as you work
        </div>
      </div>
    );
  }

  return (
    <div className="h-full w-full flex flex-col">
      <div className="px-3 pt-2 pb-1 flex-shrink-0">
        <div className="font-mono text-[9px] tracking-[1.5px] text-[#67f6ff]/50 uppercase">
          Semantic Field — click to recall
        </div>
        <div className="flex gap-3 mt-1">
          <span className="flex items-center gap-1 text-[9px] font-mono text-[#5a5a68]">
            <span className="w-1.5 h-1.5 rounded-full bg-[#f5b800] inline-block" /> memory
          </span>
          <span className="flex items-center gap-1 text-[9px] font-mono text-[#5a5a68]">
            <span className="w-1.5 h-1.5 rounded-full bg-[#67f6ff] inline-block" /> skill
          </span>
        </div>
      </div>
      <div className="flex-1 min-h-0">
        <ForceGraph2D
          graphData={graphData}
          nodeLabel="name"
          nodeAutoColorBy="group"
          nodeRelSize={5}
          linkWidth={0.5}
          backgroundColor="transparent"
          nodeColor={(node: any) => node.group === 1 ? '#f5b800' : '#67f6ff'}
          linkColor={() => 'rgba(103,246,255,0.15)'}
          onNodeClick={(node: any) => {
            sendMessage(`Recall: ${node.name}`);
          }}
        />
      </div>
    </div>
  );
}
