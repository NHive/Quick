import { createRouter, createWebHashHistory } from "vue-router"
// import Setting from "../views/Setting/index.vue"
// import {
//   About,
//   Account,
//   CustomSetting,
//   Devices,
//   HistoricalRecord,
//   Hotkey
// } from "@/views/Setting/components"

import Control from "../views/Control/index.vue"
import Home from "../views/Home/index.vue"

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    // { path: "/", component: Home, meta: { requiresAuth: false } },
    { path: "/", component: Control, meta: { requiresAuth: false } },
  ]
})

export default router
