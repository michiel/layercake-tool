export const invoke = async (command, args) => {
  console.log(`[TAURI MOCK] invoke: ${command}`, args);
  return Promise.resolve(null);
};

export class Channel {
  constructor() {
    this._listeners = [];
  }

  send(data) {
    this._listeners.forEach((listener) => listener({ data }));
  }

  close() {
    this._listeners = [];
  }

  set onmessage(listener) {
    if (typeof listener === 'function') {
      this._listeners = [listener];
    } else {
      this._listeners = [];
    }
  }
}
