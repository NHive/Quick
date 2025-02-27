const envMap = {
  dev: {apiBaseUrl: 'http://localhost:6742'},
  prod: { apiBaseUrl: 'https://cataprv.chuangzhilian.cn' },
  local: {
    apiBaseUrl: 'http://10.11.3.51:8082'
    // apiBaseUrl: 'http://10.11.66.7:8082',
  },
  hktest: { apiBaseUrl: 'https://hk.donz.ai' }
}
export default envMap.dev
