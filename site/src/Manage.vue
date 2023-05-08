<script setup>

import { computed, reactive, ref, onMounted, watch } from 'vue';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import { SERVER, GOOGLE_PROJECT_ID } from './config.js';

dayjs.extend(relativeTime);

const props = defineProps(['conversations', 'authenticated']);
const emit = defineEmits(['update']);

const showDeleted = ref(false);

function dateRepresentation(timestamp) {
    const t = dayjs(timestamp.secs_since_epoch * 1000 + timestamp.nanos_since_epoch / 1000000);
    return t.fromNow();
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

function conversationToRows(conversation) {
    const data = conversation;
    let rows = [];
    rows.push(['link', link(data.id)]);
    const date = new Date(data.metadata.creationdate.secs_since_epoch * 1000 + data.metadata.creationdate.nanos_since_epoch * 1e-6).toUTCString();
    rows.push(['date', date]);
    rows.push(['title', data.metadata.title]);
    rows.push(['avatar', data.contents.avatar]);
    rows.push(['model', data.metadata.model]);
    rows.push(['openai_link', openai_link(data.metadata.openaiid)]);
    rows.push(['length', data.metadata.length]);
    rows.push(['', '']);
    rows.push(['who', 'what']);
    for (let i = 0; i < data.contents.dialog.length; i++) {
        const dialog_row = data.contents.dialog[i];
        rows.push([dialog_row.who, dialog_row.what]);
    }
    return rows;
}

function conversationsToRows(convos) {
    if (convos.length < 1) {
        return [];
    }
    // Start with 2 columns from first conversation
    let rows = conversationToRows(convos[0]);
    // Now keep appending new column for each conversation
    for (let i = 1; i < convos.length; i++) {
        const data = conversationToRows(convos[i]);
        const end = data.length > rows.length ? data.length : rows.length;
        for (let j = 0; j < end; j++) {
            if (j >= rows.length) {
                rows.push([data[j][0]]);
            }
            if (j >= data.length) {
                rows[j].push('');
            } else {
                rows[j].push(data[j][1]);
            }
        }
    }
    return rows;
}

async function downloadMarkdown(id) {
    if (!props.authenticated) {
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
    const rows = conversationToRows(data);
    exportCsv(`conversation-${data.id}.csv`, rows);
}

async function downloadAllMarkdown(lst) {
    if (!props.authenticated) {
        return;
    }
    const ids = lst.map((item) => item.id);
    let convos = [];
    for (let i = 0; i < ids.length; i++) {
        const id = ids[i];
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
        convos.push(data);
    }
    const rows = conversationsToRows(convos);
    exportCsv('conversations.csv', rows);
}

async function deleteAction(id) {
    if (!props.authenticated) {
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
    emit('update');
}

async function undeleteAction(id) {
    if (!props.authenticated) {
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
    emit('update');
}

async function handleUpdate(id, field, val) {
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
    emit('update');
}

async function handleSetall(field, val) {
    for (let i = 0; i < props.conversations.length; i++) {
        let conversation = props.conversations[i];
        if (!conversation.deleted && conversation[field] !== val) {
            await handleUpdate(conversation.id, field, val);
        }
    }
}

const filteredConversations = computed(() => {
    return props.conversations.filter((item) => !item.deleted || showDeleted.value);
});

</script>

<template>
    <div class="px-20 pt-15 py-20 max-w-screen-xl xl:mx-auto">
        <template v-if="authenticated">
            <h1 class="text-2xl py-4">Your shared conversations</h1>
            <div class="mb-4 select-none">
                <label>
                    <input type="checkbox" id="showDeleted" v-model="showDeleted" class="my-4 mr-2 leading-tight" />
                    Show deleted conversations.
                </label>
            </div>

            <div v-show="filteredConversations.length === 0" 
                class="bg-white block rounded-lg p-4 shadow">
                <p>
                    You have no conversations to show.
                </p>
                <p>
                    Try clicking on one of your conversations at <a href="https://chat.openai.com" class="text-blue-700 hover:text-blue-500">chat.openai.com</a>
                    and click on the "Share" button.
                </p>
            </div>

            <table v-show="filteredConversations.length > 0" class="table p-4 my-4 bg-white rounded-lg shadow">
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
                        <th class="border-b-2 p-4 dark:border-dark-5 whitespace-nowrap font-normal text-left uppercase text-gray-500" title="Include in public lists">
                            Public
                        </th>
                        <th class="border-b-2 p-4 dark:border-dark-5 whitespace-nowrap font-normal text-left uppercase text-gray-500" title="Allow to be used for artificial intelligence research and development">
                            Research
                        </th>
                        <th class="border-b-2 p-4 dark:border-dark-5">
                        </th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="item in filteredConversations" class="text-gray-700">
                        <td class="border-b p-4 dark:border-dark-5">
                            <a :href="link(item.id)" target="_blank" class="text-blue-700 hover:text-blue-500">{{ item.metadata.title }}</a>
                        </td>
                        <td class="border-b p-4 dark:border-dark-5">
                            {{ dateRepresentation(item.metadata.creationdate) }}
                        </td>
                        <td class="border-b p-4 dark:border-dark-5">
                            <span v-if="!item.deleted" class="whitespace-nowrap rounded-full bg-green-100 px-2.5 py-0.5 text-sm text-green-700">Active</span>
                            <span v-if="item.deleted" class="whitespace-nowrap rounded-full bg-purple-100 px-2.5 py-0.5 text-sm text-purple-700">Deleted</span>
                        </td>
                        <td class="border-b dark:border-dark-5">
                            <button v-if="!item.deleted && item.public" class="checkmark" @click="handleUpdate(item.id, 'public', false)" title="Flip">✓</button>
                            <button v-if="!item.deleted && !item.public" class="checkmark" @click="handleUpdate(item.id, 'public', true)" title="Flip">&nbsp;&nbsp;&nbsp;</button>
                        </td>
                        <td class="border-b dark:border-dark-5">
                            <button v-if="!item.deleted && item.research" class="checkmark" @click="handleUpdate(item.id, 'research', false)" title="Flip">✓</button>
                            <button v-if="!item.deleted && !item.research" class="checkmark" @click="handleUpdate(item.id, 'research', true)" title="Flip">&nbsp;&nbsp;&nbsp;</button>
                        </td>
                        <td class="border-b p-4 dark:border-dark-5">
                            <button v-if="!item.deleted" class="btn-blue" @click="downloadMarkdown(item.id)" title="Download conversation as CSV">
                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
                                </svg>
                            </button>
                            <button v-if="!item.deleted" class="btn-red" @click="deleteAction(item.id)" title="Delete conversation">
                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
                                </svg>
                            </button>
                            <button v-if="item.deleted" class="btn-yellow" @click="undeleteAction(item.id)" title="Undelete conversation">
                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 10.5L12 3m0 0l7.5 7.5M12 3v18" />
                                </svg>
                            </button>
                        </td>
                    </tr>
                </tbody>
            </table>

            <button v-if="filteredConversations.length > 0" class="btn-yellow mb-2" @click="handleSetall('public', true)">
                Set all public
            </button>
            <button v-if="filteredConversations.length > 0" class="btn-yellow mb-2" @click="handleSetall('public', false)">
                Clear all public
            </button>
            <button v-if="filteredConversations.length > 0" class="btn-yellow mb-2" @click="handleSetall('research', true)">
                Set all research
            </button>
            <button v-if="filteredConversations.length > 0" class="btn-yellow mb-2" @click="handleSetall('research', false)">
                Clear all research
            </button>
            <button v-if="filteredConversations.length > 0" class="btn-blue flex flex-row mb-2" @click="downloadAllMarkdown(props.conversations.filter((item) => !item.deleted))" title="Download all conversations as CSV">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4 mr-2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
                </svg> All
            </button>

            <p class="my-4">
                <span class="note">Note</span>: Anyone with a link to a conversation that is not deleted can view the conversation.
            </p>
            <p v-if="showDeleted" class="my-4">
                <span class="note">Note</span>: Conversations marked deleted cannot be viewed by anyone. Anyone following
                a link to a deleted conversation will see an error page. Changes take effect after one minute.
            </p>
            <p v-if="showDeleted" class="my-4">
                <span class="note">Note</span>: Conversations marked deleted may be permanently removed from the server 
                any time after one day of being marked deleted, depending on server storage availability. Once a deleted
                conversation is permanently removed it cannot be undeleted.
            </p>
        </template>
        <template v-if="!authenticated">
            <div class="bg-white block rounded-lg p-4 shadow">
                <p>
                    You need to be signed in to manage your conversations. Click the "Sign In" button on top.
                </p>
            </div>
        </template>
    </div>
</template>
