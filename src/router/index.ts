import { createRouter, createWebHashHistory } from "vue-router"
import Setting from "../views/Setting/index.vue"
import {
  About,
  CustomSetting,
  Win,
  Network
} from "@/views/Setting/components"
import Control from "../views/Control/index.vue"
import Home from "../views/Home/index.vue"

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", component: Control, meta: { requiresAuth: false } },
    // { path: "/", component: Home, meta: { requiresAuth: false } },
    {
      path: "/setting",
      component: Setting,
      meta: { requiresAuth: false },
      children: [
        { path: "customSetting", name: "customSetting", component: CustomSetting },
        { path: "network", name: "network", component: Network },
        { path: "win", name: "win", component: Win },
        { path: "about", name: "about", component: About }
      ]
    },
  ]
})

export default router
