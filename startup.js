// This file is auto-generated to set up the initial DOM and event listeners.
// NOT USED ANYMORE

import { handle_js_event } from './patches.js';

export function buildNodeMap() {
    let nodeMap = [];
    nodeMap[0] = document.createElement('div');

    nodeMap[1] = document.createElement('button');
    nodeMap[1].textContent = "Loading...";
    nodeMap[0].appendChild(nodeMap[1]);

    nodeMap[2] = document.createElement('button');
    nodeMap[2].textContent = "Loading...";
    nodeMap[0].appendChild(nodeMap[2]);

    let buttonMap = [];
    buttonMap[0] = document.createElement('div');

    buttonMap[1] = document.createElement('button');

    nodeMap[1].textContent = "Hello, Svelte!";
    nodeMap[2].textContent = "Click me";

    document.head.querySelector('style').textContent = `
        div {
            font-family: Arial, sans-serif;
            padding: 20px;
        }
        p {
            color: #333;
            font-size: 18px;
        }
    `;

    return nodeMap;
}

export function setupEventListeners(nodeMap) {
    nodeMap[2].addEventListener('click', (e) => {
        handle_js_event(e, nodeMap, 2);
    });
}