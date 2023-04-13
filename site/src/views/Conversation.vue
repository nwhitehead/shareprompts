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

const model = computed(() => {
    if (!conversationData.value) return "";
    return conversationData.value.model;
});
function striped(i) {
    return {
        'bg-gray-50': (i % 2 === 0)
    }
}

</script>

<style>
ol {
    @apply list-decimal;
    @apply list-inside;
    @apply pb-4;
}
ul {
    @apply list-disc;
    @apply list-inside;
    @apply pb-4;
}
p {
    @apply pb-4;
    @apply break-words;
    @apply whitespace-pre-wrap;
}
</style>

<template>
    <div class="flex flex-col text-gray-700">
        <div class="group w-full border-b" :class="striped(index)" v-for="(turn, index) in dialog">
            <div class="container mx-auto gap-4 p-4 flex">
                <div class="w-[30px] flex-none">
                    <SpeakerIcon :src="avatar" v-if="turn.who === 'human'" />
                    <SpeakerIcon :src="ChatGPTIcon" v-if="turn.who === 'gpt'" />
                </div>
                <p v-if="turn.who === 'human'">{{turn.what}}</p>
                <p v-if="turn.who === 'gpt'" v-html="turn.what"></p>
            </div>
        </div>
    </div>
    <div class="p-4 container mx-auto text-black/50 text-xs">
        <p>{{model}}</p>
        <p>This conversation was recorded from <a href="https://chat.openai.com/" class="underline">https://chat.openai.com/</a>.
        </p>
    </div>

</template>
