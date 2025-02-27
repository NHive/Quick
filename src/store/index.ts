import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persist-uni'

const pinia = createPinia()

pinia.use(piniaPluginPersistedstate)

export default pinia
