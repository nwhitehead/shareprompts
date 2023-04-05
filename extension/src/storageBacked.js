
import { watch } from 'vue';

export function storageBacked(name, rValue) {
    // Cannot have this function async, breaks Vue somehow
    chrome.storage.sync.get(name).then((val) => {
        if (val[name] !== undefined) {
            const parsed = JSON.parse(val[name]);
            for (const [key, value] of Object.entries(parsed)) {
                rValue[key] = value;
            }
        }
    });
    watch(rValue, (newValue) => {
        let d = {};
        d[name] = JSON.stringify(newValue);
        // Fire off a set, don't wait for it since we have no action after it's done
        chrome.storage.sync.set(d);
    });
    return rValue;
}
