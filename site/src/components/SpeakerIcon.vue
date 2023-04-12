<script setup>
const props = defineProps(['src']);
</script>

<template>
    <img :src="props.src" width=32 height=32>
</template>
