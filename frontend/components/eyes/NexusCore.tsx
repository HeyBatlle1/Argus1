'use client';

import { useEffect, useRef } from 'react';
import { EyeState } from '@/lib/types';

interface Props {
  eyeState: EyeState;
  pulse: number;
  size?: number;
}

const EYE_COLORS: Record<EyeState, string> = {
  watching:  '#39d353',
  thinking:  '#f5b800',
  executing: '#67f6ff',
  complete:  '#ffffff',
};

export function NexusCore({ eyeState, pulse, size = 200 }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rafRef = useRef<number | null>(null);
  const liveRef = useRef({ eyeState, pulse });

  useEffect(() => { liveRef.current = { eyeState, pulse }; }, [eyeState, pulse]);

  useEffect(() => {
    const c = canvasRef.current;
    if (!c) return;
    const ctx = c.getContext('2d', { alpha: true })!;
    const W = 420, H = 420;
    c.width = W; c.height = H;
    const cx = W / 2, cy = H / 2;
    const EYE_COUNT = 19;
    let t = 0;

    function draw() {
      const { eyeState: es, pulse: p } = liveRef.current;
      const col = EYE_COLORS[es];

      ctx.clearRect(0, 0, W, H);

      // Central orb
      const r = 58 + Math.sin(t * 1.8) * 3 + (p - 3) * 0.8;
      const grad = ctx.createRadialGradient(cx, cy, r * 0.3, cx, cy, r * 1.7);
      grad.addColorStop(0, 'rgba(255,255,255,0.88)');
      grad.addColorStop(0.4, col + '88');
      grad.addColorStop(1, 'rgba(5,5,8,0.0)');
      ctx.fillStyle = grad;
      ctx.beginPath();
      ctx.arc(cx, cy, r, 0, Math.PI * 2);
      ctx.fill();

      // Circuit rings
      ctx.strokeStyle = 'rgba(103,246,255,0.10)';
      ctx.lineWidth = 1;
      for (let i = 0; i < 5; i++) {
        ctx.beginPath();
        ctx.arc(cx, cy, 72 + i * 18 + Math.sin(t + i) * 2, 0, Math.PI * 2);
        ctx.stroke();
      }

      // Orbiting eyes — the hundred eyes made visible
      for (let i = 0; i < EYE_COUNT; i++) {
        const angle = (i / EYE_COUNT) * Math.PI * 2 + t * (0.3 + i * 0.01);
        const dist = 118 + Math.sin(t * 2 + i) * 6 + (p - 4) * 0.6;
        const ex = cx + Math.cos(angle) * dist;
        const ey = cy + Math.sin(angle) * dist * 0.92;
        const sz = 4.2 + Math.sin(t * 3 + i * 1.3) * 1.1;

        ctx.fillStyle = col;
        ctx.beginPath();
        ctx.arc(ex, ey, sz, 0, Math.PI * 2);
        ctx.fill();

        // Pupil looking inward
        ctx.fillStyle = '#050508';
        ctx.beginPath();
        ctx.arc(
          ex - Math.cos(angle) * sz * 0.45,
          ey - Math.sin(angle) * sz * 0.45,
          sz * 0.42, 0, Math.PI * 2
        );
        ctx.fill();
      }

      // Pulse lines from center
      ctx.strokeStyle = col + '26';
      ctx.lineWidth = 1.5;
      for (let i = 0; i < EYE_COUNT; i += 2) {
        const a = (i / EYE_COUNT) * Math.PI * 2 + t * 0.4;
        ctx.beginPath();
        ctx.moveTo(cx, cy);
        ctx.lineTo(cx + Math.cos(a) * 92, cy + Math.sin(a) * 86);
        ctx.stroke();
      }

      t += 0.014;
      rafRef.current = requestAnimationFrame(draw);
    }

    draw();
    return () => { if (rafRef.current) cancelAnimationFrame(rafRef.current); };
  }, []);

  return (
    <div
      className="relative flex items-center justify-center flex-shrink-0 core-orb"
      style={{ width: size, height: size }}
    >
      <canvas
        ref={canvasRef}
        style={{ width: size, height: size, borderRadius: '50%' }}
      />
      <div className="absolute text-center pointer-events-none select-none">
        <div className="font-mono uppercase text-[#67f6ff]/50" style={{ fontSize: 7, letterSpacing: '3px' }}>
          ARGUS
        </div>
        <div className="font-display text-white/90 -mt-0.5" style={{ fontSize: 11, letterSpacing: '-0.5px' }}>
          NEXUS
        </div>
        <div className="font-mono uppercase text-[#f5b800]/40 mt-0.5" style={{ fontSize: 6, letterSpacing: '1.5px' }}>
          {eyeState}
        </div>
      </div>
    </div>
  );
}
