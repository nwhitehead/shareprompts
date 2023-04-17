<script setup>

import { onMounted } from 'vue';

async function getToken() {
    const msg = await chrome.runtime.sendMessage({type: "get_configuration"});
    const config = JSON.parse(msg.config);
    return msg.token;
}

onMounted(async () => {
    const token = await getToken();
    console.log(`Got token ${token}`);
});

</script>

<template>
    <h1>Share Conversations</h1>
    <h2>Manage Conversations</h2>

    <p>This is where conversations would be to manage</p>

</template>
