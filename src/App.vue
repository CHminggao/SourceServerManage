<script setup lang="ts">
import HelloWorld from './components/HelloWorld.vue'

document.addEventListener('contextmenu', (event) => {
  // 阻止右键点击的默认行为
  event.preventDefault();

});

</script>

<template>
  <HelloWorld />
</template>

