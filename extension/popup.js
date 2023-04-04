console.log("This is a popup");

const { createApp } = Vue;

createApp({
    data() {
        return {
            message: 'Hello Vue!'
        }
    }
}).mount('#app');
