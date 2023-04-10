
<script setup>

const url_params = new URLSearchParams(window.location.search);
const id = url_params.get('id');

console.log('App.vue setup', url_params);

</script>

<template>
    <h1>A Conversation</h1>

    <p>Here is the conversation you requested: {{ id || "none" }}</p>
</template>
