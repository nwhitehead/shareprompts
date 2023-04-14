<script setup>

import { ref, computed, onMounted, onUpdated } from 'vue';
import ChatGPTIcon from '../../chatgpt.png';
import SpeakerIcon from '../components/SpeakerIcon.vue';

import hljs from 'highlight.js';
import 'highlight.js/styles/stackoverflow-dark.css';

const props = defineProps(['id']);

const conversationData = ref(null);
const domref = ref(null);

async function getData(id) {
    const addr = `https://shareconversation.com/api/conversation/${id}`;
    const response = await fetch(addr);
    const jsondata = await response.json();
    return jsondata;
}

onMounted(async () => {
    const id = props.id;
    const jsondata = await getData(id);
    conversationData.value = jsondata;
});

onUpdated(() => {
    console.log('Updated DOM for conversation');
    console.log(domref);
    // domref.value.querySelectorAll('pre code').forEach((el) => {
    //     console.log(el);
    // });
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
function striped(turn) {
    return {
        'bg-gray-50': (turn.who === 'gpt')
    }
}

</script>

<style>
a {
    @apply underline;
}
ol {
    @apply list-decimal;
    @apply list-outside;
    @apply pb-4 pl-4;
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
code:not(.hljs) {
    font-weight: 700;
}
code:not(.hljs)::before {
    font-weight: 700;
    content: '`';
}
code:not(.hljs)::after {
    font-weight: 700;
    content: '`';
}
</style>

<template>
    <div class="flex flex-col text-gray-700" ref="domref">
        <p class="container mx-auto p-4" v-if="conversationData === null">Loading...</p>
        <div class="group w-full border-b" :class="striped(turn)" v-for="turn in dialog">
            <div class="container mx-auto gap-x-6 p-4 flex">
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
        <p>This conversation was recorded from <a href="https://chat.openai.com/">https://chat.openai.com/</a>.</p>
        <p><a href="/">ShareConversation</a> has no affiliation with OpenAI or ChatGPT.</p>
    </div>

</template>
