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
              '0%, 100%': { transform: 'rotate(-6deg)' },
              '50%': { transform: 'rotate(6deg)' },
            },
            sparkle: {
                '0%, 100%': { filter: 'grayscale(0)' },
                '90%': { filter: 'grayscale(100%)' },
              }
          },
        animation: {
            wiggle: 'wiggle 1s ease-in-out infinite',
            sparkle: 'sparkle 1s ease-in-out infinite',
        },
    },
  },
  plugins: [],
}
