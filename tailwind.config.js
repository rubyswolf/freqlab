/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'bg-primary': '#1a1a1a',
        'bg-secondary': '#242424',
        'bg-tertiary': '#2e2e2e',
        'bg-elevated': '#383838',
        'text-primary': '#ffffff',
        'text-secondary': '#a0a0a0',
        'text-muted': '#6b7280',
        'accent': '#6366f1',
        'accent-hover': '#818cf8',
        'accent-subtle': 'rgba(99, 102, 241, 0.1)',
        'success': '#22c55e',
        'warning': '#f59e0b',
        'error': '#ef4444',
        'error-subtle': 'rgba(239, 68, 68, 0.1)',
        'border': '#3a3a3a',
      },
    },
  },
  plugins: [],
}
