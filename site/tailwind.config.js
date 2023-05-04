/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./**/*.{js,html,vue}",
  ],
  theme: {
    extend: {
        screens: {
            print: {raw: 'print'},
        }
    },
  },
  plugins: [],
}
