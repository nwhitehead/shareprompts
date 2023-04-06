
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    // Can't just make this function async, not supported
    if (request.type === "get_configuration") {
        chrome.identity.getProfileUserInfo({}, (userInfo) => {
            console.log('userInfo', userInfo);
            (async () => {
                const dict = await chrome.storage.sync.get('config');
                console.log('Sending config', dict);
                sendResponse(dict);
            })();
        });
        return true; // Tell chrome to expect response asynchronously
    }
});
