<script setup>

import { ref, computed, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import ChatGPTIcon from '../../chatgpt.png';
import SpeakerIcon from '../components/SpeakerIcon.vue';

const conversationData = ref(null);

async function getData(id) {
    const addr = `https://shareconversation.com/api/conversation/${id}`;
    const response = await fetch(addr);
    const jsondata = await response.json();
    return jsondata;
}

const route = useRoute();

onMounted(async () => {
    const id = route.params.id;
    const jsondata = await getData(id);
    conversationData.value = jsondata;
});

const avatar = computed(() => {
    if (!conversationData.value) return "";
    return conversationData.value.contents.avatar;
});

const dialog = computed(() => {
    if (!conversationData.value) return [];
    return conversationData.value.contents.dialog;
});

</script>

<template>

    <p v-for="turn in dialog">
        <SpeakerIcon :src="avatar" v-if="turn.who === 'human'" />
        <SpeakerIcon :src="ChatGPTIcon" v-if="turn.who === 'gpt'" />
        {{turn.what}}
    </p>

</template>
