

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    (async () => {
        const dict = await chrome.storage.sync.get('config');
        if (request.type === "get_configuration") {
            sendResponse(dict);
        }    
    })();
    return true; // Tell chrome to expect response asynchronously
});
