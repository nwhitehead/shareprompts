import ExtPay from "extpay";
const client_id = 'share-conversations';
let extpay = ExtPay(client_id);

// Delay in ms between clicking on button and copying clipboard
const CLICK_COPY_DELAY = 5;
// Maximum number of times to poll clipboard before giving up
const MAX_LOOPS = 200;
// Special initial token to represent no change in clipboard (so we know when it changes)
const SPECIAL_EMPTY = "191765971691113173561678--EMPTY--18673571916517656727";

function addShareButtonContent(node) {
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('width', '5mm');
    svg.setAttribute('height', '5mm');
    svg.setAttribute('version', '1.1');
    svg.setAttribute('viewBox', '0 0 9.0162 8.8533');
    const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
    g.setAttribute('transform', 'translate(-4.3461 -4.7617)');
    const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    path.setAttribute('d', 'm9.1966 6.8633c0.0058-0.70053 0.01156-1.4011 0.01734-2.1016l2.0717 1.6875c1.1394 0.92814 2.0739 1.694 2.0765 1.702 0.0042 0.01269-4.1045 3.3842-4.1422 3.3991-0.02249-0.73726-0.01471-1.3-0.01471-2.0956l-0.03901 8e-3c-2.3433 0.14879-3.4247 1.1338-4.6528 4.1523-0.87542-5.5541 1.8295-6.4247 4.6831-6.7516z');
    path.setAttribute('fill', 'rgb(64,65,79)');
    path.setAttribute('stroke-width', '.017338');
    g.appendChild(path);
    svg.appendChild(g);
    node.replaceChildren(...[svg, " Share"]);
}

async function handleClick(button) {

    const msg = await chrome.runtime.sendMessage({type: "get_configuration"});
    const config = JSON.parse(msg.config);
    const paiduser = msg.paid;
    const token = msg.token;

    button.textContent = "Sharing (copying)...";
    button.style.cursor = "initial";
    button.disabled = true;

    const threadContainer = document.querySelector("main div .flex");

    // Figure out model
    const model = document.querySelector("main div.text-center span a.underline").textContent;

    // Extract title from left panel / top titlebar
    const elem = document.querySelector('a.bg-gray-800 div') || document.querySelector('h1');
    const title = elem.textContent;

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
            await navigator.clipboard.writeText(SPECIAL_EMPTY);
            const buttons = node.querySelectorAll('button.flex.ml-auto');
            const button = buttons[buttons.length - 1];
            button.click();
            let txt = SPECIAL_EMPTY;
            let loops = 0;
            while (txt === SPECIAL_EMPTY && loops < MAX_LOOPS) {
                await new Promise(r => setTimeout(r, CLICK_COPY_DELAY));
                txt = await navigator.clipboard.readText();
            }
            if (loops === MAX_LOOPS) {
                console.log('Exceeded maximum number of loops to retrieve ChatGPT text');
                txt = '';
            }
            conversationData.dialog.push({
                who: "gpt",
                what: txt,
            });
        }
    }
    console.log('ConversationData', conversationData);

    const openaiid = document.location.pathname;
    const addr = `https://shareconversation.com/api/conversation/`;
    const data = {
        title,
        openaiid,
        model,
        public: config.public,
        research: config.research,
        contents: conversationData,
        paiduser,
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
    try {
        const response = await fetch(addr, options);
        if (response.ok) {
            const jsondata = await response.json();
            // Go to new tab with fresh convo
            const url = `https://shareconversation.com/conversation/html/${jsondata}`;
            window.open(url, '_blank').focus();
        } else {
            if (response.status === 403 && !paiduser) {
                // Reached free limit
                extpay.openPaymentPage();
            }
        }
    } catch(error) {
        console.log('There was an error during sharing', error);
    }
    addShareButtonContent(button);
    button.style.cursor = "pointer";
    button.disabled = false;
}

function addButton() {
    const buttonsWrapper = document.querySelector("main form div button.relative");
    if (buttonsWrapper) {
        const button = document.createElement("button");
        button.id = "share";
        button.classList.add("btn", "flex", "gap-2", "justify-center", "btn-neutral");
        // svg icon here is something I drew in inkscape and optimized online
        addShareButtonContent(button);
        button.addEventListener("click", async function () {
            handleClick(button);
        });
        buttonsWrapper.insertAdjacentElement("afterend", button);
    }
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
        canvas.width = 48;
        canvas.height = 48;
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
