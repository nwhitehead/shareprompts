import { marked } from 'marked';
import hljs from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.7.0/build/es/highlight.min.js';

export function handleClick() {
    window.open('https://shareconversation.com/', '_blank');
}

window.addEventListener("DOMContentLoaded", (event) => {
    console.log("DOM fully loaded and parsed");
    const markdown = document.querySelectorAll('.markdown');
    for (let i = 0; i < markdown.length; i++) {
        const node = markdown[i];
        const src = node.innerHTML;
        node.innerHTML = marked.parse(src);
    }
    hljs.highlightAll();
});
