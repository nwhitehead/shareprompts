import { marked } from 'marked';
import hljs from 'highlight.js';

export function handleClick() {
    window.open('https://shareconversation.com/', '_blank');
}

window.addEventListener("DOMContentLoaded", (event) => {
    console.log("DOM fully loaded and parsed");
    const markdown = document.querySelectorAll('.markdown-source');
    for (let i = 0; i < markdown.length; i++) {
        const node = markdown[i];
        const src = node.innerHTML;
        node.innerHTML = marked.parse(src);
    }
});
