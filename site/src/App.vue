<script setup>

import { reactive, ref, onMounted, watch } from 'vue';

import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

const configuration = reactive({
    avatar: false,
    public: true,
    research: true,
});

const authenticated = ref(null);
const conversations = ref([]);
const MAXLENGTH = 200;

const SERVER = (import.meta.env.MODE === 'development') ? 'http://localhost' : location.origin;
const GOOGLE_PROJECT_ID = "188075293614-ngf70nb2fe17b0r32l1dhfm0gu17e2of.apps.googleusercontent.com";

dayjs.extend(relativeTime);

onMounted(async () => {
    google.accounts.id.initialize({
        client_id: GOOGLE_PROJECT_ID,
        callback: async (response) => {
            const resp = await authenticateWithServer(response.credential);
            authenticated.value = await checkIfAuthenticated();
            if (authenticated.value) {
                updateConversationsFromServer();
            }
        },
    });
    authenticated.value = await checkIfAuthenticated();
    if (authenticated.value) {
        updateConversationsFromServer();
    }
});

watch(authenticated, (value) => {
    // If value turned to true, render the Google button directly
    if (!value) {
        google.accounts.id.renderButton(
            document.getElementById("googleButton"),
            {
                theme: "outline",
                size: "large",
            }
        );
    }
});

function dateRepresentation(timestamp) {
    const t = dayjs(timestamp.secs_since_epoch * 1000 + timestamp.nanos_since_epoch / 1000000);
    return t.fromNow();
}

function firstLine(conversation) {
    const dialog = conversation.contents.dialog;
    if (dialog.length === 0) return null;
    const line = dialog[0].what;
    if (line.length <= MAXLENGTH) return line;
    return line.substring(0, MAXLENGTH);
}

function link(id) {
    return `${SERVER}/conversation/html/${id}`;
}
async function downloadMarkdown(id) {
    if (!authenticated.value) {
        return;
    }
    const addr = `${SERVER}/conversation/json/${id}`;
    const options = {
        method: 'GET',
        headers: {
            'content-type': 'application/json',
            'credentials': 'include',
        },
    };
    const response = await fetch(addr, options);
    const data = await response.json();
    console.log(data);
}

async function deleteAction(id) {
    if (!authenticated.value) {
        return;
    }
    const addr = `${SERVER}/api/conversation/${id}`;
    const options = {
        method: 'DELETE',
        headers: {
            'authorization': `Bearer ${token.value}`,
        },
    };
    const response = await fetch(addr, options);
    console.log('Delete response is', response);
    updateConversationsFromServer();
}

async function checkIfAuthenticated() {
    const addr = `${SERVER}/api/authenticated`;
    const options = {
        method: 'POST',
        headers: {
            'content-type': 'application/json',
            'credentials': 'include',
        },
    };
    try {
        const response = await fetch(addr, options);
        return await response.ok;
    } catch(err) {
        return false;
    }
}

async function authenticateWithServer(token) {
    const addr = `${SERVER}/api/authenticate`;
    const options = {
        method: 'POST',
        headers: {
            'content-type': 'application/json',
            'credentials': 'include',
            'authorization': `Bearer ${token}`,
        },
    };
    const response = await fetch(addr, options);
    return await response.ok;
}

async function updateConversationsFromServer() {
    const addr = `${SERVER}/api/conversations`;
    const options = {
        method: 'POST',
        headers: {
            'content-type': 'application/json',
            'credentials': 'include',
        },
    };
    const response = await fetch(addr, options);
    const jsondata = await response.json();
    console.log('Conversation data is', jsondata);
    conversations.value = jsondata;
}

async function logout() {
    const addr = `${SERVER}/api/logout`;
    const options = {
        method: 'POST',
        headers: {
            'content-type': 'application/json',
            'credentials': 'include',
        },
    };
    const response = await fetch(addr, options);
    return await response.ok;
}

async function handleLogout() {
    const res = await logout();
    if (res) {
        authenticated.value = false;
    }
}

// Google GIS scripts need the appAuthenticate callback to be in global window scope
window.appAuthenticate = (arg) => {
    token.value = arg.credential;
    updateConversationsFromServer();
}

</script>

<style>
body {
    @apply bg-gray-200;
}
.btn {
    @apply font-bold py-2 px-4 mr-2 rounded;
}
.btn-blue {
    @apply btn bg-blue-500 text-white;
}
.btn-blue:hover {
    @apply bg-blue-700;
}
.btn-red {
    @apply btn bg-red-500 text-white;
}
.btn-red:hover {
    @apply bg-red-700;
}
p {
    @apply mb-4;
}
span.note {
    @apply font-bold;
}
</style>

<template>
    <div class="container min-w-[300px] mx-auto p-4">
        <h1 className="text-2xl font-bold">
            Share Conversation
        </h1>

        <div v-show="!authenticated" id="googleButton"></div>

        <button v-show="authenticated" @click="handleLogout" class="btn-blue">Logout</button>

        <table v-show="authenticated" class="table p-4 my-4 bg-white rounded-lg shadow">
            <thead>
                <tr>
                    <th class="border-b-2 p-4 dark:border-dark-5 whitespace-nowrap font-normal text-left uppercase text-gray-500">
                        Title
                    </th>
                    <th class="border-b-2 p-4 dark:border-dark-5 whitespace-nowrap font-normal text-left uppercase text-gray-500">
                        Created
                    </th>
                    <th>
                    </th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="item in conversations" class="text-gray-700">
                    <td class="border-b p-4 dark:border-dark-5">
                        <a :href="link(item.id)" target="_blank" class="text-blue-500 hover:text-blue-700">{{ item.metadata.title }}</a>
                    </td>
                    <td class="border-b p-4 dark:border-dark-5">
                        {{ dateRepresentation(item.metadata.creationdate) }}
                    </td>
                    <td class="border-b p-4 dark:border-dark-5">
                        <button class="btn-blue" @click="downloadMarkdown(item.id)">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
                            </svg>
                        </button>
                        <button class="btn-red" @click="deleteAction(item.id)">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
                            </svg>
                        </button>
                    </td>
                </tr>
            </tbody>
        </table>
        <!-- <p v-for="item in conversations">
            <a :href="link(item.id)">{{ item.metadata.title }}</a>
            - {{ dateRepresentation(item.metadata.creationdate) }}
            -
            <button @click="deleteAction(item.id)" class="btn">
            </button>
        </p> -->

    </div>
</template>
