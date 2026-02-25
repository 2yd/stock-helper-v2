/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          red: '#E74C3C',
          orange: '#F39C12',
          gold: '#FFD700',
        },
        bg: {
          base: '#0D1117',
          card: '#161B22',
          elevated: '#1C2333',
        },
        txt: {
          primary: '#E6EDF3',
          secondary: '#8B949E',
          muted: '#484F58',
        },
        functional: {
          up: '#E74C3C',
          down: '#2ECC71',
          warn: '#F39C12',
          info: '#3498DB',
        },
      },
      fontFamily: {
        din: ['"DIN Next"', '"DIN Alternate"', 'system-ui', 'sans-serif'],
        mono: ['"JetBrains Mono"', '"Fira Code"', 'monospace'],
      },
    },
  },
  plugins: [
    require("tailwindcss-animate"),
  ],
}
