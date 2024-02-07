<script setup lang="ts">
import { h, onMounted, ref } from 'vue'
import { NDataTable, NButton, NSwitch, NModal, NDynamicInput, NTooltip, NForm, NFormItem, NInput, NRow, NCol } from 'naive-ui'
import { createDiscreteApi } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import {
  FormInst,
  FormItemRule,
  useMessage,
  FormRules
} from 'naive-ui'
import { invoke } from '@tauri-apps/api'
import { open } from '@tauri-apps/api/shell'



const message = useMessage()

type ServerInfo = {
  name: string,
  map: string,
  game: string,
  players: number,
  max_players: number,
  ip: string,
  port: string,
  incount: number
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
      return h('div', { style: "width: 100px;display: flex;width: 100px;justify-content: space-between;" }, [
        h(
          NButton,
          {
            strong: true,
            tertiary: true,
            secondary: true,
            type: "primary",
            size: 'small',
            onClick: () => start(row)
          },
          { default: () => '加入' }
        ),
        h(
          NButton,
          {
            strong: true,
            tertiary: true,
            secondary: true,
            type: "error",
            size: 'small',
            onClick: () => delServer(row)
          },
          { default: () => '删除' }
        )
      ])
    }
  }
]
let tempData: ServerInfo[] = [];
const rowData = ref(tempData)

const start = (server: ServerInfo) => {
  autoInActive.value = false
  if (server.players > server.incount)
    watingStart(server)
  else
    gotoServer(server.ip + ":" + server.port)
}

const delServer = (row: ServerInfo) => {
  invoke('read_server').then((_) => {
    let server: ServerFileType[] = JSON.parse(<string>_)
    server = server.filter(s => {
      if (s.ip === row.ip && s.port?.toString() === row.port) {
        if (s.incount?.toString() === row.incount.toString()) {
          return false
        }
      }
      return true
    });
    invoke('write_server', { jsonString: JSON.stringify(server) }).then((_) => { }).finally(() => {
      message.success('删除成功')
      getAllData()
    })
  })
}

const gotoServer = (com: string) => {
  open("steam://connect/" + com).then(() => { })
}

let warningState = false
const watingStart = (server: ServerInfo) => {
  const { dialog } = createDiscreteApi(
    ['dialog']
  )
  let time = 0
  warningState = true
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
      warningState = false
    },
    onPositiveClick: () => {
      gotoServer(server.ip + ":" + server.port)
    }
  })

  let it = () => {
    invoke('server_info', { ip: server.ip, port: server.port }).then((result) => {
      let s = <ServerInfo>JSON.parse(<string>result)
      time += 1
      if (s.players < server.incount) {
        gotoServer(server.ip + ":" + server.port)
        warning.destroy()
        return
      } else {
        warning.content = () => h('div', [
          h('p', `${server.name}`),
          h('p', `地图：${server.map}`),
          h('p', `人数：${s.players}/${s.max_players}`),
          h('p', `已等待：${Math.floor(time / 60)}:${time % 60}`)
        ])
        setTimeout(() => {
          it()
        }, 1000);
      }
    })
  }
  it()
}

const updateData = (data: ServerInfo[]) => {
  rowData.value = data
  if (!warningState && jkActive.value && configInput.value.length > 0 && data.filter(s => configInput.value.includes(s.map)).length > 0) {
    let d = data.filter(s => configInput.value.includes(s.map)).sort((a, b) => b.players - a.players)
    jkRowData.value = d
    jkModal.value = true
    if (autoInActive.value && !warningState) {
      if (d.length == 1) {
        // console.log(1)
        start(d[0])
      } else {
        let nohas = d.filter(z => z.players > z.incount)
        let has = d.filter(z => z.players <= z.incount)
        if (nohas.length > 0 && has.length > 0 && (has[0].players >= 20 || has[0].incount <= 20)) {
          start(has[0])
          // console.log(2)
        } else if (has.length > 0) {
          start(has[0])
          // console.log(3)
        } else if (nohas.length > 0) {
          start(nohas[0])
          // console.log(4)
        }
      }
    }
  }
}

const getAllData = () => {
  getAllDataLoading.value = true
  invoke('server_all').then((result) => {
    updateData(JSON.parse(<string>result))
  }).finally(() => {
    getAllDataLoading.value = false
    setTimeout(() => {
      if (active.value) {
        getAllData()
      }
    }, 1000);
  })
}

let configTemp: (string)[] = []
const configInput = ref(configTemp)
const saveConfig = () => {
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
const autoInActive = ref(false)

const showModal = ref(false)
const jkModal = ref(false)
let jkDataTemp: ServerInfo[] = []
const jkRowData = ref(jkDataTemp)
const jkUpdateEvent = (s: boolean) => {
  if (s) {
    if (!active.value) {
      getAllData()
    }
    active.value = s
  }
}

interface ModelType {
  ip: string | null
  port: string | null
  incount: string | null
}
interface ServerFileType {
  ip: string | null
  port: Number | null
  incount: Number | null
}

const addModal = ref(false)
const formRef = ref<FormInst | null>(null)
const modelRef = ref<ModelType>({
  ip: '',
  port: '1',
  incount: '2'
})
const rules: FormRules = {
  ip: [
    {
      required: true,
      validator(rule: FormItemRule, value: string) {
        const ipv4Regex: RegExp =
          /^(25[0-5]|2[0-4][0-9]|[0-1]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[0-1]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[0-1]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[0-1]?[0-9][0-9]?)$/;
        console.log(rule)
        if (!ipv4Regex.test(value)) {
          return new Error('ip不正确')
        }
        return true
      },
      trigger: ['input', 'blur'],
      message: '请输入IP'
    }
  ],
  port: [
    {
      required: true,
      message: '请输入端口',
      validator(rule: FormItemRule, value: string) {
        console.log(rule)
        if (!/^\d*$/.test(value)) {
          return new Error('端口应该为整数')
        } else if (Number(value) <= 0 || Number(value) >= 65535) {
          return new Error('端口错误')
        }
        return true
      }
    }
  ],
  incount: [
    {
      required: true,
      message: '输入最大可加入人数（超过需要排队加入）',
      trigger: ['input', 'blur'],
      validator(rule: FormItemRule, value: string) {
        console.log(rule)
        if (!/^\d*$/.test(value)) {
          return new Error('人数应该为整数')
        } else if (Number(value) <= 1) {
          return new Error('人数应该大于0')
        }
        return true
      }
    }
  ]
}
const handleValidateButtonClick = (e: MouseEvent) => {
  e.preventDefault()
  formRef.value?.validate((errors) => {
    if (!errors) {
      invoke('read_server').then((_) => {
        let server: ServerFileType[] = JSON.parse(<string>_)
        server.push({
          ip: modelRef.value.ip,
          port: Number(modelRef.value.port),
          incount: Number(modelRef.value.incount),
        })
        invoke('write_server', { jsonString: JSON.stringify(server) }).then((_) => { }).finally(() => {
          message.success('保存成功')
          getAllData()
          modelRef.value = <ModelType>{
            ip: '',
            port: '1',
            incount: '2'
          }
        })
      })
    } else {
      message.error('填写内容不正确！')
    }
  })
}

const handleSelectChange = (value: Boolean) => {
  if (value) {
    getAllData()
  }
}

onMounted(() => {
  invoke('read_from_config_file').then((_) => {
    configInput.value = JSON.parse(<string>_)
  })
  getAllData()
})
</script>

<template>
  <div style="width: 96vw;height: 98vh;">
    <div class="top_group">
      <div class="button_gp">
        <n-button :loading="getAllDataLoading" size="small" @click="getAllData" type="info">刷新</n-button>
        <span style="margin-left:10px;">自动刷新
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-switch @update:value="handleSelectChange" size="small" v-model:value="active" />
            </template>
            开启后每隔10s自动刷新
          </n-tooltip>
        </span>
        <n-button style="margin-left:10px;" @click="readConfig" size="small" type="info">监控地图</n-button>
        <span style="margin-left:10px;">开启监控
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-switch size="small" @update:value="jkUpdateEvent" v-model:value="jkActive" />
            </template>
            开启监控后，会开启自动刷新，并显示监控地图
          </n-tooltip>
        </span>
        <span v-if="jkActive" style="margin-left:10px;">自动加入
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-switch v-model:value="autoInActive" />
            </template>
            <span>
              自动加入规则：只有一个匹配服务器时自动加入，
              <br />
              当有多个匹配服务器时：
              <br />
              当存在无需排队且人数大于20人时自动加入(若最大人数小于20人则自动加入)
              <br />
              当只存在无需排队时自动加入人数最多服务器
              <br />
              当只存在需要排队时随机选择加入
            </span>
          </n-tooltip>
        </span>
      </div>
      <div class="button_gp">
        <n-button @click="addModal = true" size="small" type="primary">添加</n-button>
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
    <n-modal style="width: 400px;" v-model:show="addModal" preset="card" title="添加服务器" size="small" :bordered="true">
      <n-form style="margin: 10px;" label-width="80" label-placement="left" ref="formRef" size="small" :model="modelRef"
        :rules="rules">
        <n-form-item path="ip" label="IP">
          <n-input v-model:value="modelRef.ip" @keydown.enter.prevent />
        </n-form-item>
        <n-form-item path="port" label="端口">
          <n-input v-model:value="modelRef.port" @keydown.enter.prevent />
        </n-form-item>
        <n-form-item path="incount" label="最大人数">
          <n-input v-model:value="modelRef.incount" @keydown.enter.prevent />
        </n-form-item>
        <n-row :gutter="[0, 24]">
          <n-col :span="24">
            <div style="display: flex; justify-content: flex-end">
              <n-button round type="primary" @click="handleValidateButtonClick">
                添加
              </n-button>
            </div>
          </n-col>
        </n-row>
      </n-form>
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
