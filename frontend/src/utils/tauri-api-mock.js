export const invoke = async (command, args) => {
  console.log(`[TAURI MOCK] invoke: ${command}`, args);
  // You can return mock data here based on the command
  return Promise.resolve(null);
};