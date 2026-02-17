import type { Config } from 'tailwindcss'

// Colors and fonts are defined in src/styles/globals.css @theme block (Tailwind v4 native).
// This config provides borderRadius and spacing overrides that @theme does not support,
// and is auto-detected by @tailwindcss/postcss for backward compatibility.
export default {
  darkMode: 'class',
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
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
