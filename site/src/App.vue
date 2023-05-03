<script setup>

import { computed, reactive, ref, onMounted, watch } from 'vue';

import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

const configuration = reactive({
    avatar: false,
    public: true,
    research: true,
});

const authenticated = ref(null);
const showDeleted = ref(false);
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
    return `${SERVER}/conversation/html/${id}?cache=0`;
}

function openai_link(id) {
    return `https://chat.openai.com${id}`;
}

/// Generic function to convert JS array into CSV and prompt user to save with filename
// filename: string
// rows: [[string]]
// comments: [[string]]
// rows and comments are both in same format
// comments will be prefixed by "# " and ignored by parser.
function exportCsv(filename, rows) {
    function process(row) {
        let res = '';
        for (let j = 0; j < row.length; j++) {
            let val = row[j].toString();
            let result = val.replace(/"/g, '""');
            if (result.search(/("|,|\n)/g) >= 0) {
                result = '"' + result + '"';
            }
            if (j > 0) {
                res += ',';
            }
            res += result;
        }
        return res + '\n';
    }

    let csvFile = '';
    for (let i = 0; i < rows.length; i++) {
        csvFile += process(rows[i]);
    }

    const blob = new Blob([csvFile], { type: 'text/csv;charset=utf-8;' });
    let link = document.createElement("a");
    const url = URL.createObjectURL(blob);
    link.setAttribute("href", url);
    link.setAttribute("download", filename);
    link.style.visibility = 'hidden';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
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
    let rows = [];
    rows.push(['link', link(data.id)]);
    const date = new Date(data.metadata.creationdate.secs_since_epoch * 1000 + data.metadata.creationdate.nanos_since_epoch * 1e-6).toUTCString();
    rows.push(['date', date]);
    rows.push(['title', data.metadata.title]);
    rows.push(['avatar', data.contents.avatar]);
    rows.push(['model', data.metadata.model]);
    rows.push(['openai_link', openai_link(data.metadata.openaiid)]);
    rows.push(['length', data.metadata.length]);
    rows.push([]);
    rows.push(['who', 'what']);
    for (let i = 0; i < data.contents.dialog.length; i++) {
        const dialog_row = data.contents.dialog[i];
        rows.push([dialog_row.who, dialog_row.what]);
    }
    exportCsv(`conversation-${data.id}.csv`, rows);
}

async function deleteAction(id) {
    if (!authenticated.value) {
        return;
    }
    const addr = `${SERVER}/api/conversation/${id}`;
    const options = {
        method: 'DELETE',
        headers: {
            'content-type': 'application/json',
            'credentials': 'include',
        },
    };
    const response = await fetch(addr, options);
    console.log('Delete response is', response);
    updateConversationsFromServer();
}

async function undeleteAction(id) {
    if (!authenticated.value) {
        return;
    }
    const addr = `${SERVER}/api/conversation/undelete/${id}`;
    const options = {
        method: 'POST',
        headers: {
            'content-type': 'application/json',
            'credentials': 'include',
        },
    };
    const response = await fetch(addr, options);
    console.log('undelete response is', response);
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

async function handleUpdate(id, field, val) {
    console.log(`Making ${id} ${field} = ${val}`);
    const addr = `${SERVER}/conversation/json/${id}?cache=0`;
    const options = {
        method: 'GET',
        headers: {
            'content-type': 'application/json',
            'credentials': 'include',
        },
    };
    const response = await fetch(addr, options);
    const conversation = await response.json();
    console.log(conversation);
    conversation[field] = val;
    const patch_addr = `${SERVER}/api/conversation/${id}`;
    const patch_options = {
        method: 'PATCH',
        headers: {
            'content-type': 'application/json',
            'credentials': 'include',
        },
        body: JSON.stringify(conversation, null, 2),
    };
    const patch_response = await fetch(patch_addr, patch_options);
    await patch_response.ok;
    updateConversationsFromServer();
}

const filteredConversations = computed(() => {
    return conversations.value.filter((item) => !item.deleted || showDeleted.value);
});

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
.btn-yellow {
    @apply btn bg-yellow-500 text-white;
}
.btn-yellow:hover {
    @apply bg-yellow-700;
}
.checkmark {
    @apply rounded py-2 px-2 ml-4 bg-gray-200 hover:bg-gray-300;
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

        <div class="mb-4" v-show="authenticated">
            <label>
                <input type="checkbox" id="showDeleted" v-model="showDeleted" class="mr-2 leading-tight" />
                Show deleted conversations.
            </label>
        </div>

        <div v-show="authenticated && filteredConversations.length === 0" 
            class="bg-white block rounded-lg p-4 shadow">
            <p>
                You have no conversations to show.
            </p>
            <p>
                Try clicking on one of your conversations at <a href="https://chat.openai.com" class="text-blue-500 hover:text-blue-700">chat.openai.com</a>
                and click on the "Share" button.
            </p>
        </div>

        <table v-show="authenticated && filteredConversations.length > 0" class="table p-4 my-4 bg-white rounded-lg shadow">
            <thead>
                <tr>
                    <th class="border-b-2 p-4 dark:border-dark-5 whitespace-nowrap font-normal text-left uppercase text-gray-500">
                        Title
                    </th>
                    <th class="border-b-2 p-4 dark:border-dark-5 whitespace-nowrap font-normal text-left uppercase text-gray-500">
                        Created
                    </th>
                    <th class="border-b-2 p-4 dark:border-dark-5 whitespace-nowrap font-normal text-left uppercase text-gray-500">
                        Status
                    </th>
                    <th class="border-b-2 p-4 dark:border-dark-5 whitespace-nowrap font-normal text-left uppercase text-gray-500">
                        Public
                    </th>
                    <th class="border-b-2 p-4 dark:border-dark-5 whitespace-nowrap font-normal text-left uppercase text-gray-500">
                        Research
                    </th>
                    <th></th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="item in filteredConversations" class="text-gray-700">
                    <td class="border-b p-4 dark:border-dark-5">
                        <a :href="link(item.id)" target="_blank" class="text-blue-500 hover:text-blue-700">{{ item.metadata.title }}</a>
                    </td>
                    <td class="border-b p-4 dark:border-dark-5">
                        {{ dateRepresentation(item.metadata.creationdate) }}
                    </td>
                    <td class="border-b p-4 dark:border-dark-5">
                        <span v-if="!item.deleted" class="whitespace-nowrap rounded-full bg-green-100 px-2.5 py-0.5 text-sm text-green-700">Active</span>
                        <span v-if="item.deleted" class="whitespace-nowrap rounded-full bg-purple-100 px-2.5 py-0.5 text-sm text-purple-700">Deleted</span>
                    </td>
                    <td>
                        <button v-if="!item.deleted && item.public" class="checkmark" @click="handleUpdate(item.id, 'public', false)">✓</button>
                        <button v-if="!item.deleted && !item.public" class="checkmark" @click="handleUpdate(item.id, 'public', true)">&nbsp;&nbsp;&nbsp;</button>
                    </td>
                    <td>
                        <button v-if="!item.deleted && item.research" class="checkmark" @click="handleUpdate(item.id, 'research', false)">✓</button>
                        <button v-if="!item.deleted && !item.research" class="checkmark" @click="handleUpdate(item.id, 'research', true)">&nbsp;&nbsp;&nbsp;</button>
                    </td>
                    <td class="border-b p-4 dark:border-dark-5">
                        <button class="btn-blue" @click="downloadMarkdown(item.id)" title="Download conversation as CSV">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
                            </svg>
                        </button>
                        <button v-if="!item.deleted" class="btn-red" @click="deleteAction(item.id)" title="Delete conversation">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
                            </svg>
                        </button>
                        <button v-if="item.deleted" class="btn-yellow" @click="undeleteAction(item.id)" title="Undelete conversation">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 10.5L12 3m0 0l7.5 7.5M12 3v18" />
                            </svg>
                        </button>
                    </td>
                </tr>
            </tbody>
        </table>

        <p>
            <span class="note">Note</span>: Anyone with a link to a conversation that is not deleted can view the conversation.
        </p>
        <p v-if="showDeleted">
            <span class="note">Note</span>: Conversations marked deleted cannot be viewed by anyone. Anyone following
            a link to a deleted conversation will see an error page. Changes take effect after one minute.
        </p>
        <p v-if="showDeleted">
            <span class="note">Note</span>: Conversations marked deleted may be permanently removed from the server 
            any time after one day of being marked deleted, depending on server storage availability. Once a deleted
            conversation is permanently removed it cannot be undeleted.
        </p>

    </div>
</template>
