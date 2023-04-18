
async function handleClick(button) {

    const msg = await chrome.runtime.sendMessage({type: "get_configuration"});
    const config = JSON.parse(msg.config);
    const token = msg.token;

    button.textContent = "Sharing...";
    button.style.cursor = "initial";
    button.disabled = true;

    const threadContainer = document.querySelector("main div .flex");

    // Figure out model
    // Start by assuming ChatGPT Plus
    let model = "ChatGPT Plus";
    // If we don't see an upgrade button, then look for specific model in link at bottom
    const upgrade_button = document.querySelector("span.gold-new-button");
    if (upgrade_button && upgrade_button.innerText.includes("Upgrade")) {
        model = document.querySelector("main a.underline").innerHTML;
    }

    // Extract title from left panel
    const title = document.querySelector('a.bg-gray-800 div').textContent;

    const avatar = config.avatar ? getAvatar() : getAnonymousAvatar();
    const conversationData = {
        avatar,
        dialog: [],
    };
    // Fill out dialog part
    for (const node of threadContainer.children) {
        const markdown = node.querySelector(".markdown");
        if ([...node.classList].includes("dark:bg-gray-800")) {
            const warning = node.querySelector(".text-orange-500");
            if (warning) {
                conversationData.dialog.push({
                    who: "human",
                    what: warning.innerText.split("\n")[0],
                });
            } else {
                const text = node.querySelector(".whitespace-pre-wrap");
                conversationData.dialog.push({
                    who: "human",
                    what: text.textContent,
                });
            }
        } else if (markdown) {
            conversationData.dialog.push({
                who: "gpt",
                what: markdown.outerHTML,
            });
        }
    }
    // console.log('config', config);
    // console.log('conversationData', conversationData);
    // console.log('oauth2 token', token);
    // // Actually POST it
    // console.log('POSTing');
    const addr = `https://shareconversation.com/api/conversation/`;
    // console.log(`addr=${addr}`);
    const data = {
        title,
        model,
        public: config.public,
        research: config.research,
        contents: conversationData,
    };
    const options = {
        method: 'POST',
        mode: 'cors',
        headers: {
            'content-type': 'application/json',
            'authorization': `Bearer ${token}`,
        },
        body: JSON.stringify(data, null, 2),
    };
    // console.log('fetch POST options', options);
    const response = await fetch(addr, options);
    const jsondata = await response.json();
    // console.log('Conversation data is', jsondata);
    // Go to new tab with fresh convo
    const url = `https://shareconversation.com/conversation/${jsondata}`;
    button.textContent = "Share";
    button.style.cursor = "pointer";
    button.disabled = false;
    window.open(url, '_blank').focus();
}

function addButton() {
    const button = document.createElement("button");  
    button.id = "share";
    button.classList.add("btn", "flex", "gap-2", "justify-center", "btn-neutral");
    // svg icon here is something I drew in inkscape and optimized online
    button.innerHTML = `<?xml version="1.0" encoding="UTF-8"?>
<svg width="5mm" height="5mm" version="1.1" viewBox="0 0 9.0162 8.8533" xmlns="http://www.w3.org/2000/svg">
<g transform="translate(-4.3461 -4.7617)">
<path d="m9.1966 6.8633c0.0058-0.70053 0.01156-1.4011 0.01734-2.1016l2.0717 1.6875c1.1394 0.92814 2.0739 1.694 2.0765 1.702 0.0042 0.01269-4.1045 3.3842-4.1422 3.3991-0.02249-0.73726-0.01471-1.3-0.01471-2.0956l-0.03901 8e-3c-2.3433 0.14879-3.4247 1.1338-4.6528 4.1523-0.87542-5.5541 1.8295-6.4247 4.6831-6.7516z" fill="#d9d9e3" stroke-width=".017338"/>
</g>
</svg> Share`;
    button.addEventListener("click", async function () {
        handleClick(button);
    });
    const buttonsWrapper = document.querySelector("main form div div");
    buttonsWrapper.appendChild(button);
}

function ensureButton() {
    if (!document.querySelector('#share')) {
        addButton();
    }
}

function getAnonymousAvatar() {
    // Just make a solid color for now
    const canvas = document.createElement("canvas");
    canvas.width = 32;
    canvas.height = 32;
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = "rgb(239,92,128)";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    return canvas.toDataURL("image/png");
}

function getAvatar() {
    try {
        // Get image from page and draw to fresh canvas
        const canvas = document.createElement("canvas");
        const image = document.querySelector("main img.rounded-sm");
        canvas.width = 32;
        canvas.height = 32;
        canvas.getContext("2d").drawImage(image, 0, 0);
        // Save as PNG data url
        return canvas.toDataURL("image/png");
    } catch (error) {
        console.log("Error generating avatar image.");
        return null;
    }
}

// Need to keep adding the share button since React has shadow DOM and single page navigation
setInterval(() => {
    ensureButton();
}, 500);
