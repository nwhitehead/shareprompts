
import ExtPay from "extpay";

const client_id = 'share-conversations';
var extpay = ExtPay(client_id);
extpay.startBackground(); 

async function authorize() {
    const redirectURL = browser.identity.getRedirectURL();
    const clientID = "188075293614-ufu933v3duhro6d8hp1t2pnvc3evkcbq.apps.googleusercontent.com";
    const scopes = [ "openid" ];
    let authURL = "https://accounts.google.com/o/oauth2/auth";
    authURL += `?client_id=${clientID}`;
    authURL += `&response_type=token`;
    authURL += `&redirect_uri=${encodeURIComponent(redirectURL)}`;
    authURL += `&scope=${encodeURIComponent(scopes.join(' '))}`;
  
    return browser.identity.launchWebAuthFlow({
      interactive: true,
      url: authURL
    });
  }

async function handle_message(request, _sender, sendResponse) {
    if (request.type === "get_configuration") {
        var extpay = ExtPay(client_id);
        const user = await extpay.getUser();
        const token = await authorize();
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
