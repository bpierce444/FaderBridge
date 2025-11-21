/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // FaderBridge Dark Room Standard palette
        slate: {
          950: '#0a0e14', // Deep background
          900: '#1a1f2e',
          800: '#2a2f3e',
          700: '#3a3f4e',
        },
        cyan: {
          500: '#00d9ff', // Primary accent
          400: '#33e1ff',
          600: '#00b8d9',
        },
        amber: {
          500: '#ffa500', // Warning/activity
        },
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
    },
  },
  plugins: [],
}
