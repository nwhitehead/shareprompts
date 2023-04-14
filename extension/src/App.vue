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

</script>

<template>
    <h1>Share Conversations</h1>

    <p>
        <label>
            <input type="checkbox" id="avatar" v-model="configuration.avatar" />
            Include actual avatar picture.
        </label>
    </p>
    <p>
        <label>
            <input type="checkbox" id="public" v-model="configuration.public" />
            Include in public index.
        </label>
    </p>
    <p>
        <label>
            <input type="checkbox" id="research" v-model="configuration.research" />
            Allow to be used for artificial intelligence research and development.
        </label>
    </p>

    <p>
        Note: Anyone that has the link to a conversation can see it. The "public" option includes the
        conversation in public lists.
    </p>
    <p> Note: Do not include personally identifying information in the conversations you share.
    </p>
    <p>
        Note: You can delete previously shared conversations from this website but this cannot delete
        any archived copies that others have saved while the conversation was shared.
    </p>

</template>
