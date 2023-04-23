import { marked } from 'marked';
import hljs from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.7.0/build/es/highlight.min.js';

export function handleClick() {
    window.open('https://shareconversation.com/', '_blank');
}

window.addEventListener("DOMContentLoaded", (event) => {
    // Get everything in .markdown class, interpret as markdown and update DOM
    const markdown = document.querySelectorAll('.markdown');
    for (let i = 0; i < markdown.length; i++) {
        const node = markdown[i];
        const src = node.innerHTML;
        node.innerHTML = marked.parse(src);
    }
    // Look for pre code sections, highlight
    hljs.highlightAll();
    // Update pre code sections with detected language text
    var blocks = document.querySelectorAll('pre code.hljs');
    Array.prototype.forEach.call(blocks, function(block) {
        var language = block.result.language;
        block.insertAdjacentHTML("beforebegin",`<div class="flex items-center text-stone-200 bg-stone-800 px-4 py-2 text-xs justify-between rounded-t-md"><span>${language}</span></div>`)
    });
});
