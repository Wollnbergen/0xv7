/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        sultan: {
          gold: '#F7931A',
          dark: '#0D1117',
          darker: '#010409',
          card: '#161B22',
          border: '#30363D',
          text: '#C9D1D9',
          muted: '#8B949E',
        }
      }
    },
  },
  plugins: [],
}
