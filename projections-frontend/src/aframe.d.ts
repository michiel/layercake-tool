// A-Frame type declarations for TypeScript and JSX

/// <reference types="react" />

declare module 'aframe' {
  const aframe: any
  export = aframe
}

declare global {
  namespace JSX {
    interface IntrinsicElements {
      'a-scene': any
      'a-entity': any
      'a-box': any
      'a-sphere': any
      'a-cylinder': any
      'a-plane': any
      'a-sky': any
      'a-camera': any
      'a-light': any
      'a-text': any
      'a-image': any
      'a-video': any
      'a-sound': any
      'a-cursor': any
      'a-gltf-model': any
    }
  }

  interface Window {
    AFRAME: any
  }
}

export {}
