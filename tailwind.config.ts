import type { Config } from 'tailwindcss'

export default {
  darkMode: 'class',
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        base: '#0A0A0F',
        surface: '#141419',
        elevated: '#1E1E26',
        'text-primary': '#F4F4F5',
        'text-secondary': '#B4B4BB',
        'text-muted': '#71717A',
        'border-default': '#27272A',
        'border-subtle': '#1E1E26',
        'accent-primary': '#F59E0B',
        'accent-hover': '#D97706',
        'trace-best': '#22C55E',
        'trace-average': '#71717A',
        'trace-regression': '#EF4444',
        success: '#22C55E',
        warning: '#FBBF24',
        error: '#EF4444',
        info: '#3B82F6',
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      borderRadius: {
        sm: '4px',
        md: '6px',
        lg: '8px',
      },
      spacing: {
        '1': '4px',
        '2': '8px',
        '3': '12px',
        '4': '16px',
        '6': '24px',
        '8': '32px',
        '12': '48px',
      },
    },
  },
} satisfies Config
