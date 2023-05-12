
import ExtPay from "extpay";

const client_id = 'share-conversations';
var extpay = ExtPay(client_id);
extpay.startBackground(); 

async function handle_message(request, _sender, sendResponse) {
    if (request.type === "get_configuration") {
        var extpay = ExtPay(client_id);
        const user = await extpay.getUser();
        const token = await chrome.identity.getAuthToken({interactive: true});
        let dict = await chrome.storage.sync.get('config');
        if (dict.config === undefined) {
            dict = {'config': '{"avatar":true,"public":true,"research":true}'};
        }
        dict.token = token.token;
        dict.paid = user.paid;
        console.log('Sending config', dict);
        sendResponse(dict);
    }
}

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    // Can't just make this function async, not supported
    handle_message(request, sender, sendResponse);
    return true; // Tell chrome to expect response asynchronously
});
