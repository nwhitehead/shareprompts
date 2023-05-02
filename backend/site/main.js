import { marked } from 'marked';
import hljs from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.7.0/build/es/highlight.min.js';

function handleClick() {
    window.open('/', '_blank');
}
window.handleClick = handleClick;

function handleCopy() {
    // Copy current location to clipboard (take out any args)
    navigator.clipboard.writeText(document.location.origin + document.location.pathname);
    // Now show notification bubble
    const copied = document.querySelector("#copied");
    const old_visibility = copied.style.visibility;
    const old_display = copied.style.display;
    copied.style.visibility = "visible";
    copied.style.display = "block";
    // Reset back after a brief time
    setTimeout(() => {
        copied.style.visibility = old_visibility;
        copied.style.display = old_display;
    }, 2000);
}
window.handleCopy = handleCopy;

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
        var language = block.result.language || '';
        block.insertAdjacentHTML("beforebegin",`<div class="flex items-center text-stone-200 bg-stone-800 px-4 py-2 text-xs justify-between rounded-t-md"><span>${language}</span></div>`)
    });
});
