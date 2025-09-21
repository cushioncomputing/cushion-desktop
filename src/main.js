import { invoke } from '@tauri-apps/api/core';
import { TrayIcon } from '@tauri-apps/api/tray';
import { Menu } from '@tauri-apps/api/menu';
import { exit } from '@tauri-apps/api/process';

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

async function showWindow() {
  await invoke("show_main_window");
}

async function setupSystemTray() {
  try {
    // Create menu for system tray
    const menu = await Menu.new({
      items: [
        {
          id: 'show',
          text: 'Show Cushion',
          action: showWindow,
        },
        {
          id: 'quit',
          text: 'Quit',
          action: () => {
            exit(0);
          },
        },
      ],
    });

    // Create tray icon
    const tray = await TrayIcon.new({
      menu,
      menuOnLeftClick: false,
      action: (event) => {
        if (event.type === 'Click') {
          showWindow();
        }
      },
    });

  } catch (error) {
    console.error('Failed to setup system tray:', error);
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  // Setup system tray
  await setupSystemTray();

});
