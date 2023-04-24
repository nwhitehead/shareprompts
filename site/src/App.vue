<script setup>

import { reactive, ref } from 'vue';

const configuration = reactive({
    avatar: false,
    public: true,
    research: true,
});

const conversations = ref(null);

function onSignIn(googleUser) {
  var id_token = googleUser.getAuthResponse().id_token;
  console.log(`id_token=${id_token}`);
}
function handleManage() {
    console.log('Lets do oauth', window);
}

async function updateConversationsFromServer(token) {
    const addr = `http://localhost/api/conversations`;
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

window.appAuthenticate = (arg) => {
    console.log('Auth callback', arg);
    updateConversationsFromServer(arg.credential);
}

</script>

<style>
.btn {
    @apply font-bold py-2 px-4 rounded;
}
.btn-blue {
    @apply btn bg-blue-500 text-white;
}
.btn-blue:hover {
    @apply bg-blue-700;
}
p {
    @apply mb-4;
}
span.note {
    @apply font-bold;
}
</style>

<template>
    <div class="container min-w-[300px] mx-auto p-4 bg-white">
        <h1 className="text-2xl font-bold">
            Share Conversation
        </h1>

<div id="g_id_onload"
     data-client_id="188075293614-ngf70nb2fe17b0r32l1dhfm0gu17e2of.apps.googleusercontent.com"
     data-context="signin"
     data-ux_mode="popup"
     data-callback="appAuthenticate"
     data-auto_select="true"
     data-itp_support="true">
</div>

<div class="g_id_signin"
     data-type="standard"
     data-shape="pill"
     data-theme="outline"
     data-text="signin_with"
     data-size="large"
     data-logo_alignment="left">
</div>

        <fieldset class="border border-solid border-stone-300 p-3">
            <legend>Options</legend>
            <div class="mb-4">
                <label>
                    <input type="checkbox" id="avatar" v-model="configuration.avatar" class="mr-2 leading-tight" />
                    Include actual avatar picture.
                </label>
            </div>
            <div class="mb-4">
                <label>
                    <input type="checkbox" id="public" v-model="configuration.public" class="mr-2 leading-tight" />
                    Include in public index.
                </label>
            </div>
            <div class="">
                <label>
                    <input type="checkbox" id="research" v-model="configuration.research" class="mr-2 leading-tight" />
                    Allow to be used for artificial intelligence research and development.
                </label>
            </div>
        </fieldset>

        <p>
            <span class="note">Note</span>: Anyone that has the link to a conversation can see it. The "public" option includes the
            conversation in public lists.
        </p>
        <p> <span class="note">Note</span>: Do not include personally identifying information in the conversations you share.
        </p>
        <p>
            <span class="note">Note</span>: You can delete previously shared conversations but this cannot delete
            any archived copies that others have saved while the conversation was shared.
        </p>

        <button @click="handleManage" class="btn-blue">Manage my conversations</button>
    </div>
</template>
