const populateWebGPUBrowserStubs = () => {
  if (typeof window === 'undefined') return

  const win = window as any
  if (typeof win.GPUShaderStage === 'undefined') {
    win.GPUShaderStage = {
      VERTEX: 0x1,
      FRAGMENT: 0x2,
      COMPUTE: 0x4,
    }
    console.log('[projections] WebGPU shader stage stub installed to prevent missing-API errors')
  }
}

populateWebGPUBrowserStubs()
