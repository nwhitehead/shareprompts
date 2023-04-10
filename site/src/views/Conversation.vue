<script setup>

import { onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';

async function getData(id) {
    console.log('Getting data for conversation', id);
    const addr = `https://shareconversation.com/api/conversation/${id}`;
    console.log(`addr=${addr}`);
    const response = await fetch(addr);
    const jsondata = await response.json();
    console.log('Conversation data is', jsondata);
    return jsondata;
}

const route = useRoute();

onMounted(async () => {
    const id = route.params.id;
    const jsondata = await getData(id);
});

</script>

<template>
    <h1>A Conversation</h1>

    <p>Here is the conversation you requested: {{ $route.params.id }}</p>
</template>
