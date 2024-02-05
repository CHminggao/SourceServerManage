<script setup lang="ts">
import HelloWorld from './components/HelloWorld.vue'
import { NMessageProvider, NConfigProvider } from 'naive-ui'
import { zhCN } from 'naive-ui'
import { event } from '@tauri-apps/api'
import { appWindow } from '@tauri-apps/api/window';

event.listen('single-instance', (_) => {appWindow.setFocus();});

document.addEventListener('contextmenu', (e) => {
  // 阻止右键点击的默认行为
  e.preventDefault();

});
</script>

<template>
  <n-config-provider :locale="zhCN">
    <n-message-provider>
      <HelloWorld />
    </n-message-provider>
  </n-config-provider>
</template>

