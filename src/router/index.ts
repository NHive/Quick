import { createRouter, createWebHashHistory } from "vue-router"
import Setting from "../views/Setting/index.vue"
import {
  About,
  Account,
  CustomSetting,
  Devices,
  HistoricalRecord,
  Hotkey
} from "@/views/Setting/components"

import Control from "../views/Control/index.vue"

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", component: Control, meta: { requiresAuth: false } },
    {
      path: "/setting",
      component: Setting,
      meta: { requiresAuth: false },
      children: [
        { path: "user", name: "user", component: Account },
        { path: "customSetting", name: "customSetting", component: CustomSetting },
        { path: "historicalRecord", name: "historicalRecord", component: HistoricalRecord },
        { path: "hotkey", name: "hotkey", component: Hotkey },
        { path: "devices", name: "devices", component: Devices },
        { path: "about", name: "about", component: About }
      ]
    },

  ]
})

export default router
