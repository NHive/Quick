import { defineStore } from 'pinia'

export const useLableStore = defineStore('label-set', {
  state: () => ({lableArr: []}),
  getters: {
    getLabel(): any {
      return this.lableArr || []
    }
  },
  actions: {
    // async resetLabel() {
    //     await postDelLabel();
    //     this.setLabel();
    // },
    setLabel(val: any) {
      this.lableArr = val
    }
  },
  persist: {enabled: true}
})
