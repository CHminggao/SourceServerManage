<script setup lang="ts">
import HelloWorld from './components/HelloWorld.vue'
import { NMessageProvider,NConfigProvider } from 'naive-ui'
import { zhCN } from 'naive-ui'
document.addEventListener('contextmenu', (event) => {
  // 阻止右键点击的默认行为
  event.preventDefault();

});

</script>

<template>
  <n-config-provider :locale="zhCN" >
    <n-message-provider>
    <HelloWorld />
  </n-message-provider>
  </n-config-provider>
  

</template>

