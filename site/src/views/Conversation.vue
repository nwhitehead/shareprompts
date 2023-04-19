<script setup>

import { ref, computed } from 'vue';
import { useRoute } from 'vue-router';
import ChatGPTIcon from '../../chatgpt.png';
import SpeakerIcon from '../components/SpeakerIcon.vue';
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';
import { useHead } from '@unhead/vue';

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

useHead({
    title: 'My Awesome Site',
    meta: [
        { property: 'og:title', content: 'The Rockettes'},
    ]
});

getData(props.id).then((response) => {
    conversationData.value = response;
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

const title = computed(() => {
    if (!conversationData.value) return "Loading";
    return conversationData.value.title;
});

const url = computed(() => {
    // const path = useRoute().fullPath;
    // console.log(path);
    // return path;
    const path = `https://shareconversation.com/conversation/${props.id}`;
    return path;
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
<div class="w-full h-full flex flex-col">
    <div class="dark sticky top-0 bg-gray-800 items-center">
        <div class="flex flex-row flex-1">
            <button class="text-gray-200 p-2">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
                </svg>
            </button>
            <h1 class="text-gray-200 flex-1 text-center p-2">{{title}}</h1>
            <button class="text-gray-200 p-2">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                </svg>
            </button>
        </div>
    </div>
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
</div>

<hr>


<div class="flex flex-row text-2xl text-gray-800">
    <div class="flex-1 grow-0 m-4">
        <ShareNetwork network="facebook" :url="url" :title="title" hashtags="ai">
            <FontAwesomeIcon icon="fa-brands fa-facebook" />
        </ShareNetwork>
    </div>
    <div class="flex-1 grow-0 m-4">
        <ShareNetwork network="twitter" :url="url" :title="title" hashtags="ai">
            <FontAwesomeIcon icon="fa-brands fa-twitter" />
        </ShareNetwork>
    </div>
    <div class="flex-1 grow-0 m-4">
        <ShareNetwork network="reddit" :url="url" :title="title" hashtags="ai">
            <FontAwesomeIcon icon="fa-brands fa-reddit" />
        </ShareNetwork>
    </div>
</div>

<div class="p-[60px]">
</div>

</template>
