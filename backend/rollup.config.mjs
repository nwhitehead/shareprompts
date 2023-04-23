// rollup.config.mjs

import { nodeResolve } from '@rollup/plugin-node-resolve';

export default {
	input: 'site/main.js',
	output: {
		file: 'dist/main.js',
	},
    plugins: [
        nodeResolve(),
    ]
};
