
<script setup>

// const url_params = new URLSearchParams(window.location.search);
// const id = url_params.get('id');

// console.log('App.vue setup', url_params);

</script>

<template>
    <p><router-link to="/about">Go to About</router-link></p>
    
    <p><router-link to="/conversation/c535280a53db4947bb85df001359c92d">Test convo</router-link></p>

    <router-view></router-view>
</template>
