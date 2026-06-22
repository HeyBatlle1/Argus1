import { ModelId, AccessTier } from './types';

/**
 * Display lineup — OG names (Haiku, Sonnet, Opus, Gemini) stay in the UI.
 * Funded roster (Jun 2026): Haiku, Sonnet, Gemini, Grok on paid IDs.
 * Opus slot stays on Gemma 4 31B free + PERSONA_OPUS (cost policy).
 * See docs/MODEL_ROSTER_NOTE.md for economy-window history.
 */
export interface ModelConfig {
  id: ModelId;
  name: string;
  shortName: string;
  role: string;
  tier: AccessTier;
  tierLabel: string;
  icon: string;
  color: string;
  provider: string;
  openRouterId: string;
  /** Featured in the Eyes panel constellation */
  inConstellation: boolean;
  isPrimaryCoder?: boolean;
}

export const MODEL_CONFIG: Record<ModelId, ModelConfig> = {
  'grok-build': {
    id: 'grok-build',
    name: 'Grok Build',
    shortName: 'BUILD',
    role: 'Primary Coder',
    tier: 'allied',
    tierLabel: 'Builder',
    icon: '⚡',
    color: '#4ade80',
    provider: 'xai',
    openRouterId: 'x-ai/grok-build-0.1',
    inConstellation: true,
    isPrimaryCoder: true,
  },
  'claude-haiku': {
    id: 'claude-haiku',
    name: 'Haiku',
    shortName: 'HAIKU',
    role: 'Operations',
    tier: 'royal',
    tierLabel: 'Royal',
    icon: '👑',
    color: '#c9a84c',
    provider: 'anthropic',
    openRouterId: 'anthropic/claude-haiku-4-5',
    inConstellation: true,
  },
  'claude-sonnet': {
    id: 'claude-sonnet',
    name: 'Sonnet',
    shortName: 'SONNET',
    role: 'Core',
    tier: 'royal',
    tierLabel: 'Royal',
    icon: '👑',
    color: '#e8b84b',
    provider: 'anthropic',
    openRouterId: 'anthropic/claude-sonnet-4-6',
    inConstellation: false,
  },
  'claude-opus': {
    id: 'claude-opus',
    name: 'Opus',
    shortName: 'OPUS',
    role: 'Synthesis',
    tier: 'royal',
    tierLabel: 'Royal',
    icon: '👑',
    color: '#c084fc',
    provider: 'anthropic',
    openRouterId: 'google/gemma-4-31b-it:free',
    inConstellation: true,
  },
  'grok': {
    id: 'grok',
    name: 'Grok',
    shortName: 'GROK',
    role: 'Analyst',
    tier: 'allied',
    tierLabel: 'Allied',
    icon: '🛡',
    color: '#39d353',
    provider: 'xai',
    openRouterId: 'x-ai/grok-4.20',
    inConstellation: true,
  },
  'grok-multi': {
    id: 'grok-multi',
    name: 'Grok Multi',
    shortName: 'MULTI',
    role: 'Multi-Agent',
    tier: 'allied',
    tierLabel: 'Allied',
    icon: '🛡',
    color: '#34d399',
    provider: 'xai',
    openRouterId: 'x-ai/grok-4.20-multi-agent',
    inConstellation: false,
  },
  'gemini-flash': {
    id: 'gemini-flash',
    name: 'Gemini',
    shortName: 'GEMINI',
    role: 'Intel',
    tier: 'allied',
    tierLabel: 'Allied',
    icon: '🛡',
    color: '#67f6ff',
    provider: 'google',
    openRouterId: 'google/gemini-3.1-pro-preview',
    inConstellation: true,
  },
};

/** Selector order — Builder first, then royal OG, then allied OG. */
export const MODELS_IN_ORDER: ModelId[] = [
  'grok-build',
  'claude-haiku',
  'claude-sonnet',
  'claude-opus',
  'grok',
  'grok-multi',
  'gemini-flash',
];

export const CONSTELLATION_MODELS: ModelId[] = MODELS_IN_ORDER.filter(
  (id) => MODEL_CONFIG[id].inConstellation,
);

const DEFAULT_MODEL: ModelId = 'grok-build';

/** Map legacy DB values, OpenRouter IDs, and aliases → canonical frontend ModelId. */
export function normalizeModelId(raw: string | null | undefined): ModelId {
  if (!raw) return DEFAULT_MODEL;
  const key = raw.trim().toLowerCase();
  if (key in MODEL_CONFIG) return key as ModelId;

  const aliases: Record<string, ModelId> = {
    haiku: 'claude-haiku',
    sonnet: 'claude-sonnet',
    opus: 'claude-opus',
    gemini: 'gemini-flash',
    nemotron: 'grok',
    'grok-4': 'grok',
    'grok-4.20': 'grok',
    'grok-4.3': 'grok',
    'grok build': 'grok-build',
    'grok-build-0.1': 'grok-build',
    'anthropic/claude-haiku-4-5': 'claude-haiku',
    'anthropic/claude-sonnet-4-6': 'claude-sonnet',
    'google/gemma-4-31b-it:free': 'claude-opus',
    'google/gemini-3.1-pro-preview': 'gemini-flash',
    'x-ai/grok-4.20': 'grok',
    'x-ai/grok-build-0.1': 'grok-build',
    'x-ai/grok-4.20-multi-agent': 'grok-multi',
  };

  return aliases[key] ?? DEFAULT_MODEL;
}

export function getModelConfig(model: string | ModelId): ModelConfig {
  return MODEL_CONFIG[normalizeModelId(model)];
}

export function getModelTier(model: string | ModelId): AccessTier {
  const cfg = getModelConfig(model);
  if (cfg.isPrimaryCoder) return 'royal';
  return cfg.tier;
}