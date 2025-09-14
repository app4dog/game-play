declare module '@capacitor-community/camera-preview' {
  export const CameraPreview: {
    start: (opts: { position: 'rear' | 'front'; toBack: boolean; width?: number; height?: number }) => Promise<void>
    stop: () => Promise<void>
    captureSample: (opts: { quality?: number }) => Promise<{ value?: string; data?: string }>
  }
}

