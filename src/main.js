import App from './App.svelte';

let app = null;

window.addEventListener('load', () => {
     app = new App({
        target: document.body,
        props: {
            name: 'world'
        }
    });
});

export default app;