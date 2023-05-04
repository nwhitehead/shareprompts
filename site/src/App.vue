<script setup>

import { computed, reactive, ref, onMounted, watch } from 'vue';
import { SERVER, GOOGLE_PROJECT_ID } from './config.js';
import Manage from './Manage.vue';

const authenticated = ref(null);
const conversations = reactive([]);
const MAXLENGTH = 200;
let gis_initialized = false;

onMounted(async () => {
    window.addEventListener("load", () => {
        google.accounts.id.initialize({
            client_id: GOOGLE_PROJECT_ID,
            callback: async (response) => {
                const resp = await authenticateWithServer(response.credential);
                authenticated.value = await checkIfAuthenticated();
            },
        });
        gis_initialized = true;
        google.accounts.id.renderButton(
            document.getElementById("googleButton"),
            {
                theme: "outline",
                size: "large",
            }
        );
    });
    authenticated.value = await checkIfAuthenticated();
});

watch(authenticated, (value) => {
    if (value) {
        // If value turned true, get conversations
        updateConversationsFromServer();
    } else {
        // If value turned to false, render the Google button directly (e.g. on logout)
        if (gis_initialized) {
            google.accounts.id.renderButton(
                document.getElementById("googleButton"),
                {
                    theme: "outline",
                    size: "large",
                }
            );
        }
    }
});

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
    conversations.length = 0;
    conversations.push(...jsondata);
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

<template>
    <div class="container min-w-[300px] mx-auto p-4">
        <h1 className="text-2xl font-bold">
            Share Conversation
        </h1>

        <div v-show="!authenticated" id="googleButton" class="my-4"></div>

        <template v-if="authenticated">

            <button @click="handleLogout" class="my-4 btn-blue">Logout</button>

            <Manage
                :authenticated="authenticated"
                :conversations="conversations"
                @update="updateConversationsFromServer()"
            />

        </template>
    </div>
</template>
