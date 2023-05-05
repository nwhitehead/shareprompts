/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./**/*.{js,html,vue}",
  ],
  theme: {
    extend: {
        screens: {
            print: {raw: 'print'},
        },
        fontFamily: {
            'special': ['Lato', 'sans-serif']
        },
        keyframes: {
            wiggle: {
              '0%, 100%': { transform: 'rotate(-30deg)' },
              '50%': { transform: 'rotate(30deg)' },
            }
        },
        animation: {
            wiggle: 'wiggle 1s ease-in-out infinite',
        },
    },
  },
  plugins: [],
}
