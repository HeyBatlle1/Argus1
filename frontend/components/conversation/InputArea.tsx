'use client';

import { useState, useRef, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Send } from 'lucide-react';
import { useAgentStore } from '@/hooks/useAgentState';
import { EYE_SYMBOLS } from '@/lib/constants';

export function InputArea() {
  const [value, setValue] = useState('');
  const [focused, setFocused] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const sendMessage = useAgentStore((s) => s.sendMessage);
  const isStreaming = useAgentStore((s) => s.isStreaming);
  const eyeState = useAgentStore((s) => s.eyeState);
  const initConnection = useAgentStore((s) => s.initConnection);

  useEffect(() => { initConnection(); }, [initConnection]);

  useEffect(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 140) + 'px';
  }, [value]);

  function submit() {
    const text = value.trim();
    if (!text || isStreaming) return;
    setValue('');
    sendMessage(text);
  }

  function onKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submit();
    }
  }

  const placeholder = isStreaming
    ? `${EYE_SYMBOLS[eyeState]} ${eyeState === 'thinking' ? 'Thinking...' : eyeState === 'executing' ? 'Executing...' : 'Processing...'}`
    : '◉  Ask Argus anything...';

  const canSend = !!value.trim() && !isStreaming;

  return (
    <div
      className="flex-shrink-0 px-4 py-4"
      style={{ background: '#0d0d14', borderTop: '1px solid #1e1e32' }}
    >
      {/* Elegant input shell */}
      <div
        style={{
          borderRadius: '14px',
          border: isStreaming
            ? '1px solid rgba(201,168,76,0.35)'
            : focused
            ? '1px solid rgba(201,168,76,0.5)'
            : '1px solid #2a2a42',
          background: focused
            ? 'rgba(201,168,76,0.03)'
            : '#0a0a12',
          boxShadow: isStreaming
            ? '0 0 0 3px rgba(201,168,76,0.08), inset 0 1px 0 rgba(255,255,255,0.03)'
            : focused
            ? '0 0 0 3px rgba(201,168,76,0.06), inset 0 1px 0 rgba(255,255,255,0.03)'
            : 'inset 0 1px 0 rgba(255,255,255,0.02)',
          transition: 'border-color 0.2s, box-shadow 0.2s, background 0.2s',
          padding: '10px 12px 10px 16px',
          display: 'flex',
          alignItems: 'flex-end',
          gap: '10px',
        }}
      >
        <textarea
          ref={textareaRef}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={onKeyDown}
          onFocus={() => setFocused(true)}
          onBlur={() => setFocused(false)}
          disabled={isStreaming}
          placeholder={placeholder}
          rows={1}
          className="flex-1 bg-transparent resize-none outline-none leading-relaxed"
          style={{
            fontFamily: "'Instrument Sans', sans-serif",
            fontSize: '14px',
            color: '#e8e8f0',
            maxHeight: '140px',
            paddingBottom: '2px',
          }}
        />

        {/* Send button */}
        <motion.button
          onClick={submit}
          disabled={!canSend}
          className="flex-shrink-0 flex items-center justify-center cursor-pointer"
          style={{
            width: 32,
            height: 32,
            borderRadius: '10px',
            background: canSend ? 'rgba(201,168,76,0.2)' : 'rgba(255,255,255,0.04)',
            border: canSend ? '1px solid rgba(201,168,76,0.5)' : '1px solid #2a2a42',
            color: canSend ? '#c9a84c' : '#3a3a5a',
            cursor: canSend ? 'pointer' : 'not-allowed',
            transition: 'all 0.18s',
          }}
          animate={
            isStreaming
              ? { boxShadow: ['0 0 4px rgba(201,168,76,0.2)', '0 0 12px rgba(201,168,76,0.45)', '0 0 4px rgba(201,168,76,0.2)'] }
              : {}
          }
          transition={isStreaming ? { duration: 1.4, repeat: Infinity } : {}}
          whileHover={canSend ? { scale: 1.06, background: 'rgba(201,168,76,0.3)' } : {}}
          whileTap={canSend ? { scale: 0.93 } : {}}
        >
          <Send size={13} />
        </motion.button>
      </div>

      {/* Hint row */}
      <div className="flex justify-between items-center mt-2 px-1">
        <span className="text-[9px] font-mono" style={{ color: '#2a2a42' }}>
          Enter to send · Shift+Enter for newline
        </span>
        <span className="text-[9px] font-mono" style={{ color: value.length > 200 ? '#c9a84c' : '#2a2a42' }}>
          {value.length > 0 ? value.length : ''}
        </span>
      </div>
    </div>
  );
}
