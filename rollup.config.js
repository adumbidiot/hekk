import svelte from 'rollup-plugin-svelte';
import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import livereload from 'rollup-plugin-livereload';
import { terser } from 'rollup-plugin-terser';
import babel from '@rollup/plugin-babel';
import sveltePreprocess from 'svelte-preprocess';

const production = !process.env.ROLLUP_WATCH;

export default {
	input: 'src/main.js',
	output: {
		sourcemap: !production,
		format: 'iife',
		name: 'app',
		file: 'public/build/bundle.js'
	},
	plugins: [
		svelte({
            preprocess: sveltePreprocess(),
			dev: !production,
			css: css => css.write('bundle.css'),
		}),
        
		resolve({
			browser: true,
			dedupe: importee => importee === 'svelte' || importee.startsWith('svelte/')
		}),
        
		commonjs(),
        
        !production && livereload('public'),
        
		babel({
			extensions: ['.js', '.mjs', '.html', '.svelte'],
			include: ['src/**', 'node_modules/svelte/**', 'node_modules/carbon-components-svelte/**'],
            babelHelpers: 'bundled',
		}),
        
		!production && serve(),
		production && terser(),
	],
	watch: {
		clearScreen: false
	}
};

function serve() {
	let started = false;

	return {
		writeBundle() {
			if (!started) {
				started = true;

				require('child_process').spawn('yarn', ['run', 'start', '--', '--dev'], {
					stdio: ['ignore', 'inherit', 'inherit'],
					shell: true
				});
			}
		}
	};
}