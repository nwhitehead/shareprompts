<script setup>

import { onMounted, ref, reactive } from 'vue';
import { storageBacked } from './storageBacked.js';
import ExtPay from "extpay";

let paid = ref(null);
const client_id = 'share-conversations';
let extpay = ExtPay(client_id);

const configuration = storageBacked('config',
    reactive({
        'avatar': true,
        'public': true,
        'research': true,
    })
);

function handleManage() {
    window.open('https://shareconversation.com', '_blank');
}

function handleUpgrade() {
    console.log('Upgrade');
    extpay.openPaymentPage();
}

onMounted(async () => {
    if (paid.value === null) {
        const user = await extpay.getUser();
        paid.value = user.paid;
    }
});

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
.btn-green {
    @apply btn bg-green-500 text-white;
}
.btn-green:hover {
    @apply bg-green-700;
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

        <fieldset class="border border-solid border-stone-300 p-3 mt-4">
            <legend>Options</legend>
            <div class="mb-4 select-none">
                <label>
                    <input type="checkbox" id="avatar" v-model="configuration.avatar" class="mr-2 leading-tight" />
                    Include actual avatar picture.
                </label>
            </div>
            <div class="mb-4 select-none">
                <label>
                    <input type="checkbox" id="public" v-model="configuration.public" class="mr-2 leading-tight" />
                    Include in public index.
                </label>
            </div>
            <div class="select-none">
                <label>
                    <input type="checkbox" id="research" v-model="configuration.research" class="mr-2 leading-tight" />
                    Allow to be used for artificial intelligence research and development.
                </label>
            </div>
        </fieldset>

        <p class="mt-4 mb-4">
            <span class="note">Note</span>: These options only apply to new shared conversations.
        </p>
        <p class="mb-4">
            <span class="note">Note</span>: Anyone that has the link to a conversation can see it. The "public" option includes the
            conversation in public lists.
        </p>
        <p class="mb-4">
            <span class="note">Note</span>: Do not include personally identifying information in the conversations you share.
        </p>
        <p class="mb-4">
            <span class="note">Note</span>: You can delete previously shared conversations but this cannot delete
            any archived copies that others have saved while the conversation was shared.
        </p>

        <button @click="handleManage" class="btn-blue mr-2">Manage conversations</button>
        <button v-if="paid === false" @click="handleUpgrade" class="btn-green">Upgrade</button>
    </div>
</template>
