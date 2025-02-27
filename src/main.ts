import './assets/style/main.css'
import './assets/style/global.scss'
import './assets/font/iconfont.css'
import 'ant-design-vue/dist/reset.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import Antd from 'ant-design-vue'
import App from './App.vue'
import router from './router'
import store from '@/store'
import { i18n } from './utils/i18n'

const init = async () => {
  const app = createApp(App)
  app.use(i18n)
  app.use(createPinia())
  app.use(store)
  app.use(router)
  app.use(Antd)

  app.mount('#app')
}

init()
