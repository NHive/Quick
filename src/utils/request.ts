// fetch封装：
// 设置：超时time
// baseUrl和环境频道
// get、post请求方式
// 请求拦截——请求头—参数类型
// 响应拦截—异常统一处理
/* eslint-disable */
import { fetch } from "@tauri-apps/plugin-http"

export type ReqUrlType = URL | Request | string
export type Body = Blob | BufferSource | FormData | URLSearchParams | string | Object
export type FetchBody = Blob | BufferSource | FormData | URLSearchParams | string
export interface ConfigType {
  timeout?: number,
  headers?: Record<string, any>
}

class request {

  private timeout; // 超时时间422437
  constructor() {
    this.timeout = 2000;
  }

  // 初始化请求url值
  private initFetchUrl(url: ReqUrlType) {
    // 判断是否是绝对路径
    const isAbsoluteURLReg = /^([a-z][a-z\d+-.]*:)?\/\//i;
    // TODO 正式环境 域名
    const baseUrl = import.meta.env.DEV ? 'https://newbee.dev.czbrcj.cn' : 'https://newbee.fast.czbrcj.cn'
    if (typeof url === 'string' && !isAbsoluteURLReg.test(url)) {
      const pathName = /^(\/)/.test(url) ? url.slice(1) : url
      return `${baseUrl}/${pathName}`
    }
    return url
  }
  // 初始化请求配置
  private initFetchConfig(method: string, params?: Body, config?: Record<string, any>) {
    const baseConfig = {
      connectTimeout: this.timeout,
      method,
      headers: {} as Record<string, any>,
      body: '' as FetchBody
    }
    // 根据参数类型设置content-type
    if (!!params) {
      if (typeof params === 'string') {
        baseConfig.headers['content-type'] = "text/plain;charset=UTF-8";
      } else if (params instanceof FormData) {
        baseConfig.headers['content-type'] = "multipart/form-data;charset=UTF-8";
      } else if (typeof params === 'object') {
        baseConfig.headers['content-type'] = "application/json;charset=UTF-8";
        params = JSON.stringify(params)
      } else {
        baseConfig.headers['content-type'] = "application/octet-stream;charset=UTF-8";
      }
    }

    if (config && Object.keys(config).length) {
      baseConfig.connectTimeout = config.timeout ?? this.timeout;
      baseConfig.headers = { ...(config?.headers ?? {}) }
    }
    baseConfig.body = params as FetchBody;
    console.log('baseConfig', baseConfig, baseConfig.body)
    return baseConfig;
  }

  // 请求方法
  private http(url: ReqUrlType, method: string, params?: Body, config?: Record<string, any>) {
    // 模拟请求拦截——请求域名处理
    const fetchUrl = this.initFetchUrl(url)
    // 模拟请求拦截——请求参数处理
    const fetchconfig = this.initFetchConfig(method, params, config)

    return fetch(fetchUrl, fetchconfig)
      .then((res: Response) => {
        console.log('http', res, res.body, !!res.body)
        if (res.body) {
          return res.json().then((result) => {
            console.log('httpData', res, result)
            if (res.status >= 200 && res.status < 400) {
              return Promise.resolve({ status: res.status, data: result });
            }
            // TODO 对特定响应状态做统一处理
            return Promise.reject({ status: res.status, data: result });
          }).catch((err) => {
            console.log('httpError', err)
            return Promise.reject()
          })
        } else {
          if (res.status >= 200 && res.status < 400) {
            return Promise.resolve({ status: res.status });
          }
          // TODO 对特定响应状态做统一处理
          return Promise.reject({ status: res.status });
        }

      })
      .catch((err) => {
        console.error(err);
        return Promise.reject(err);
      });
  }

  // get方法
  public get(url: ReqUrlType, params?: Body, config?: ConfigType) {
    return this.http(url, 'get', params, config);
  }
  // post方法
  public post(url: ReqUrlType, params?: Body, config?: ConfigType) {
    return this.http(url, 'post', params, config);
  }

}
const requestInstance = new request()
export default requestInstance;

