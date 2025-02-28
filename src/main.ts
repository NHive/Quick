import 'ant-design-vue/dist/reset.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import Antd from 'ant-design-vue'
import App from './App.vue'
import router from './router'
import { i18n } from './utils/i18n'

const init = async () => {
  const app = createApp(App)
  app.use(i18n)
  app.use(createPinia())
  app.use(router)
  app.use(Antd)

  app.mount('#app')
}

init()
