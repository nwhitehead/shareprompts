<script setup>

import { onMounted, ref } from 'vue';

const MAXLENGTH = 200;
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
    return `https://shareconversation.com/conversation/html/${id}`;
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

async function updateConversationsFromServer() {
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
}

onMounted(async () => {
    updateConversationsFromServer();
});

</script>

<template>
    <div class="container min-w-[300px]">
        <h1 className="text-3xl font-bold underline">
            Share Conversation
        </h1>

        <h2>Manage Conversations</h2>

        <p v-for="item in conversations">
            <a :href="link(item.id)">{{ item.id }}</a>
            - {{ firstLine(item) }}
            - {{ dateRepresentation(item.creationdate) }}
            -
            <button @click="deleteAction(item.id)" class="btn">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
                </svg>
            </button>
        </p>
    </div>
</template>
