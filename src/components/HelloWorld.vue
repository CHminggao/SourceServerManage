<script setup lang="ts">
import { h, onMounted, ref } from 'vue'
import { NDataTable, NButton, NSwitch, NModal, NDynamicInput,NTooltip } from 'naive-ui'
import { createDiscreteApi } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { invoke } from '@tauri-apps/api'

type ServerInfo = {
  name: string,
  map: string,
  game: string,
  players: Number,
  max_players: Number,
  ip: string,
  port: string,
  incount: Number
}

const colums = [
  {
    title: '服务器',
    key: 'name',
    minWidth: 750
  }, {
    title: '地图',
    key: 'map',
    width: 250
  },
  {
    title: '人数',
    key: 'players',
    sorter: 'default',
    width: 110,
    render(row: ServerInfo) {
      return row.players + '/' + row.max_players
    }
  }, {
    title: '操作',
    render(row: ServerInfo) {
      return h(
        NButton,
        {
          strong: true,
          tertiary: true,
          type: "primary",
          size: 'small',
          onClick: () => start(row)
        },
        { default: () => '加入' }
      )
    }
  }
]
let tempData: ServerInfo[] = [];
const rowData = ref(tempData)

const start = (server: ServerInfo) => {
  if (server.players > server.incount)
    watingStart(server)
  else
    gotoServer(server.ip + ":" + server.port)
}
const gotoServer = (com: string) => {
  invoke('start_game', { server: "steam://connect/" + com }).then(() => { })
}
let it = 0
const watingStart = (server: ServerInfo) => {
  clearInterval(it)
  const { dialog } = createDiscreteApi(
    ['dialog']
  )
  let time = 0
  let warning = dialog.warning({
    title: '人数已满',
    content: () => h('div',
      [
        h('p', `${server.name}`),
        h('p', `地图：${server.map}`),
        h('p', `人数：${server.players}/${server.max_players}`),
        h('p', `已等待：${Math.floor(time / 60)}:${time % 60}`)
      ]),
    positiveText: '直接加入',
    negativeText: '取消',
    maskClosable: false,
    onAfterLeave: () => {
      clearInterval(it)
    },
    onPositiveClick: () => {
      gotoServer(server.ip + ":" + server.port)
    }
  })

  it = setInterval(() => {
    invoke('server_info', { ip: server.ip, port: server.port }).then((result) => {
      let s = <ServerInfo>JSON.parse(<string>result)
      time += 1
      if (s.players < server.incount) {
        gotoServer(server.ip + ":" + server.port)
        warning.destroy()
        return
      }
      warning.content = () => h('div', [
        h('p', `${server.name}`),
        h('p', `地图：${server.map}`),
        h('p', `人数：${s.players}/${s.max_players}`),
        h('p', `已等待：${Math.floor(time / 60)}:${time % 60}`)
      ])
    })
  }, 1000);
}

const updateData = (data: ServerInfo[]) => {
  rowData.value = data
  if (jkActive.value && configInput.value.length > 0) {
    jkRowData.value = (data.filter(s => configInput.value.includes(s.map)))
    jkModal.value = true
  }
}

const getAllData = () => {
  getAllDataLoading.value = true
  invoke('server_all').then((result) => {
    updateData(JSON.parse(<string>result))
  }).finally(() => {
    getAllDataLoading.value = false
  })
}

let configTemp: (string)[] = []
const configInput = ref(configTemp)
const saveConfig = () => {
  console.log(JSON.stringify(configInput.value))
  invoke('write_config_file', { jsonString: JSON.stringify(configInput.value) }).then((_) => {
  }).finally(() => {
    showModal.value = false
  })
}
const readConfig = () => {
  invoke('read_from_config_file').then((_) => {
    configInput.value = JSON.parse(<string>_)
  }).finally(() => {
    showModal.value = true
  })
}

const getAllDataLoading = ref(false)
const active = ref(false)
const jkActive = ref(false)
const showModal = ref(false)
const jkModal = ref(false)
let jkDataTemp: ServerInfo[] = []
const jkRowData = ref(jkDataTemp)
const jkUpdateEvent = (s:boolean) => {
  if(s){
    active.value=s
  }
}

onMounted(() => {
  invoke('read_from_config_file').then((_) => {
    configInput.value = JSON.parse(<string>_)
  })
  getAllData()
  setInterval(() => {
    if (active.value) {
      getAllData()
    }
  }, 10000)
})
</script>

<template>
  <div style="width: 96vw;height: 98vh;">
    <div class="top_group">
      <div class="button_gp">
        <n-button :loading="getAllDataLoading" @click="getAllData" type="info">刷新</n-button>
        <span style="margin-left:10px;">自动刷新 
          
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-switch v-model:value="active" />
            </template>
            开启后每隔10s自动刷新
          </n-tooltip>
        </span>
        <n-button style="margin-left:10px;" @click="readConfig" type="info">监控地图</n-button>
        <span style="margin-left:10px;">开启监控
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-switch @update:value="jkUpdateEvent" v-model:value="jkActive" />
            </template>
            开启监控后，会开启自动刷新，并显示监控地图
          </n-tooltip>
        </span>
        <!-- <span style="margin-left:10px;">自动加入 <n-switch v-model:value="active" /></span> -->
      </div>
    </div>


    <div class="card">
      <n-data-table max-height="550" min-height="550" :columns="(colums as DataTableColumns)" :data="rowData" />
    </div>
    <n-modal style="width: 500px;" v-model:show="showModal" preset="card" title="监控地图" size="small" :bordered="true">
      <div style="max-height: 400px;overflow-y: auto;">
        <n-dynamic-input v-model:value="configInput" placeholder="请输入" />
      </div>
      <n-button style="margin-top:10px;" @click="saveConfig" type="info">保存</n-button>
    </n-modal>
    <n-modal v-model:show="jkModal" preset="card" title="服务器" size="small" :bordered="true">
      <n-data-table max-height="400" min-height="400" :columns="(colums as DataTableColumns)" :data="jkRowData" />
    </n-modal>
  </div>
</template>

<style scoped>
.card {
  margin-top: 12px;
}

.button_gp {
  margin-top: 12px;
  text-align: left;
}

.top_group {
  display: flex;
  justify-content: space-between;
}
</style>
