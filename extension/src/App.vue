<script setup>

import { reactive } from 'vue';
import { storageBacked } from './storageBacked.js';

const configuration = storageBacked('config',
    reactive({
        'avatar': true,
        'public': true,
        'research': true,
    })
);

function handleManage() {
    console.log('Manage');
    window.open('manage.html', '_blank');
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
