<script setup>

import { onMounted, ref } from 'vue';

const MAXLENGTH = 20;
let token = null;

async function getToken() {
    const msg = await chrome.runtime.sendMessage({type: "get_configuration"});
    const config = JSON.parse(msg.config);
    return msg.token;
}

let conversations = ref([]);

function dateRepresentation(timestamp) {
    const d = new Date(timestamp.secs_since_epoch * 1000 + timestamp.nanos_since_epoch / 1000000);
    return d.toString();
}

function firstLine(conversation) {
    const dialog = conversation.contents.dialog;
    if (dialog.length === 0) return null;
    const line = dialog[0].what;
    if (line.length <= MAXLENGTH) return line;
    return line.substring(0, MAXLENGTH);
}

function link(id) {
    return `https://shareconversation.com/conversation/${id}`;
}

async function deleteAction(id) {
    if (token === null) {
        token = await getToken();
    }
    console.log(`Got token ${token}`);
    const addr = `https://shareconversation.com/api/conversation/${id}`;
    const options = {
        method: 'DELETE',
        mode: 'cors',
        headers: {
            'authorization': `Bearer ${token}`,
        },
    };
    console.log('Options', options);
    const response = await fetch(addr, options);
    console.log('Delete response is', response);
}

onMounted(async () => {
    if (token === null) {
        token = await getToken();
    }
    console.log(`Got token ${token}`);
    const addr = `https://shareconversation.com/api/conversations`;
    const options = {
        method: 'GET',
        mode: 'cors',
        headers: {
            'content-type': 'application/json',
            'authorization': `Bearer ${token}`,
        },
    };
    console.log('Options', options);
    const response = await fetch(addr, options);
    const jsondata = await response.json();
    console.log('Conversation data is', jsondata);
    conversations.value = jsondata;
});

</script>

<template>
    <h1>Share Conversations</h1>
    <h2>Manage Conversations</h2>

    <p v-for="item in conversations">
        <a :href="link(item.id)">{{ item.id }}</a>
        - {{ firstLine(item) }}
        - {{ dateRepresentation(item.creationdate) }}
        - <button @click="deleteAction(item.id)">Delete</button>
    </p>

</template>
