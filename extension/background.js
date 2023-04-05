

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    (async () => {
        const v = await chrome.storage.sync.get();
        if (request.type === "configuration") {
            const resp = {farewell: "goodbye", config: v};
            sendResponse(resp);
        }    
    })();
    return true; // Tell chrome to expect response asynchronously
});
