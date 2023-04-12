
<script setup>
import NavBar from './components/NavBar.vue';
import '../index.css';
</script>

<template>
    <h1 className="text-3xl font-bold underline text-blue-500">Conversation</h1>
    <NavBar />
    <router-view></router-view>
</template>
