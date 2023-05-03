
import ExtPay from "extpay";

const client_id = '188075293614-ufu933v3duhro6d8hp1t2pnvc3evkcbq.apps.googleusercontent.com';
var extpay = ExtPay(client_id);
extpay.startBackground(); 

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    // Can't just make this function async, not supported
    if (request.type === "get_configuration") {
        var extpay = ExtPay(client_id);
        extpay.getUser().then((user) => {
            chrome.identity.getAuthToken({interactive: true}, (token) => {
                (async () => {
                    const dict = await chrome.storage.sync.get('config');
                    dict.token = token;
                    dict.paid = user.paid;
                    console.log('Sending config', dict);
                    sendResponse(dict);
                })();
            });
        });
        return true; // Tell chrome to expect response asynchronously
    }
});
